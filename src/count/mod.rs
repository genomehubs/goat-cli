//!
//! Invoked by calling:
//! `goat-cli count <args>`

use crate::client::GoatClient;
use crate::error::{Error, ErrorKind, Result};
use futures::StreamExt;
use serde_json::json;

use crate::cli::SearchRequest;
use crate::output::Table;
use crate::utils::cli_matches::{self, CliAction};

/// How to show a search query to the user; an empty one (no `-t`/`-f`)
/// searches all taxa.
fn query_label(search_query: &str) -> &str {
    if search_query.is_empty() {
        "<all taxa>"
    } else {
        search_query
    }
}

/// `goat-cli count` presents an identical CLI to `goat-cli search` but prints
/// to the console in the CLI call here, and to the stderr in the `goat-cli search` call.
pub async fn count(
    request: &SearchRequest,
    cli: bool,
    print_warning: bool,
    unique_ids: Vec<String>,
) -> Result<Option<u64>> {
    let (size_int, url_vector, url_vector_api) =
        match cli_matches::process_cli_args(request, "count", unique_ids)? {
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
    .buffered(crate::client::concurrency(concurrent_requests))
    .collect::<Vec<_>>();

    let awaited_fetches = fetches.await;

    match cli {
        true => {
            // print to console
            let mut outer_count = 0;
            let mut table = Table::new(&["search_query", "count"]);
            for el in awaited_fetches {
                let (search_query, count) = el?;
                table.push(vec![json!(query_label(&search_query)), json!(count)]);
                outer_count += count;
            }
            table.print(request.output.format)?;
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
                if print_warning && count == 0 {
                    eprintln!("No results for search query {}.", query_label(&search_query));
                } else if print_warning && size_int < count {
                    eprintln!(
                        "For search query {}, size specified ({}) was less than the number of results returned, ({}).",
                        query_label(&search_query), size_int, count
                    );
                }
                outer_count += count;
            }

            Ok(Some(outer_count))
        }
    }
}
