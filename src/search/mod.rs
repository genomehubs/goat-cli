//!
//! Invoked by calling:
//! `goat-cli search <args>`

use futures::{FutureExt, StreamExt};

use crate::client::GoatClient;
use crate::error::Result;
use crate::utils::cli_matches::CliAction;
use crate::utils::{cli_matches, utils};
use crate::cli::SearchRequest;
use crate::count;

/// Execute the `search` subcommand from `goat-cli`. Print a TSV.
pub async fn search(request: &SearchRequest, unique_ids: Vec<String>) -> Result<()> {
    let url_vector_api =
        match cli_matches::process_cli_args(request, "search", unique_ids.clone())? {
            CliAction::Continue { urls, .. } => urls,
            CliAction::PrintedAndExit => return Ok(()),
        };

    let concurrent_requests = url_vector_api.len();

    let client = GoatClient::new();
    let fetches = futures::stream::iter(url_vector_api.into_iter().map(|path| {
        let client = client.clone();
        async move { client.get_text(&path, "text/tab-separated-values").await }
    }))
    .buffered(crate::client::concurrency(concurrent_requests))
    .collect::<Vec<_>>();

    // the count is only used to print warnings, so fetch it alongside the
    // search rather than before it.
    let (_, awaited_fetches) = futures::try_join!(
        count::count(request, false, true, unique_ids),
        fetches.map(Ok)
    )?;

    utils::format_tsv_output(awaited_fetches)?;

    Ok(())
}
