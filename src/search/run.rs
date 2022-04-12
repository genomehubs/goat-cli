use anyhow::{bail, Result};
use futures::StreamExt;
use reqwest;
use reqwest::header::ACCEPT;

use crate::count::count;
use crate::utils::{cli_matches, utils};

/// Execute the `search` subcommand from `goat-cli`. Print a TSV.
pub async fn search(matches: &clap::ArgMatches) -> Result<()> {
    let (_size_int, _url_vector, url_vector_api) =
        cli_matches::process_cli_args(matches, "search")?;
    let concurrent_requests = url_vector_api.len();

    // print count warnings.
    count::count(matches, false, true).await?;

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
