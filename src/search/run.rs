use crate::{
    search::agg_values::Records, search::combine_output::CombinedValues, search::output_agg,
    search::output_raw, search::raw_values::RawRecords, utils::ranks::RankHeaders, utils::url,
    utils::utils,
};

use anyhow::{bail, Result};
use futures::StreamExt;
use reqwest;
use serde_json::Value;

// tax tree check number of hits. If less than, give a warning.
// give info on taxa not found

pub async fn search<'a>(matches: &clap::ArgMatches<'a>) -> Result<()> {
    let include_raw_values = matches.is_present("raw");
    let assembly = matches.is_present("assembly");
    let cvalues = matches.is_present("c-values");
    let karyotype = matches.is_present("karyotype");
    let gs = matches.is_present("genome-size");
    let all = matches.is_present("all");
    let print_url = matches.is_present("url");
    let tax_tree_bool = matches.is_present("phylogeny");
    let busco = matches.is_present("busco");
    // non-default fields.
    let mitochondrion = matches.is_present("mitochondria");
    let plastid = matches.is_present("plastid");

    // merge the field flags
    let fields = url::FieldBuilder {
        all,
        assembly,
        busco,
        cvalues,
        gs,
        karyotype,
        mitochondrion,
        plastid,
    };

    let size = matches.value_of("size").unwrap();
    match size.parse::<usize>() {
        Ok(e) => {
            if e > 10000 {
                bail!("Searches with more than 10,000 results are not currently supported.")
            }
        }
        Err(e) => bail!("Did you pass an integer? {}", e),
    }

    let tax_name_op = matches.value_of("taxon");
    let filename_op = matches.value_of("file");
    let ranks = matches.value_of("ranks").unwrap(); // safe to unwrap here.

    // and let's get out the vector of ranks immediately
    let ranks_vec = utils::get_rank_vector(ranks);

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
        Some(s) => url_vector = utils::parse_multiple_taxids(s),
        None => match filename_op {
            Some(s) => url_vector = utils::lines_from_file(s)?,
            None => bail!("[-]\tOne of -f (--file) or -t (--tax-id) should be specified."),
        },
    }

    let url_vector_api = url::make_goat_search_urls(
        url_vector,
        &*url::GOAT_URL,
        tax_tree,
        include_estimates,
        include_raw_values,
        summarise_values_by,
        result,
        &*url::TAXONOMY,
        size,
        ranks,
        fields,
    );

    // so we can make as many concurrent requests
    // as there are taxa
    // I've tested this up to 20,000 with no problems.
    let concurrent_requests = url_vector_api.len();

    if print_url {
        for (index, url) in url_vector_api.iter().enumerate() {
            println!("{}.\tGoaT API URL: {}", index, url);
        }
        std::process::exit(0);
    }

    let fetches = futures::stream::iter(url_vector_api.into_iter().map(|path| async move {
        // possibly make a again::RetryPolicy
        // to catch all the values in a *very* large request.
        match again::retry(|| reqwest::get(&path)).await {
            Ok(resp) => match resp.text().await {
                Ok(body) => {
                    // serialise the JSON. No typing.
                    let v: Value = serde_json::from_str(&body)?;
                    utils::check_number_hits(&v, size)?;
                    // and let's get out the vector of ranks immediately
                    let ranks_vec = utils::get_rank_vector(ranks);

                    match include_raw_values {
                        true => {
                            let mut records = RawRecords::new();
                            records.get_results(&v, &ranks_vec)?;

                            Ok(CombinedValues {
                                raw: Some(records),
                                agg: None,
                            })
                        }
                        false => {
                            let mut records = Records::new();
                            records.get_results(&v, &ranks_vec)?;

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
    .buffer_unordered(concurrent_requests)
    .collect::<Vec<_>>();

    let awaited_fetches = fetches.await;

    match include_raw_values {
        true => {
            output_raw::print_raw_output(awaited_fetches, fields.clone(), RankHeaders(ranks_vec))?
        }
        false => {
            output_agg::print_agg_output(awaited_fetches, fields.clone(), RankHeaders(ranks_vec))?
        }
    }
    Ok(())
}
