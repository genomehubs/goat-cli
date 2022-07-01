//!
//! Module for progress bar addition to searches.
//!
//! Add with `--progress-bar` in `goat-cli search` and
//! `goat-cli newick`.

use anyhow::{bail, ensure, Result};
use async_std::task;
use futures::StreamExt;
use indicatif;
use reqwest;
use reqwest::header::ACCEPT;
use serde_json::Value;
use std::time::Duration;

use crate::utils::cli_matches;
use crate::{count, IndexType};
use crate::{GOAT_URL, UPPER_CLI_SIZE_LIMIT};

// a function to create and display a progress bar
// for large requests. Currently limited to single large requests.

/// Adds a progress bar to large requests.
pub async fn progress_bar(
    matches: &clap::ArgMatches,
    api: &str,
    unique_ids: Vec<String>,
    index_type: IndexType,
) -> Result<()> {
    // wait briefly before submitting
    // so we are sure the API has recieved and set the queryId
    task::sleep(Duration::from_secs(2)).await;
    // TODO: clean this up.
    let (size_int, _url_vector, url_vector_api) = match api {
        // doesn't matter what is in the vecs, they just need to be length 1
        // as newick only supports single url calls right now.
        // this is really bad coding...
        "newick" => (0u64, vec!["init".to_string()], vec!["init".to_string()]),
        other => cli_matches::process_cli_args(matches, other, unique_ids.clone(), index_type.clone())?,
    };

    ensure!(
        unique_ids.len() == url_vector_api.len(),
        "No reason these lengths should be different."
    );

    let concurrent_requests = url_vector_api.len();

    // should be fine to always unwrap this
    let no_query_hits = count::count(matches, false, false, unique_ids.clone(), index_type)
        .await?
        .unwrap();
    // might need tweaking...
    // special case newick
    if api != "newick" {
        // I think these actually need to be
        // 10,000... but that's our upper limit for search
        if no_query_hits < 10000 || size_int < 10000 {
            return Ok(());
        }
    }

    // add the query ID's to a vec
    let mut query_id_vec = Vec::new();
    for i in 0..concurrent_requests {
        let query_id = format!("{}progress?queryId=goat_cli_{}", *GOAT_URL, unique_ids[i]);
        query_id_vec.push(query_id);
    }

    // we want to wrap this in a loop
    // and break when sum(progress_x) == sum(progress_total)
    let bar = indicatif::ProgressBar::new(512);
    let bar_style = ("█▓▓▒░░░ ", "magenta");
    bar.set_style(
        indicatif::ProgressStyle::default_bar()
            .template(&format!(
                "{{prefix:.bold}}▕{{bar:57.{}}}▏{{pos}}/{{len}} {{wide_msg}}",
                bar_style.1
            ))
            .progress_chars(bar_style.0),
    );
    bar.set_prefix("Fetching from GoaT: ");

    loop {
        // main body
        let fetches =
            futures::stream::iter(query_id_vec.clone().into_iter().map(|path| async move {
                // possibly make a again::RetryPolicy
                // to catch all the values in a *very* large request.
                let client = reqwest::Client::new();

                match again::retry(|| client.get(&path).header(ACCEPT, "application/json").send())
                    .await
                {
                    Ok(resp) => match resp.text().await {
                        Ok(body) => {
                            let v: Value = serde_json::from_str(&body)?;

                            match &v["progress"] {
                                Value::Null => Ok(None),
                                Value::Bool(_b) => Ok(None),
                                Value::Number(_n) => Ok(None),
                                Value::String(_s) => Ok(None),
                                Value::Array(_a) => Ok(None),
                                Value::Object(_o) => {
                                    let progress_total = match v["progress"]["total"].as_u64() {
                                        Some(p) => Some(p),
                                        None => None,
                                    };
                                    let progress_x = match v["progress"]["x"].as_u64() {
                                        Some(p) => Some(p),
                                        None => None,
                                    };

                                    Ok(Some((progress_x, progress_total)))
                                }
                            }
                        }
                        Err(_) => bail!("ERROR reading {}", path),
                    },
                    Err(_) => bail!("ERROR downloading {}", path),
                }
            }))
            .buffered(concurrent_requests)
            // complicated. Each u64 can be an option, as some
            // queries will finish before others
            // entire tuple is an option, as other progress enums evaluate to None.
            .collect::<Vec<Result<Option<(Option<u64>, Option<u64>)>>>>();

        let awaited_fetches = fetches.await;
        // what's going on here?
        let progress_total: Result<Vec<_>, _> = awaited_fetches.into_iter().collect();

        let mut progress_x_total = 0;
        let mut progress_total_total = 0;
        for el in progress_total.unwrap() {
            let x_tot_tup = match el {
                Some(t) => t,
                None => (None, None),
            };
            progress_x_total += x_tot_tup.0.unwrap_or(0);
            progress_total_total += x_tot_tup.1.unwrap_or(0);
        }

        // special case newick
        match api {
            "newick" => bar.set_length(progress_total_total),
            _ => match progress_total_total > *UPPER_CLI_SIZE_LIMIT as u64 {
                true => bar.set_length(size_int),
                false => bar.set_length(progress_total_total),
            },
        }

        bar.set_position(progress_x_total);

        if progress_x_total >= progress_total_total {
            break;
        }

        task::sleep(Duration::from_millis(1)).await;
    }

    Ok(())
}
