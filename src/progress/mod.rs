//!
//! Module for progress bar addition to searches.
//!
//! Add with `--progress-bar` in `goat-cli search` and
//! `goat-cli newick`.

use futures::StreamExt;
use indicatif;
use serde_json::Value;
use std::time::Duration;
use tokio::time::sleep;

use crate::client::GoatClient;
use crate::error::Result;
use crate::cli::SearchRequest;
use crate::count;
use crate::utils::cli_matches::{self, CliAction};
use crate::{GOAT_URL, UPPER_CLI_SIZE_LIMIT};

// a function to create and display a progress bar
// for large requests. Currently limited to single large requests.

/// Adds a progress bar to large requests: a `search` (`Some(request)`),
/// or a `newick` report (`None`).
pub async fn progress_bar(search: Option<&SearchRequest>, unique_ids: Vec<String>) -> Result<()> {
    // nothing is fetched when only printing the URLs
    if let Some(request) = search {
        if request.output.url || request.output.goat_ui_url {
            return Ok(());
        }
    }
    // wait briefly before submitting
    // so we are sure the API has recieved and set the queryId
    sleep(Duration::from_secs(2)).await;
    let (size_int, url_vector_api) = match search {
        None => (0u64, vec!["init".to_string()]),
        Some(request) => {
            match cli_matches::process_cli_args(request, "search", unique_ids.clone())? {
                CliAction::Continue { size, urls, .. } => (size, urls),
                CliAction::PrintedAndExit => return Ok(()),
            }
        }
    };

    let concurrent_requests = url_vector_api.len();

    // For search/count-style commands, use a count preflight to decide
    // whether a progress bar is worthwhile. Newick does not share the same
    // CLI shape, so do not route it through count::count/process_cli_args.
    if let Some(request) = search {
        let no_query_hits = count::count(request, false, false, unique_ids.clone())
            .await?
            .unwrap();

        if no_query_hits < 10000 || size_int < 10000 {
            return Ok(());
        }
    }

    // add the query ID's to a vec
    let mut query_id_vec = Vec::new();
    for i in unique_ids.iter().take(concurrent_requests) {
        let query_id = format!("{}progress?queryId=goat_cli_{}", *GOAT_URL, i);
        query_id_vec.push(query_id);
    }

    // hidden until GoaT reports a total, so it never shows a made-up length
    let bar = indicatif::ProgressBar::hidden();
    let bar_style = ("█▓▓▒░░░ ", "magenta");
    bar.set_style(
        indicatif::ProgressStyle::default_bar()
            .template(&format!(
                "{{prefix:.bold}}▕{{bar:57.{}}}▏{{pos}}/{{len}} {{wide_msg}}",
                bar_style.1
            ))?
            .progress_chars(bar_style.0),
    );
    bar.set_prefix("Fetching from GoaT: ");

    // the last (done, total) GoaT reported for each query. GoaT stops
    // reporting a query once it is finished, which then counts as done.
    let mut last: Vec<Option<(u64, u64)>> = vec![None; query_id_vec.len()];
    let client = GoatClient::new();
    loop {
        let fetches = futures::stream::iter(query_id_vec.clone().into_iter().map(|path| {
            let client = client.clone();
            async move {
                let v: Value = client.get_json(&path).await?;
                Ok(match (v["progress"]["x"].as_u64(), v["progress"]["total"].as_u64()) {
                    (Some(x), Some(total)) => Some((x, total)),
                    _ => None,
                })
            }
        }))
        .buffered(crate::client::concurrency(concurrent_requests))
        .collect::<Vec<Result<Option<(u64, u64)>>>>()
        .await;

        let mut running = false;
        for (seen, polled) in last.iter_mut().zip(fetches) {
            match (polled?, *seen) {
                (Some(now), _) => {
                    *seen = Some(now);
                    running = true;
                }
                (None, Some((_, total))) => *seen = Some((total, total)),
                (None, None) => {}
            }
        }
        let (done, mut total) = last
            .iter()
            .flatten()
            .fold((0, 0), |(d, t), (x, n)| (d + x, t + n));
        // a search's total is all hits; the bar shows those fetched
        if search.is_some() && total > *UPPER_CLI_SIZE_LIMIT as u64 {
            total = size_int;
        }

        if total > 0 {
            if bar.is_hidden() {
                bar.set_draw_target(indicatif::ProgressDrawTarget::stderr());
            }
            // GoaT's count can run a little past its total
            bar.set_length(total.max(done));
            bar.set_position(done);
        }
        // GoaT drops a finished search from /progress, but keeps a finished
        // report there at done == total
        if !running || (total > 0 && done >= total) {
            break;
        }

        sleep(Duration::from_millis(500)).await;
    }
    if !bar.is_hidden() {
        bar.set_position(bar.length().unwrap_or(0));
        bar.finish();
    }

    Ok(())
}
