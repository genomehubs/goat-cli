//!
//! Invoked by calling:
//! `goat-cli taxon/assembly lookup <args>`

use crate::client::GoatClient;
use crate::error::Result;
use crate::cli::LookupArgs;
use crate::output::Table;
use crate::IndexType;
use futures::StreamExt;
use serde_json::Value;

/// The inner structs used in lookup.
pub mod lookup;
use lookup::{AssemblyCollector, Collector, Lookups, TaxonCollector};

pub enum LookupAction {
    Continue,
    PrintedAndExit,
}

/// Main entry point for `goat-cli lookup`.
pub async fn lookup(
    args: &LookupArgs,
    cli: bool,
    index_type: IndexType,
) -> Result<LookupAction> {
    let lookups = Lookups::new(args, index_type)?;
    let url_vector_api = lookups.make_urls();
    let size = args.size;

    if args.url {
        for (index, (url, _)) in url_vector_api.iter().enumerate() {
            crate::outln!("{}.\tGoaT lookup API URL: {}", index, url)?;
        }
        return Ok(LookupAction::PrintedAndExit);
    }

    let concurrent_requests = url_vector_api.len();

    let client = GoatClient::new();
    let fetches = futures::stream::iter(
        url_vector_api
            .into_iter()
            .enumerate()
            .map(|(idx, (path, search_query))| {
                let client = client.clone();
                async move {
                    let v: Value = client.get_json(&path).await?;
                            // print a warning if number of hits > size specified.
                            let request_size_op = &v["status"]["hits"].as_u64();
                            match request_size_op {
                                Some(s) => {
                                    if size < *s {
                                        eprintln!(
                                            "For search query {}, size specified ({}) was less than the number of results returned, ({}).",
                                            search_query, size, s
                                        )
                                    }
                                }
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

                            // we have all the information to process the results
                            let collector = match index_type {
                                IndexType::Taxon => Collector::Taxon(process_taxon_results(
                                    v,
                                    search_query,
                                    suggestions_text,
                                )),
                                IndexType::Assembly => Collector::Assembly(
                                    process_assembly_results(v, search_query, suggestions_text),
                                ),
                            };

                    Ok((idx, collector))
                }
            }),
    )
    .buffer_unordered(crate::client::concurrency(concurrent_requests))
    .collect::<Vec<_>>();

    let awaited_fetches = fetches.await;

    let mut ordered_results = Vec::new();
    for el in awaited_fetches {
        match el {
            Ok(v) => ordered_results.push(v),
            Err(e) => return Err(e),
        }
    }

    ordered_results.sort_by_key(|(idx, _)| *idx);

    if cli {
        let mut table = Table::new(match index_type {
            IndexType::Taxon => TaxonCollector::HEADER,
            IndexType::Assembly => AssemblyCollector::HEADER,
        });
        for (_original_idx, collector) in ordered_results {
            match collector {
                Collector::Taxon(e) => e.add_rows(&mut table),
                Collector::Assembly(e) => e.add_rows(&mut table),
            }
        }
        table.print(args.format)?;
    }

    Ok(LookupAction::Continue)
}

/// As the taxon and assembly return JSON's are in
/// different structures, they have to be parsed differently.
///
/// Each must return [`Result<Collector, anyhow::Error>`].
fn process_taxon_results(
    v: Value,
    search_query: String,
    suggestions_text: Option<Vec<Option<String>>>,
) -> TaxonCollector {
    // and the taxon ID
    // we need to iterate over the array of results.
    // potentially look at the scores, and keep those over a certain amount
    // or keep everything. Currently I'm missing synonymous genera.

    let mut taxon_id_vec = Vec::new();
    let mut taxon_rank_vec = Vec::new();
    let mut taxon_names_array_vec = Vec::new();

    let results_array = v["results"].as_array();
    // unwrap safely here
    if let Some(arr) = results_array {
        for el in arr {
            let taxon_id = el["result"]["taxon_id"].as_str();
            let taxon_rank = el["result"]["taxon_rank"].as_str();
            let taxon_names_array_op = el["result"]["taxon_names"].as_array();

            let taxon_names_array = match taxon_names_array_op {
                Some(vec) => {
                    let mut collect_names = Vec::new();
                    for el in vec.iter() {
                        let key = el["name"].as_str().unwrap_or("-");
                        let value = el["class"].as_str().unwrap_or("-");
                        // let source = el["source"].as_str().unwrap_or("-");
                        collect_names.push((key.to_string(), value.to_string()));
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

    // Vec<Option<&str>> -> Vec<Option<String>>
    let taxon_id = taxon_id_vec.iter().map(|e| e.map(String::from)).collect();
    let taxon_rank = taxon_rank_vec.iter().map(|e| e.map(String::from)).collect();

    TaxonCollector {
        search: Some(search_query),
        suggestions: suggestions_text,
        taxon_id,
        taxon_names: taxon_names_array_vec,
        taxon_rank,
    }
}

/// The assembly counterpart to the above function.
fn process_assembly_results(
    v: Value,
    search_query: String,
    suggestions_text: Option<Vec<Option<String>>>,
) -> AssemblyCollector {
    // taxon ID stays the same
    let mut taxon_id_vec = Vec::new();
    // there is no taxon rank
    let mut identifiers_array_vec = Vec::new();

    let results_array = v["results"].as_array();
    // unwrap safely here
    if let Some(arr) = results_array {
        for el in arr {
            let taxon_id = el["result"]["taxon_id"].as_str();
            let identifiers_array_op = el["result"]["identifiers"].as_array();

            let identifiers_array = match identifiers_array_op {
                Some(vec) => {
                    let mut collect_names = Vec::new();
                    for el in vec.iter() {
                        let key = el["identifier"].as_str().unwrap_or("-");
                        let value = el["class"].as_str().unwrap_or("-");
                        // let source = el["source"].as_str().unwrap_or("-");
                        collect_names.push((key.to_string(), value.to_string()));
                    }
                    Some(collect_names)
                }
                None => None,
            };

            // gather results into the vecs
            taxon_id_vec.push(taxon_id);
            identifiers_array_vec.push(identifiers_array);
        }
    }

    // Vec<Option<&str>> -> Vec<Option<String>>
    let taxon_id = taxon_id_vec.iter().map(|e| e.map(String::from)).collect();

    AssemblyCollector {
        search: Some(search_query),
        suggestions: suggestions_text,
        taxon_id,
        identifiers: identifiers_array_vec,
    }
}
