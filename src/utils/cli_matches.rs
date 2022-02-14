// a module to parse the command line args
use crate::{utils::url, utils::utils};

use anyhow::{bail, Result};
use lazy_static::lazy_static;

// global size limits on pinging the API
lazy_static! {
    pub static ref UPPER_CLI_SIZE_LIMIT: usize = 50000;
    pub static ref UPPER_CLI_FILE_LIMIT: usize = 500;
}

pub fn process_cli_args(
    matches: &clap::ArgMatches,
    api: &str,
) -> Result<(u64, Vec<String>, Vec<String>)> {
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
    // all names
    let names = matches.is_present("names");
    // all target lists data
    let target_lists = matches.is_present("target-lists");
    // scaffold + contig n50
    let n50 = matches.is_present("n50");
    // bioproject & sample ID
    let bioproject = matches.is_present("bioproject");
    // tidy data.
    // must be true if raw values included
    let tidy = match include_raw_values {
        true => true,
        false => matches.is_present("tidy"),
    };
    let gene_count = matches.is_present("gene-count");
    let date = matches.is_present("date");
    let country_list = matches.is_present("country-list");
    // including estimates
    let include_estimates = matches.is_present("include-estimates");
    // status
    let status = matches.is_present("status");
    // expression
    let expression = match matches.value_of("expression") {
        Some(s) => url::format_expression(s)?,
        None => "".to_string(),
    };
    // print expression table
    let print_expression = matches.is_present("print-expression");
    let variable_string = matches.value_of("variables");

    if print_expression {
        crate::utils::expression::print_variable_data();
        std::process::exit(0);
    }

    // merge the field flags
    let fields = url::FieldBuilder {
        all,
        assembly,
        bioproject,
        busco,
        country_list,
        cvalues,
        date,
        gene_count,
        gs,
        karyotype,
        legislation,
        mitochondrion,
        names,
        n50,
        plastid,
        ploidy,
        sex_determination,
        status,
        target_lists,
        tidy,
    };

    // do some size checking
    let size = match matches.value_of("size") {
        Some(s) => s,
        None => "0",
    };

    let size_int: u64;
    match size.parse::<u64>() {
        Ok(e) => {
            size_int = e;
            if e as usize > *UPPER_CLI_SIZE_LIMIT {
                let limit_string = utils::pretty_print_usize(*UPPER_CLI_SIZE_LIMIT);
                bail!(
                    "Searches with more than {} results are not currently supported.",
                    limit_string
                )
            }
        }
        Err(e) => bail!("Did you pass an integer? {}", e),
    }
    let ranks = match matches.value_of("ranks") {
        Some(r) => r,
        // only to stop progress panicking on newick.
        None => "",
    };

    // tree includes all descendents of a node
    let tax_tree = match tax_tree_bool {
        true => "tree",
        false => "name",
    };

    // some GoaT defaults. https://goat.genomehubs.org/search?query=tax_name%28Drosophila%29&result=taxon&fields=all&includeEstimates=true&summaryValues=count&taxonomy=ncbi#tax_name(Drosophila)
    let result = "taxon";
    let summarise_values_by = "count";

    // re-implement this
    let tax_name_op = matches.value_of("taxon");
    let filename_op = matches.value_of("file");

    let url_vector: Vec<String>;
    // if -t use this
    match tax_name_op {
        Some(s) => {
            // catch empty string hanging here.
            if s == "" {
                bail!("[-]\tEmpty string found, please specify a taxon.")
            }
            url_vector = utils::parse_comma_separated(s)
        }
        None => match filename_op {
            Some(s) => {
                url_vector = utils::lines_from_file(s)?;
                // check length of vector and bail if > 1000
                if url_vector.len() > *UPPER_CLI_FILE_LIMIT {
                    let limit_string = utils::pretty_print_usize(*UPPER_CLI_FILE_LIMIT);
                    bail!(
                        "[-]\tNumber of taxa specified cannot exceed {}.",
                        limit_string
                    )
                }
            }
            None => bail!("[-]\tOne of -f (--file) or -t (--tax-id) should be specified."),
        },
    }

    let url_vector_api = url::make_goat_urls(
        api,
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
        variable_string,
        &expression,
    )?;

    if print_url {
        for (index, url) in url_vector_api.iter().enumerate() {
            println!("{}.\tGoaT API URL: {}", index, url);
        }
        std::process::exit(0);
    }

    // return the url vector
    Ok((size_int, url_vector, url_vector_api))
}
