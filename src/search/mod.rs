//!
//! Invoked by calling:
//! `goat-cli search <args>`

use futures::{FutureExt, StreamExt};

use crate::client::GoatClient;
use crate::error::Result;
use crate::utils::cli_matches::CliAction;
use crate::utils::{cli_matches, utils};
use crate::{count, IndexType};

/// Execute the `search` subcommand from `goat-cli`. Print a TSV.
pub async fn search(
    matches: &clap::ArgMatches,
    unique_ids: Vec<String>,
    index_type: IndexType,
) -> Result<()> {
    let url_vector_api =
        match cli_matches::process_cli_args(matches, "search", unique_ids.clone(), index_type)? {
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
        count::count(matches, false, true, unique_ids, index_type),
        fetches.map(Ok)
    )?;

    utils::format_tsv_output(awaited_fetches)?;

    Ok(())
}
