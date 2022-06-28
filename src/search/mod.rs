//!
//! Invoked by calling:
//! `goat-cli search <args>`

use anyhow::{bail, Result};
use futures::StreamExt;
use reqwest;
use reqwest::header::ACCEPT;

use crate::utils::{cli_matches, utils};
use crate::{count, IndexType};

/// Execute the `search` subcommand from `goat-cli`. Print a TSV.
pub async fn search(
    matches: &clap::ArgMatches,
    unique_ids: Vec<String>,
    index_type: IndexType,
) -> Result<()> {
    let (_size_int, _url_vector, url_vector_api) =
        cli_matches::process_cli_args(matches, "search", unique_ids.clone(), index_type.clone())?;
    let concurrent_requests = url_vector_api.len();

    // print count warnings.
    count::count(matches, false, true, unique_ids, index_type).await?;

    let fetches = futures::stream::iter(url_vector_api.into_iter().map(|path| async move {
        // possibly make a again::RetryPolicy
        // to catch all the values in a *very* large request.
        let client = reqwest::Client::new();

        match again::retry(|| {
            client
                .get(&path)
                .header(ACCEPT, "text/tab-separated-values")
                .send()
        })
        .await
        {
            Ok(resp) => match resp.text().await {
                Ok(body) => Ok(body),
                Err(_) => bail!("ERROR reading {}", path),
            },
            Err(_) => bail!("ERROR downloading {}", path),
        }
    }))
    .buffered(concurrent_requests)
    .collect::<Vec<_>>();

    let awaited_fetches = fetches.await;

    utils::format_tsv_output(awaited_fetches)?;

    Ok(())
}
