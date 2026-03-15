//!
//! Invoked by calling:
//! `goat-cli count <args>`

use crate::client::GoatClient;
use crate::error::{Error, ErrorKind, Result};
use futures::StreamExt;

use crate::utils::cli_matches::{self, CliAction};
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
        match cli_matches::process_cli_args(matches, "count", unique_ids, index_type)? {
            CliAction::Continue { size, taxa, urls } => (size, taxa, urls),
            CliAction::PrintedAndExit => return Ok(None),
        };

    let concurrent_requests = url_vector_api.len();

    let client = GoatClient::new();
    let fetches = futures::stream::iter(
        url_vector_api
            .into_iter()
            .zip(url_vector.iter().cloned())
            .map(|(path, search_query)| {
                let client = client.clone();
                async move {
                    let v = client.get_json(&path).await?;
                    let count = v["count"].as_u64().ok_or_else(|| {
                        Error::new(ErrorKind::GenericCli(format!(
                            "Bad count response: {:?}",
                            v
                        )))
                    })?;
                    Ok((search_query, count))
                }
            }),
    )
    .buffered(concurrent_requests)
    .collect::<Vec<_>>();

    let awaited_fetches = fetches.await;

    match cli {
        true => {
            // print to console
            let mut outer_count = 0;
            println!("search_query\tcount");
            for el in awaited_fetches {
                let (search_query, count) = match el {
                    Ok(e) => e,
                    Err(e) => return Err(e),
                };
                println!("{}\t{}", search_query, count);
                outer_count += count;
            }
            Ok(Some(outer_count))
        }
        false => {
            // need
            let mut outer_count = 0;
            // the zip does not correspond to the awaited fetches...
            // need to match them
            for el in awaited_fetches {
                let (search_query, count) = match el {
                    Ok(e) => e,
                    Err(e) => return Err(e),
                };
                if print_warning && size_int < count {
                    eprintln!(
                        "For search query {}, size specified ({}) was less than the number of results returned, ({}).",
                        search_query, size_int, count
                    );
                }
                outer_count += count;
            }

            Ok(Some(outer_count))
        }
    }
}
