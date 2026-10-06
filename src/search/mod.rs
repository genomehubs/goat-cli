//!
//! Invoked by calling:
//! `goat-cli search <args>`

use futures::{FutureExt, StreamExt};
use serde_json::{json, Value};

use crate::client::GoatClient;
use crate::error::Result;
use crate::utils::cli_matches::CliAction;
use crate::utils::{cli_matches, utils};
use crate::cli::SearchRequest;
use crate::count;
use crate::output::Format;

/// Execute the `search` subcommand from `goat-cli`. Print a TSV, CSV or
/// JSON (`--format`).
pub async fn search(request: &SearchRequest, unique_ids: Vec<String>) -> Result<()> {
    let url_vector_api =
        match cli_matches::process_cli_args(request, "search", unique_ids.clone())? {
            CliAction::Continue { urls, .. } => urls,
            CliAction::PrintedAndExit => return Ok(()),
        };

    let concurrent_requests = url_vector_api.len();

    let format = request.output.format;
    let client = GoatClient::new();
    let fetches = futures::stream::iter(url_vector_api.into_iter().map(|path| {
        let client = client.clone();
        async move { client.get_text(&path, format.accept()).await }
    }))
    .buffered(crate::client::concurrency(concurrent_requests))
    .collect::<Vec<_>>();

    // the count is only used to print warnings, so fetch it alongside the
    // search rather than before it.
    let (_, awaited_fetches) = futures::try_join!(
        count::count(request, false, true, unique_ids),
        fetches.map(Ok)
    )?;

    match format {
        Format::Json => merge_json(awaited_fetches.into_iter().collect::<Result<Vec<_>>>()?)?,
        // GoaT's TSV and CSV have one header line, then a row per line
        Format::Tsv | Format::Csv => utils::format_tsv_output(awaited_fetches)?,
    }

    Ok(())
}

/// Print the JSON responses for each taxon as one response, with all of
/// their results and the total hits.
fn merge_json(responses: Vec<String>) -> Result<()> {
    let mut hits = 0;
    let mut results = Vec::new();
    for response in &responses {
        let mut response: Value = serde_json::from_str(response)?;
        hits += response["status"]["hits"].as_u64().unwrap_or(0);
        if let Value::Array(r) = response["results"].take() {
            results.extend(r);
        }
    }
    let merged = json!({"status": {"success": true, "hits": hits}, "results": results});
    crate::outln!("{}", merged)?;
    Ok(())
}
