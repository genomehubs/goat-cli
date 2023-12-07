use crate::utils::{
    expression, tax_ranks, url, utils,
    variable_data::{GOAT_ASSEMBLY_VARIABLE_DATA, GOAT_TAXON_VARIABLE_DATA},
};
use crate::{IndexType, TaxType, GOAT_URL, TAXONOMY, UPPER_CLI_FILE_LIMIT, UPPER_CLI_SIZE_LIMIT};
use anyhow::{bail, Result};
use std::path::PathBuf;

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
    let print_url = *matches.get_one::<bool>("url").expect("cli defaults false");
    let print_goat_ui_url = *matches
        .get_one::<bool>("goat-ui-url")
        .expect("cli defaults false");
    let tax_tree_enum = match *matches
        .get_one::<bool>("descendents")
        .expect("cli defaults false")
    {
        true => TaxType::Tree,
        false => TaxType::Name,
    };
    // I think lineage is of limited value for assembly? but keep here anyways
    let tax_lineage_enum = match *matches.get_one::<bool>("lineage").unwrap_or(&false) {
        true => TaxType::Lineage,
        false => TaxType::Name,
    };
    let include_estimates = *matches
        .get_one::<bool>("include-estimates")
        .expect("cli defaults false");
    let expression = match matches.get_one::<String>("expression") {
        Some(s) => url::format_expression(s, index_type)?,
        None => "".to_string(),
    };
    // map needed to convert Option<String> -> Option<&str>
    let variable_string = matches.get_one::<String>("variables").map(|x| &**x);
    // this output will differ depending on taxon/assembly
    // but keep cli arg the same
    let print_expression = *matches
        .get_one::<bool>("print-expression")
        .expect("cli defaults false");

    let tax_rank = match matches.get_one::<String>("tax-rank") {
        Some(t) => tax_ranks::TaxRanks::init().parse(t, false)?,
        None => "".to_string(),
    };
    let size = *matches.get_one::<u64>("size").expect("cli default = 50");
    let ranks = matches
        .get_one::<String>("ranks")
        .expect("cli default = none");
    let tax_name_op = matches.get_one::<String>("taxon");
    let filename_op = matches.get_one::<PathBuf>("file");
    let result = index_type.to_string();
    let summarise_values_by = "count";
    // add in exclusion of missing and ancestral values by default, but allow the user
    // to toggle this on the command line
    let exclude = *matches.get_one::<bool>("exclude").unwrap_or(&false);

    // command line args unique to taxon
    let taxon_include_raw_values = *matches.get_one::<bool>("taxon-raw").unwrap_or(&false);
    let taxon_tidy = match taxon_include_raw_values {
        true => true,
        false => *matches.get_one::<bool>("taxon-tidy").unwrap_or(&false),
    };
    let taxon_assembly = *matches.get_one::<bool>("taxon-assembly").unwrap_or(&false);
    let taxon_cvalues = *matches.get_one::<bool>("taxon-c-values").unwrap_or(&false);
    let taxon_karyotype = *matches.get_one::<bool>("taxon-karyotype").unwrap_or(&false);
    let taxon_gs = *matches
        .get_one::<bool>("taxon-genome-size")
        .unwrap_or(&false);
    let taxon_busco = *matches.get_one::<bool>("taxon-busco").unwrap_or(&false);
    let taxon_gc_percent = *matches
        .get_one::<bool>("taxon-gc-percent")
        .unwrap_or(&false);
    let taxon_mitochondrion = *matches
        .get_one::<bool>("taxon-mitochondria")
        .unwrap_or(&false);
    let taxon_plastid = *matches.get_one::<bool>("taxon-plastid").unwrap_or(&false);
    let taxon_ploidy = *matches.get_one::<bool>("taxon-ploidy").unwrap_or(&false);
    let taxon_sex_determination = *matches
        .get_one::<bool>("taxon-sex-determination")
        .unwrap_or(&false);
    let taxon_legislation = *matches
        .get_one::<bool>("taxon-legislation")
        .unwrap_or(&false);
    let taxon_names = *matches.get_one::<bool>("taxon-names").unwrap_or(&false);
    let taxon_target_lists = *matches
        .get_one::<bool>("taxon-target-lists")
        .unwrap_or(&false);
    let taxon_n50 = *matches.get_one::<bool>("taxon-n50").unwrap_or(&false);
    let taxon_bioproject = *matches
        .get_one::<bool>("taxon-bioproject")
        .unwrap_or(&false);
    let taxon_gene_count = *matches
        .get_one::<bool>("taxon-gene-count")
        .unwrap_or(&false);
    let taxon_date = *matches.get_one::<bool>("taxon-date").unwrap_or(&false);
    let taxon_country_list = *matches
        .get_one::<bool>("taxon-country-list")
        .unwrap_or(&false);
    let taxon_status = *matches.get_one::<bool>("taxon-status").unwrap_or(&false);
    let taxon_toggle_direct = *matches.get_one::<bool>("toggle-direct").unwrap_or(&false);

    // command line args unique to assembly
    let assembly_assembly = *matches
        .get_one::<bool>("assembly-assembly")
        .unwrap_or(&false);
    let assembly_karyotype = *matches
        .get_one::<bool>("assembly-karyotype")
        .unwrap_or(&false);
    let assembly_contig = *matches.get_one::<bool>("assembly-contig").unwrap_or(&false);
    let assembly_scaffold = *matches
        .get_one::<bool>("assembly-scaffold")
        .unwrap_or(&false);
    let assembly_gc = *matches.get_one::<bool>("assembly-gc").unwrap_or(&false);
    let assembly_gene = *matches
        .get_one::<bool>("assembly-gene-count")
        .unwrap_or(&false);
    let assembly_busco = *matches.get_one::<bool>("assembly-busco").unwrap_or(&false);
    let assembly_btk = *matches.get_one::<bool>("assembly-btk").unwrap_or(&false);

    if print_expression {
        match index_type {
            IndexType::Taxon => expression::print_variable_data(&GOAT_TAXON_VARIABLE_DATA),
            IndexType::Assembly => expression::print_variable_data(&GOAT_ASSEMBLY_VARIABLE_DATA),
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
        taxon_toggle_direct,
        assembly_assembly,
        assembly_karyotype,
        assembly_contig,
        assembly_scaffold,
        assembly_gc,
        assembly_gene,
        assembly_busco,
        assembly_btk,
    };

    if size as usize > *UPPER_CLI_SIZE_LIMIT {
        let limit_string = utils::pretty_print_usize(*UPPER_CLI_SIZE_LIMIT);
        bail!(
            "Searches with more than {} results are not currently supported.",
            limit_string
        )
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
            if s.is_empty() {
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
        exclude,
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
    Ok((size, url_vector, url_vector_api))
}
