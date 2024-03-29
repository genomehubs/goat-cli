//!
//! Invoked by calling:
//! `goat-cli count <args>`

use crate::error::{Error, ErrorKind, Result};
use futures::StreamExt;
use reqwest;
use reqwest::header::ACCEPT;
use serde_json::Value;

use crate::utils::cli_matches;
use crate::IndexType;

/// `goat-cli count` presents an identical CLI to `goat-cli search` but prints
/// to the console in the CLI call here, and to the stderr in the `goat-cli search` call.
pub async fn count(
    matches: &clap::ArgMatches,
    cli: bool,
    print_warning: bool,
    unique_ids: Vec<String>,
    index_type: IndexType,
) -> Result<Option<u64>> {
    let (size_int, url_vector, url_vector_api) =
        cli_matches::process_cli_args(matches, "count", unique_ids, index_type)?;
    let concurrent_requests = url_vector_api.len();

    let fetches = futures::stream::iter(url_vector_api.iter().map(|path| async move {
        // possibly make a again::RetryPolicy
        // to catch all the values in a *very* large request.
        let client = reqwest::Client::new();

        match again::retry(|| client.get(path).header(ACCEPT, "application/json").send()).await {
            Ok(resp) => match resp.text().await {
                Ok(body) => {
                    let v: Value = match serde_json::from_str(&body) {
                        Ok(inner_v) => inner_v,
                        Err(err) => return Err(Error::new(ErrorKind::SerdeJSON(err))),
                    };
                    let count = &v["count"].as_u64();
                    match count {
                        Some(c) => Ok(*c),
                        None => Ok(0), // bail!("Bad response."),
                    }
                }
                Err(err) => Err(Error::new(ErrorKind::Reqwest(err))),
            },
            Err(err) => Err(Error::new(ErrorKind::Reqwest(err))),
        }
    }))
    .buffered(concurrent_requests)
    .collect::<Vec<_>>();

    let awaited_fetches = fetches.await;

    match cli {
        true => {
            // print to console
            let mut outer_count = 0;
            println!("search_query\tcount");
            for (el1, el2) in awaited_fetches.into_iter().zip(url_vector.iter()) {
                let count = match el1 {
                    Ok(e) => e,
                    Err(e) => return Err(e),
                };
                println!("{}\t{}", el2, count);
                outer_count += count;
            }
            Ok(Some(outer_count))
        }
        false => {
            // need
            let mut outer_count = 0;
            // the zip does not correspond to the awaited fetches...
            // need to match them
            for (el1, el2) in awaited_fetches.into_iter().zip(url_vector.iter()) {
                let count = match el1 {
                    Ok(e) => e,
                    Err(e) => return Err(e),
                };
                if print_warning && size_int < count {
                    eprintln!("For search query {}, size specified ({}) was less than the number of results returned, ({}).", el2, size_int, count)
                }
                outer_count += count;
            }

            Ok(Some(outer_count))
        }
    }
}
