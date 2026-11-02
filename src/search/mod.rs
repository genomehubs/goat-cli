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

/// Print the JSON responses for each taxon as one response.
fn merge_json(responses: Vec<String>) -> Result<()> {
    crate::outln!("{}", merged_json(responses)?)?;
    Ok(())
}

/// GoaT's JSON response for one taxon, as is; for several, the first
/// response with the results of all of them and the total hits.
fn merged_json(mut responses: Vec<String>) -> Result<String> {
    if responses.len() <= 1 {
        return Ok(responses.pop().unwrap_or_default());
    }
    let mut merged: Value = serde_json::from_str(&responses[0])?;
    let mut hits = 0;
    let mut results = Vec::new();
    for response in &responses {
        let mut response: Value = serde_json::from_str(response)?;
        hits += response["status"]["hits"].as_u64().unwrap_or(0);
        if let Value::Array(r) = response["results"].take() {
            results.extend(r);
        }
    }
    merged["status"]["hits"] = json!(hits);
    merged["results"] = Value::Array(results);
    Ok(merged.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_response_is_unchanged() {
        let response = r#"{"status":{"hits":1,"took":5},"results":[{"id":1}],"fields":["x"],"query":{}}"#;
        assert_eq!(merged_json(vec![response.to_string()]).unwrap(), response);
    }

    #[test]
    fn test_merge_keeps_first_response_and_combines_results() {
        let a = r#"{"status":{"hits":1,"success":true,"took":5},"results":[{"id":1}],"fields":["x"]}"#;
        let b = r#"{"status":{"hits":2,"success":true,"took":9},"results":[{"id":2},{"id":3}],"fields":["x"]}"#;
        let merged: Value = serde_json::from_str(&merged_json(vec![a.into(), b.into()]).unwrap()).unwrap();
        assert_eq!(merged["status"], json!({"hits": 3, "success": true, "took": 5}));
        assert_eq!(merged["results"], json!([{"id": 1}, {"id": 2}, {"id": 3}]));
        assert_eq!(merged["fields"], json!(["x"]));
    }
}
