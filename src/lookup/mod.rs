//!
//! Invoked by calling:
//! `goat-cli lookup <args>`

use anyhow::{bail, Result};
use futures::StreamExt;
use reqwest;
use reqwest::header::ACCEPT;
use serde_json::Value;

/// The inner structs used in lookup.
pub mod lookup;
use lookup::{Collector, Lookups};

/// Main entry point for `goat-cli lookup`.
pub async fn lookup(matches: &clap::ArgMatches, cli: bool) -> Result<Option<Vec<String>>> {
    let lookups = Lookups::new(matches)?;
    let url_vector_api = lookups.make_urls();
    let print_url = matches.is_present("url");
    let size = matches.value_of("size").unwrap();

    if print_url {
        for (index, (url, _)) in url_vector_api.iter().enumerate() {
            println!("{}.\tGoaT lookup API URL: {}", index, url);
        }
        // don't exit here internally; we'll exit later
        if cli {
            std::process::exit(0);
        }
    }
    // so we can make as many concurrent requests
    let concurrent_requests = url_vector_api.len();

    let fetches = futures::stream::iter(url_vector_api.into_iter().map(|(path, search_query)| async move {
        // possibly make a again::RetryPolicy
        // to catch all the values in a *very* large request.
        let client = reqwest::Client::new();

        match again::retry(|| client.get(&path).header(ACCEPT, "application/json").send()).await {
            Ok(resp) => match resp.text().await {
                Ok(body) => {
                    let v: Value = serde_json::from_str(&body)?;
                    // print a warning if number of hits > size specified.
                    let request_size_op = &v["status"]["hits"].as_u64();
                    match request_size_op {
                        Some(s) => {
                            if size.parse::<u64>()? < *s {
                                eprintln!(
                                "[-]\tFor seach query {}, size specified ({}) was less than the number of results returned, ({}).",
                                search_query, size, s
                            )
                        }
                    },
                        None => (),
                    }

                    // get all the suggestions
                    let suggestions_text_op = &v["suggestions"].as_array();
                    // collect into a vec
                    let mut suggestions_vec = Vec::new();
                    let suggestions_text = match suggestions_text_op {
                        Some(suggestions) => {
                            for el in *suggestions {
                                let sug_str = el["suggestion"]["text"].as_str();
                                let sug_string_op = sug_str.map(String::from);
                                suggestions_vec.push(sug_string_op);
                            }
                            Some(suggestions_vec.clone())
                        }
                        None => None,
                    };
                    // and the taxon ID
                    // we need to iterate over the array of results.
                    // potentially look at the scores, and keep those over a certain amount
                    // or keep everything. Currently I'm missing synonymous genera.

                    let mut taxon_id_vec = Vec::new();
                    let mut taxon_rank_vec = Vec::new();
                    let mut taxon_names_array_vec = Vec::new();

                    let results_array = v["results"].as_array();
                    // unwrap safely here
                    match results_array {
                        Some(arr) => {
                            for el in arr {
                                let taxon_id = el["result"]["taxon_id"].as_str();
                                let taxon_rank = el["result"]["taxon_rank"].as_str();
                                let taxon_names_array_op = el["result"]["taxon_names"].as_array();

                                let taxon_names_array = match taxon_names_array_op {
                                    Some(vec) => {
                                        let mut collect_names = Vec::new();
                                        for el in vec.into_iter() {
                                            let key = el["name"].as_str().unwrap_or("-");
                                            let value = el["class"].as_str().unwrap_or("-");
                                            // let source = el["source"].as_str().unwrap_or("-");
                                            collect_names
                                                .push((key.to_string(), value.to_string()));
                                        }
                                        Some(collect_names)
                                    }
                                    None => None,
                                };

                                // gather results into the vecs
                                taxon_id_vec.push(taxon_id);
                                taxon_rank_vec.push(taxon_rank);
                                taxon_names_array_vec.push(taxon_names_array);
                            }
                        }
                        None => {}
                    }

                    // Vec<Option<&str>> -> Vec<Option<String>>
                    let taxon_id = taxon_id_vec.iter().map(|e| e.map(String::from)).collect();
                    let taxon_rank = taxon_rank_vec.iter().map(|e| e.map(String::from)).collect();

                    Ok(Collector {
                        search: Some(search_query.to_string()),
                        suggestions: suggestions_text,
                        taxon_id,
                        taxon_names: taxon_names_array_vec,
                        taxon_rank,
                    })
                }
                Err(_) => bail!("ERROR reading {}", path),
            },
            Err(_) => bail!("ERROR downloading {}", path),
        }
    }))
    .buffer_unordered(concurrent_requests)
    .collect::<Vec<_>>();

    let awaited_fetches = fetches.await;

    let mut return_taxid_vec: Vec<String> = Vec::new();

    for (index, el) in awaited_fetches.iter().enumerate() {
        match el {
            Ok(e) => {
                if cli {
                    e.print_result(index)?;
                } else {
                    // dead code currently
                    let taxid_op = e.return_taxid_vec()?;
                    match taxid_op {
                        Some(s) => return_taxid_vec.push(s),
                        // the None variant can't push a "",
                        // otherwise the URL hangs.
                        None => return_taxid_vec.push("-".to_string()),
                    }
                }
            }
            Err(_) => bail!("No results found."),
        }
    }

    Ok(Some(return_taxid_vec))
}
