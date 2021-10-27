use crate::utils::url::{GOAT_URL, TAXONOMY};
use crate::utils::utils::{lines_from_file, make_goat_search_urls, parse_multiple_taxids};
use crate::{search::agg_values::Records, search::output::*, search::raw_values::RawRecords};

use anyhow::{bail, Result};
use futures::StreamExt;
use reqwest;
use serde_json::Value;

pub async fn search<'a>(matches: &clap::ArgMatches<'a>) -> Result<()> {
    // should do some checking on the name,
    // and also parse comma separated names/taxids
    let include_raw_values = matches.is_present("raw");
    let assembly = matches.is_present("assembly");
    let cvalues = matches.is_present("c-values");
    let karyotype = matches.is_present("karyotype");
    let gs = matches.is_present("genome-size");
    let all = matches.is_present("all");
    let print_url = matches.is_present("url");
    let tax_tree_bool = matches.is_present("tree");
    let busco = matches.is_present("busco");

    let size = matches.value_of("size").unwrap();

    let tax_name_op = matches.value_of("tax-id");
    let filename_op = matches.value_of("file");

    // tree includes all descendents of a node
    let tax_tree = match tax_tree_bool {
        true => "tree",
        false => "name",
    };

    // some GoaT defaults.
    let result = "taxon";
    let summarise_values_by = "count";
    let include_estimates = true;

    let url_vector: Vec<String>;
    // if -t use this
    match tax_name_op {
        Some(s) => url_vector = parse_multiple_taxids(s),
        None => match filename_op {
            Some(s) => url_vector = lines_from_file(s)?,
            None => bail!("[-]\tOne of -f (--file) or -t (--tax-id) should be specified."),
        },
    }

    let url_vector_api = make_goat_search_urls(
        url_vector,
        &*GOAT_URL,
        tax_tree,
        include_estimates,
        include_raw_values,
        summarise_values_by,
        result,
        &*TAXONOMY,
        size,
    );

    // so we can make as many concurrent requests
    // as there are taxa
    let url_vector_api_len = url_vector_api.len();

    if print_url {
        for (index, url) in url_vector_api.iter().enumerate() {
            println!("{}.\tGoaT API URL: {}", index, url);
        }
        std::process::exit(0);
    }

    let fetches = futures::stream::iter(url_vector_api.into_iter().map(|path| async move {
        match reqwest::get(&path).await {
            Ok(resp) => match resp.text().await {
                Ok(body) => {
                    // serialise the JSON. No typing.
                    let v: Value = serde_json::from_str(&body)?;

                    match include_raw_values {
                        true => {
                            let mut records = RawRecords::new();
                            records.get_results(&v)?;

                            Ok(CombinedValues {
                                raw: Some(records),
                                agg: None,
                            })
                        }
                        false => {
                            let mut records = Records::new();
                            records.get_results(&v)?;
                            // records.
                            Ok(CombinedValues {
                                raw: None,
                                agg: Some(records),
                            })
                        }
                    }
                }
                Err(_) => bail!("[-]\tERROR reading {}", path),
            },
            Err(_) => bail!("[-]\tERROR downloading {}", path),
        }
    }))
    .buffer_unordered(url_vector_api_len)
    .collect::<Vec<_>>();

    let awaited_fetches = fetches.await;

    match include_raw_values {
        true => print_raw_output(awaited_fetches, all, assembly, gs, cvalues, karyotype)?,
        false => print_agg_output(
            awaited_fetches,
            all,
            assembly,
            gs,
            cvalues,
            karyotype,
            busco,
        )?,
    }
    Ok(())
}
