use crate::utils::url::{GOAT_URL, TAXONOMY};
use crate::utils::utils::{lines_from_file, make_goat_search_urls, parse_multiple_taxids};
use crate::{
    search::agg_values::{get_results, Records},
    search::output::*,
    search::raw_values::{
        get_raw_assembly, AggRawFetches, RawAssembly, RawCValues, RawChromosomeNumbers, RawGSs,
        RawHaploidNumbers,
    },
};

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
            None => bail!("[-]\tOne of -f or -t should be specified."),
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
                            // might segregate this code out later to make more readable.
                            // get taxon name and tax-id for all future printing.
                            // TODO: should really get this from inside the json more
                            let taxon_name = &v["results"][0]["result"]["scientific_name"]
                                .as_str()
                                .unwrap_or("No taxon name found.");
                            let taxon_ncbi = &v["results"][0]["result"]["taxon_id"]
                                .as_str()
                                .unwrap_or("No taxon ID found.");

                            let mut raw_assembly = RawAssembly::new();
                            get_raw_assembly(&v, &mut raw_assembly, "assembly_level")?;
                            get_raw_assembly(&v, &mut raw_assembly, "assembly_span")?;
                            // merge assembly records and print
                            let merged = raw_assembly.merge(taxon_name, taxon_ncbi);
                            // now for the rest of the traits
                            // add taxon name and ID here too.
                            let mut c_values = RawCValues::new();
                            c_values.populate(&v, taxon_name, taxon_ncbi);
                            let mut chrom_nums = RawChromosomeNumbers::new();
                            chrom_nums.populate(&v, taxon_name, taxon_ncbi);
                            let mut genome_sizes = RawGSs::new();
                            genome_sizes.populate(&v, taxon_name, taxon_ncbi);
                            let mut haploid_numbers = RawHaploidNumbers::new();
                            haploid_numbers.populate(&v, taxon_name, taxon_ncbi);

                            Ok(CombinedValues {
                                raw: Some(AggRawFetches {
                                    combined_raw: merged,
                                    c_values: c_values,
                                    chrom_nums: chrom_nums,
                                    genome_sizes: genome_sizes,
                                    haploid: haploid_numbers,
                                }),
                                agg: None,
                            })
                        }
                        false => {
                            let mut records = Records::new();
                            get_results(&v, &mut records)?;
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
