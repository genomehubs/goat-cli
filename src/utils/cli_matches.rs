use crate::utils::{
    expression, tax_ranks, url, utils,
    variable_data::{GOAT_ASSEMBLY_VARIABLE_DATA, GOAT_TAXON_VARIABLE_DATA},
};
use crate::{IndexType, TaxType, GOAT_URL, TAXONOMY, UPPER_CLI_FILE_LIMIT, UPPER_CLI_SIZE_LIMIT};
use anyhow::{bail, Result};

/// Take CLI arguments and parse them. Return a tuple of:
///
/// (the size arg you passed, vector of taxon ID's, vector of URLs, and a vector
/// of unique ID's).
pub fn process_cli_args(
    matches: &clap::ArgMatches,
    api: &str,
    unique_ids: Vec<String>,
    index_type: IndexType,
) -> Result<(u64, Vec<String>, Vec<String>)> {
    // command line args same between taxon/assembly
    let print_url = matches.is_present("url");
    let print_goat_ui_url = matches.is_present("goat-ui-url");
    let tax_tree_enum = match matches.is_present("descendents") {
        true => TaxType::Tree,
        false => TaxType::Name,
    };
    // I think lineage is of limited value for assembly? but keep here anyways
    let tax_lineage_enum = match matches.is_present("lineage") {
        true => TaxType::Lineage,
        false => TaxType::Name,
    };
    let include_estimates = matches.is_present("include-estimates");
    let expression = match matches.value_of("expression") {
        Some(s) => url::format_expression(s, index_type)?,
        None => "".to_string(),
    };
    let variable_string = matches.value_of("variables");
    // this output will differ depending on taxon/assembly
    // but keep cli arg the same
    let print_expression = matches.is_present("print-expression");

    let tax_rank = match matches.value_of("tax-rank") {
        Some(t) => tax_ranks::TaxRanks::init().parse(t, false)?,
        None => "".to_string(),
    };
    let size = match matches.value_of("size") {
        Some(s) => s,
        None => "0",
    };
    let ranks = match matches.value_of("ranks") {
        Some(r) => r,
        // only to stop progress panicking on newick.
        None => "",
    };
    let tax_name_op = matches.value_of("taxon");
    let filename_op = matches.value_of("file");
    let result = index_type.to_string();
    let summarise_values_by = "count";

    // command line args unique to taxon

    let taxon_include_raw_values = matches.is_present("taxon-raw");
    let taxon_tidy = match taxon_include_raw_values {
        true => true,
        false => matches.is_present("taxon-tidy"),
    };
    let taxon_assembly = matches.is_present("taxon-assembly");
    let taxon_cvalues = matches.is_present("taxon-c-values");
    let taxon_karyotype = matches.is_present("taxon-karyotype");
    let taxon_gs = matches.is_present("taxon-genome-size");
    let taxon_busco = matches.is_present("taxon-busco");
    let taxon_gc_percent = matches.is_present("taxon-gc-percent");
    let taxon_mitochondrion = matches.is_present("taxon-mitochondria");
    let taxon_plastid = matches.is_present("taxon-plastid");
    let taxon_ploidy = matches.is_present("taxon-ploidy");
    let taxon_sex_determination = matches.is_present("taxon-sex-determination");
    let taxon_legislation = matches.is_present("taxon-legislation");
    let taxon_names = matches.is_present("taxon-names");
    let taxon_target_lists = matches.is_present("taxon-target-lists");
    let taxon_n50 = matches.is_present("taxon-n50");
    let taxon_bioproject = matches.is_present("taxon-bioproject");
    let taxon_gene_count = matches.is_present("taxon-gene-count");
    let taxon_date = matches.is_present("taxon-date");
    let taxon_country_list = matches.is_present("taxon-country-list");
    let taxon_status = matches.is_present("taxon-status");

    // command line args unique to assembly
    let assembly_assembly = matches.is_present("assembly-assembly");
    let assembly_karyotype = matches.is_present("assembly-karyotype");
    let assembly_contig = matches.is_present("assembly-contig");
    let assembly_scaffold = matches.is_present("assembly-scaffold");
    let assembly_gc = matches.is_present("assembly-gc");
    let assembly_gene = matches.is_present("assembly-gene-count");
    let assembly_busco = matches.is_present("assembly-busco");
    let assembly_btk = matches.is_present("assembly-btk");

    if print_expression {
        match index_type {
            IndexType::Taxon => expression::print_variable_data(&*GOAT_TAXON_VARIABLE_DATA),
            IndexType::Assembly => expression::print_variable_data(&*GOAT_ASSEMBLY_VARIABLE_DATA),
        }
        std::process::exit(0);
    }

    // merge the field flags
    let fields = url::FieldBuilder {
        taxon_assembly,
        taxon_bioproject,
        taxon_busco,
        taxon_country_list,
        taxon_cvalues,
        taxon_date,
        taxon_gc_percent,
        taxon_gene_count,
        taxon_gs,
        taxon_karyotype,
        taxon_legislation,
        taxon_mitochondrion,
        taxon_names,
        taxon_n50,
        taxon_plastid,
        taxon_ploidy,
        taxon_sex_determination,
        taxon_status,
        taxon_target_lists,
        taxon_tidy,
        assembly_assembly,
        assembly_karyotype,
        assembly_contig,
        assembly_scaffold,
        assembly_gc,
        assembly_gene,
        assembly_busco,
        assembly_btk,
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
        Err(e) => bail!("Did you pass an integer to `--size`? Info: {}", e),
    }

    // tree includes all descendents of a node
    let tax_tree = match (tax_tree_enum, tax_lineage_enum) {
        (TaxType::Tree, TaxType::Name) => "tree",
        (TaxType::Name, TaxType::Lineage) => "lineage",
        (TaxType::Name, TaxType::Name) => "name",
        (_, _) => bail!("If we get here, I've done something wrong in the `TaxType` enum logic. Please file an issue."),
    };

    let url_vector: Vec<String>;
    // if -t use this
    match tax_name_op {
        Some(s) => {
            // catch empty string hanging here.
            if s == "" {
                bail!("Empty string found, please specify a taxon.")
            }
            url_vector = utils::parse_comma_separated(s)
        }
        None => match filename_op {
            Some(s) => {
                url_vector = utils::lines_from_file(s)?;
                // check length of vector and bail if > 1000
                if url_vector.len() > *UPPER_CLI_FILE_LIMIT {
                    let limit_string = utils::pretty_print_usize(*UPPER_CLI_FILE_LIMIT);
                    bail!("Number of taxa specified cannot exceed {}.", limit_string)
                }
            }
            None => bail!("One of -f (--file) or -t (--taxon) should be specified."),
        },
    }

    let url_vector_api = url::make_goat_urls(
        api,
        &url_vector,
        &*GOAT_URL,
        tax_tree,
        include_estimates,
        // check again whether to include
        // raw values in `assembly` index.
        taxon_include_raw_values,
        summarise_values_by,
        &result,
        &*TAXONOMY,
        size,
        ranks,
        fields,
        variable_string,
        &expression,
        &tax_rank,
        unique_ids,
        index_type,
    )?;

    if print_url {
        for (index, url) in url_vector_api.iter().enumerate() {
            println!("{}.\tGoaT API URL: {}", index, url);
        }
        std::process::exit(0);
    } else if print_goat_ui_url {
        for (index, url) in url_vector_api.iter().enumerate() {
            // remove api/v2/
            let new_url = url.replace("api/v2/", "");
            println!("{}.\tGoaT API URL: {}", index, new_url);
        }
        std::process::exit(0);
    }

    // return the url vector
    Ok((size_int, url_vector, url_vector_api))
}
