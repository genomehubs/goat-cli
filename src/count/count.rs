use crate::utils::{url, utils};

use anyhow::{bail, Result};
use futures::StreamExt;
use reqwest;
use reqwest::header::ACCEPT;
use serde_json::Value;

// count is essentially identical to search, but prints to the console in the
// CLI call, or to stderr in the search call. Hence the cli parameter.

pub async fn count<'a>(matches: &clap::ArgMatches<'a>, cli: bool) -> Result<()> {
    // all the same as search

    let include_raw_values = matches.is_present("raw");
    let assembly = matches.is_present("assembly");
    let cvalues = matches.is_present("c-values");
    let karyotype = matches.is_present("karyotype");
    let gs = matches.is_present("genome-size");
    let all = matches.is_present("all");
    let print_url = matches.is_present("url");
    let tax_tree_bool = matches.is_present("descendents");
    let busco = matches.is_present("busco");
    // non-default fields.
    let mitochondrion = matches.is_present("mitochondria");
    let plastid = matches.is_present("plastid");
    let ploidy = matches.is_present("ploidy");
    let sex_determination = matches.is_present("sex-determination");
    // all legislation
    let legislation = matches.is_present("legislation");
    // all names data
    let names = matches.is_present("names");
    // all target lists data
    let target_lists = matches.is_present("target-lists");
    // scaffold + contig n50
    let n50 = matches.is_present("n50");
    // bioproject & sample ID
    let bioproject = matches.is_present("bioproject");
    // tidy data
    let mut tidy = matches.is_present("tidy");
    // and guard against error
    if include_raw_values {
        tidy = true;
    }
    let gene_count = matches.is_present("gene-count");
    let date = matches.is_present("date");
    // including estimates
    let include_estimates = matches.is_present("include-estimates");

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
        ploidy,
        sex_determination,
        legislation,
        names,
        target_lists,
        n50,
        bioproject,
        tidy,
        gene_count,
        date,
    };

    // do some size checking
    let size = matches.value_of("size").unwrap();
    let size_int: u64;
    match size.parse::<u64>() {
        Ok(e) => {
            size_int = e;
            if e > 10000 {
                bail!("Searches with more than 10,000 results are not currently supported.")
            }
        }
        Err(e) => bail!("Did you pass an integer? {}", e),
    }
    let ranks = matches.value_of("ranks").unwrap(); // safe to unwrap here.

    // tree includes all descendents of a node
    let tax_tree = match tax_tree_bool {
        true => "tree",
        false => "name",
    };

    // some GoaT defaults.
    let result = "taxon";
    let summarise_values_by = "max";

    // re-implement this
    let tax_name_op = matches.value_of("taxon");
    let filename_op = matches.value_of("file");

    let url_vector: Vec<String>;
    // if -t use this
    match tax_name_op {
        Some(s) => url_vector = utils::parse_multiple_taxids(s),
        None => match filename_op {
            Some(s) => {
                url_vector = utils::lines_from_file(s)?;
                // check length of vector and bail if > 1000
                if url_vector.len() > 10000 {
                    bail!("[-]\tNumber of taxa specified cannot exceed 10,000.")
                }
            }
            None => bail!("[-]\tOne of -f (--file) or -t (--tax-id) should be specified."),
        },
    }

    let url_vector_api = url::make_goat_urls(
        "count",
        &url_vector,
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

    let fetches = futures::stream::iter(url_vector_api.iter().map(|path| async move {
        // possibly make a again::RetryPolicy
        // to catch all the values in a *very* large request.
        let client = reqwest::Client::new();

        match again::retry(|| client.get(path).header(ACCEPT, "application/json").send()).await {
            Ok(resp) => match resp.text().await {
                Ok(body) => {
                    let v: Value = serde_json::from_str(&body)?;
                    let count = &v["count"].as_u64();
                    match count {
                        Some(c) => Ok(*c),
                        None => bail!("Bad response."),
                    }
                }
                Err(_) => bail!("ERROR reading {}", path),
            },
            Err(_) => bail!("ERROR downloading {}", path),
        }
    }))
    .buffer_unordered(concurrent_requests)
    .collect::<Vec<_>>();

    let awaited_fetches = fetches.await;

    match cli {
        true => {
            // print to console
            println!("search_query\tcount");
            for (el1, el2) in awaited_fetches.iter().zip(url_vector.iter()) {
                let count = match el1 {
                    Ok(e) => e,
                    Err(e) => bail!("{}", e),
                };
                println!("{}\t{}", el2, count)
            }
        }
        false => {
            for (el1, el2) in awaited_fetches.iter().zip(url_vector.iter()) {
                let count = match el1 {
                    Ok(e) => e,
                    Err(e) => bail!("{}", e),
                };
                if size_int < *count {
                    eprintln!("[-]\tFor search query {}, size specified ({}) was less than the number of results returned, ({}).", el2, size_int, count)
                }
            }
        }
    }

    Ok(())
}
