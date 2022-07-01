// Max Brown
// Wellcome Sanger Institute: 2022

use anyhow::{Result};
use clap::{crate_version, AppSettings, Arg, Command};
use futures::try_join;
use tokio;

use goat_cli::{
    count, lookup, progress,
    report::newick,
    search,
    utils::utils::{generate_unique_strings, pretty_print_usize},
    IndexType,
    UPPER_CLI_FILE_LIMIT, UPPER_CLI_SIZE_LIMIT,
};

#[tokio::main]
async fn main() -> Result<()> {
    // global helps for both assembly/taxon subcommands.
    let upper_file_limit = pretty_print_usize(*UPPER_CLI_FILE_LIMIT);
    let upper_cli_limit = pretty_print_usize(*UPPER_CLI_SIZE_LIMIT);
    let taxon_file_or_lookup_help = &format!("A file of NCBI taxonomy ID's (tips) and/or binomial names.\nEach line should contain a single entry.\nFile size is limited to {} entries.", upper_file_limit)[..];
    let taxon_size_help = &format!("The number of results to return. Max {} currently.", upper_cli_limit)[..];

    let taxon_search_and_count = |name, about| {
        Command::new(name) 
            .about(about)
            .arg(
                Arg::new("taxon")
                    .short('t')
                    .long("taxon")
                    .takes_value(true)
                    .required_unless_present_any(["file", "print-expression", "variables"])
                    .help("The taxon to search. An NCBI taxon ID, or the name of a taxon at any rank."),
            )
            .arg(
                Arg::new("file")
                    .short('f')
                    .long("file")
                    .takes_value(true)
                    .required_unless_present_any(["taxon", "print-expression", "variables"])
                    .help(taxon_file_or_lookup_help),
            )
            .arg(
                Arg::new("variables")
                    .short('v')
                    .long("variables")
                    .takes_value(true)
                    .required_unless_present_any(["file", "print-expression", "taxon"])
                    .help("Variable parser. Input a comma separated string of variables.")
            )
            .arg(
                Arg::new("size")
                    .long("size")
                    .default_value("50")
                    .help(taxon_size_help),
            )
            .arg(
                Arg::new("ranks")
                    .short('R')
                    .long("ranks")
                    .possible_values(&["none", "subspecies", "species", "genus", "family", "order", "class", "phylum", "kingdom", "superkingdom"])
                    .default_value("none")
                    .help("Choose a rank to display with the results. All ranks up to the given rank are displayed.")
            )
            .arg(
                Arg::new("expression")
                    .short('e')
                    .long("expression")
                    .takes_value(true)
                    .required(false)
                    .help("Use an expression to filter results server-side.")
            )
            .arg(
                Arg::new("tax-rank")
                    .long("tax-rank")
                    .takes_value(true)
                    .required(false)
                    .help("The taxonomic rank to return the results at.")
            )
            // display level 1
            .arg(
                Arg::new("taxon-assembly")
                    .short('a')
                    .long("assembly")
                    .help("Print assembly data (assembly span, assembly level)"),
            )
            .arg(
                Arg::new("taxon-busco")
                    .short('b')
                    .long("busco")
                    .help("Print BUSCO estimates."),
            )
            .arg(
                Arg::new("taxon-gc-percent")
                    .short('g') 
                    .long("gc-percent")
                    .help("Print GC%.")
            )
            .arg(
                Arg::new("taxon-karyotype")
                    .short('k')
                    .long("karyotype")
                    .help("Print karyotype data (chromosome number & haploid number)."),
            )
            .arg(
                Arg::new("taxon-genome-size")
                    .short('G')
                    .long("genome-size")
                    .help("Print genome size data."),
            )
            // further display levels
            .arg(
                Arg::new("taxon-bioproject")
                    .short('B')
                    .long("bioproject")
                    .help("Print the bioproject and biosample ID of records.")
            )
            .arg(
                Arg::new("taxon-n50")
                    .short('N')
                    .long("n50")
                    .help("Print the contig & scaffold n50 of assemblies.")
            )
            .arg(
                Arg::new("taxon-date")
                    .short('D')
                    .long("date")
                    .help("Print EBP & assembly dates.")
            )
            .arg(
                Arg::new("taxon-gene-count")
                    .long("gene-count")
                    .help("Print gene count data.")
            )
            .arg(
                Arg::new("taxon-mitochondria")
                    .short('m')
                    .long("mitochondria")
                    .help("Print mitochondrial genome assembly size & GC%.")
            )
            .arg(
                Arg::new("taxon-plastid")
                    .short('p')
                    .long("plastid")
                    .help("Print plastid genome assembly size & GC%.")
            )
            .arg(
                Arg::new("taxon-sex-determination")
                    .short('S')
                    .long("sex-determination")
                    .help("Print sex determination data."),
            )
            .arg(
                Arg::new("taxon-ploidy")
                    .short('P')
                    .long("ploidy")
                    .help("Print ploidy estimates.")
            )
            .arg(
                Arg::new("taxon-c-values")
                    .short('c')
                    .long("c-values")
                    .help("Print c-value data."),
            )
            .arg(
                Arg::new("taxon-legislation")
                    .long("legislation")
                    .help("Print legislation data."),
            )
            .arg(
                Arg::new("lineage")
                    .short('l')
                    .long("lineage")
                    .conflicts_with("descendents")
                    .help("Displays lineage information. I.e. from this node in the tree go back and give all the nodes to the root. Conflicts with descendents."),
            )
            .arg(
                Arg::new("taxon-target-lists")
                    .long("target-lists")
                    .help("Print target list data associated with each taxon.")
            )
            .arg(
                Arg::new("taxon-country-list")
                    .short('C')
                    .long("country-list")
                    // what's the best description for this?
                    .help("Print list of countries where taxon is found.")
            )
            .arg(
                Arg::new("taxon-status")
                    .long("status")
                    .help("Print all data associated with how far this taxon has progressed with genomic sequencing.\nThis includes sample collection, acquisition, progress in sequencing, and whether submitted to INSDC.")
            )
            .arg(
                Arg::new("taxon-names")
                    .short('n')
                    .long("names")
                    .help("Print all associated name data (synonyms, Tree of Life ID, and common names)."),
            )
            .arg(
                Arg::new("taxon-raw")
                    .short('r')
                    .long("raw")
                    .help("Print raw values (i.e. no aggregation/summary)."),
            )
            .arg(
                Arg::new("descendents")
                    .short('d')
                    .long("descendents")
                    .help("Get information for all descendents of a common ancestor."),
            )
            .arg(
                Arg::new("taxon-tidy")
                    .long("tidy")
                    .short('T')
                    .help("Print data in tidy format.")
            )
            .arg(
                Arg::new("include-estimates")
                    .short('i')
                    .long("include-estimates")
                    .conflicts_with("raw")
                    .help("Include ancestral estimates. Omitting this flag includes only direct estimates from a taxon. Cannot be used with --raw.")
            )
            .arg(
                Arg::new("print-expression")
                    .long("print-expression")
                    .help("Print all variables in GoaT currently, with their associated variants.\nUseful for construction of expressions.")
            )
            .arg(
                Arg::new("progress-bar")
                    .long("progress-bar")
                    .help("Add a progress bar to large queries, to estimate time left.")
            )
            .arg(
                Arg::new("url")
                    .short('u')
                    .long("url")
                    .help("Print the underlying GoaT API URL(s). Useful for debugging."),
            )
        };

    let assembly_search_and_count = |name, about| {
        Command::new(name) 
            .about(about)
            .arg(
                Arg::new("taxon")
                    .short('t')
                    .long("taxon")
                    .takes_value(true)
                    .required_unless_present_any(["file", "print-expression", "variables"])
                    .help("The taxon to search. An NCBI taxon ID, or the name of a taxon at any rank."),
            )
            .arg(
                Arg::new("file")
                    .short('f')
                    .long("file")
                    .takes_value(true)
                    .required_unless_present_any(["taxon", "print-expression", "variables"])
                    .help(taxon_file_or_lookup_help),
            )
            .arg(
                // will require a new database
                Arg::new("variables")
                    .short('v')
                    .long("variables")
                    .takes_value(true)
                    .required_unless_present_any(["file", "print-expression", "taxon"])
                    .help("Variable parser. Input a comma separated string of variables.")
            )
            .arg(
                Arg::new("size")
                    .long("size")
                    .default_value("50")
                    .help(taxon_size_help),
            )
            .arg(
                Arg::new("ranks")
                    .short('R')
                    .long("ranks")
                    .possible_values(&["none", "subspecies", "species", "genus", "family", "order", "class", "phylum", "kingdom", "superkingdom"])
                    .default_value("none")
                    .help("Choose a rank to display with the results. All ranks up to the given rank are displayed.")
            )
            // will also need another database.
            .arg(
                Arg::new("expression")
                    .short('e')
                    .long("expression")
                    .takes_value(true)
                    .required(false)
                    .help("Use an expression to filter results server-side.")
            )
            .arg(
                Arg::new("tax-rank")
                    .long("tax-rank")
                    .takes_value(true)
                    .required(false)
                    .help("The taxonomic rank to return the results at.")
            )
            .arg(
                Arg::new("lineage")
                    .short('l')
                    .long("lineage")
                    .conflicts_with("descendents")
                    .help("Displays lineage information. I.e. from this node in the tree go back and give all the nodes to the root. Conflicts with descendents."),
            )
            .arg(
                Arg::new("assembly-assembly")
                    .short('a')
                    .long("assembly")
                    .help("Print assembly data (span & level)"),
            )
            .arg(
                Arg::new("assembly-karyotype")
                    .short('k')
                    .long("karyotype")
                    .help("Print karyotype data (chromosome number only)."),
            )
            .arg(
                Arg::new("assembly-contig")
                    .short('c')
                    .long("contig")
                    .help("Print contig data (count, l50, n50)."),
            )
            .arg(
                Arg::new("assembly-scaffold")
                    .short('s')
                    .long("scaffold")
                    .help("Print scaffold data (count, l50, n50)."),
            )
            .arg(
                Arg::new("assembly-gene-count")
                    .short('g')
                    .long("gene-count")
                    .help("Print gene count data (gene count, non-coding gene count)."),
            )
            .arg(
                Arg::new("assembly-busco")
                    .short('b')
                    .long("busco")
                    .help("Print BUSCO data (BUSCO completeness, lineage, and string)."),
            )
            .arg(
                Arg::new("assembly-btk")
                    .long("btk")
                    .help("Print BlobToolKit data (no-hit, target)."),
            )
            .arg(
                Arg::new("descendents")
                    .short('d')
                    .long("descendents")
                    .help("Get information for all descendents of a common ancestor."),
            )
            .arg(
                Arg::new("include-estimates")
                    .short('i')
                    .long("include-estimates")
                    .conflicts_with("raw")
                    .help("Include ancestral estimates. Omitting this flag includes only direct estimates from a taxon. Cannot be used with --raw.")
            )
            .arg(
                Arg::new("print-expression")
                    .long("print-expression")
                    .help("Print all variables in GoaT currently, with their associated variants.\nUseful for construction of expressions.")
            )
            .arg(
                Arg::new("progress-bar")
                    .long("progress-bar")
                    .help("Add a progress bar to large queries, to estimate time left.")
            )
            .arg(
                Arg::new("url")
                    .short('u')
                    .long("url")
                    .help("Print the underlying GoaT API URL(s). Useful for debugging."),
            )
    };

    // actually parse the matches.
    let matches = Command::new("goat-cli")
        // to fix the binary name in the help messages
        .bin_name("goat-cli")
        .version(crate_version!())
        .propagate_version(true)
        .arg_required_else_help(true)
        .global_setting(AppSettings::DeriveDisplayOrder)
        .author("Max Brown, Richard Challis, Sujai Kumar, Cibele Sotero-Caio <goat@genomehubs.org>")
        .about("Genomes on a Tree. Query metadata across the tree of life.\n\nFor a tutorial on usage, visit: https://github.com/genomehubs/goat-cli/wiki\nVisit the GoaT website here: https://goat.genomehubs.org/")
        // using a taxon index
        .subcommand(
            Command::new("taxon")
                    .arg_required_else_help(true)
                    .about("Query by taxon index.")
                    .subcommand(
                        taxon_search_and_count("search", "Query metadata for any taxon across the tree of life by taxon index.")
                    )
                    .subcommand(
                        taxon_search_and_count("count", "Return the count of results for any taxon across the tree of life by taxon index.")
                    )
                    .subcommand(
                        Command::new("lookup")
                            .about("Return information relating to a taxon name, e.g. synonyms, authorities.")
                            .arg(
                                Arg::new("taxon")
                                    .short('t')
                                    .long("taxon")
                                    .takes_value(true)
                                    .required_unless_present("file")
                                    .help("The taxon to search. An NCBI taxon ID, or the name of a taxon at any rank."),
                            )
                            .arg(
                                Arg::new("file")
                                    .short('f')
                                    .long("file")
                                    .takes_value(true)
                                    .required_unless_present_any(["taxon"])
                                    .help(taxon_file_or_lookup_help),
                            )
                            .arg(
                                Arg::new("url")
                                    .short('u')
                                    .long("url")
                                    .help("Print lookup URL.")
                            )
                            .arg(
                                Arg::new("size")
                                    .short('s')
                                    .long("size")
                                    .default_value("10")
                                    .help("The number of results to return."),
                            )
                    )
                    .subcommand(
                        Command::new("newick")
                            .about("Generate a newick tree from input taxa.")
                            .arg(
                                Arg::new("taxon")
                                    .short('t')
                                    .long("taxon")
                                    .takes_value(true)
                                    .required_unless_present("file")
                                    .help("The taxon to return a newick of. Multiple taxa will return the joint tree."),
                            )
                            .arg(
                                Arg::new("url")
                                    .short('u')
                                    .long("url")
                                    .help("Print lookup URL.")
                            )
                            .arg(
                                Arg::new("rank")
                                    .short('r')
                                    .long("rank")
                                    .default_value("species")
                                    .possible_values(&["species", "genus", "family", "order"])
                                    .help("The number of results to return."),
                            )
                            .arg(
                                Arg::new("progress-bar")
                                    .long("progress-bar")
                                    .help("Add a progress bar to large queries, to estimate time left.")
                            ),
                    )
            )
        // alright, so what subcommands go here?
        .subcommand(
            Command::new("assembly")
                    .arg_required_else_help(true)
                    .about("Query by assembly index.")
                    .subcommand(
                        assembly_search_and_count("search", "Query metadata for any taxon across the tree of life by assembly index.")
                    )
                    .subcommand(
                        assembly_search_and_count("count", "Return the count of results for any taxon across the tree of life by assembly index.")
                    )
                    .subcommand(
                        Command::new("lookup")
                            .about("Return information relating to a taxon name, e.g. synonyms, authorities.")
                                .arg(
                                    Arg::new("taxon")
                                        .short('t')
                                        .long("taxon")
                                        .takes_value(true)
                                        .required_unless_present("file")
                                        .help("The taxon to search. An NCBI taxon ID, or the name of a taxon at any rank."),
                                )
                                .arg(
                                    Arg::new("file")
                                        .short('f')
                                        .long("file")
                                        .takes_value(true)
                                        .required_unless_present_any(["taxon"])
                                        .help(taxon_file_or_lookup_help),
                                )
                                .arg(
                                    Arg::new("url")
                                        .short('u')
                                        .long("url")
                                        .help("Print lookup URL.")
                                )
                                .arg(
                                    Arg::new("size")
                                        .short('s')
                                        .long("size")
                                        .default_value("10")
                                        .help("The number of results to return."),
                                )
                    )
            )
        .get_matches();

    // nested matching on subcommands
    match matches.subcommand() {
        // outer == taxon/assembly
        Some(("taxon", matches)) => match matches.subcommand() {
            // inner are all the taxon matches here.
            Some(("search", matches)) => {
                let progress_bar = matches.is_present("progress-bar");
                let unique_ids = generate_unique_strings(matches, IndexType::Taxon)?;

                match progress_bar {
                    true => {
                        try_join!(
                            search::search(&matches, unique_ids.clone(), IndexType::Taxon),
                            progress::progress_bar(&matches, "search", unique_ids, IndexType::Taxon)
                        )?;
                    }
                    false => {
                        let _ = search::search(&matches, unique_ids, IndexType::Taxon).await?;
                    }
                }
            }
            Some(("count", matches)) => {
                let unique_ids = generate_unique_strings(matches, IndexType::Taxon)?;
                count::count(&matches, true, false, unique_ids, IndexType::Taxon).await?;
            }
            Some(("lookup", matches)) => {
                lookup::lookup(&matches, true, IndexType::Taxon).await?;
            }
            Some(("newick", matches)) => {
                let progress_bar = matches.is_present("progress-bar");
                let unique_ids = generate_unique_strings(matches, IndexType::Taxon)?;

                match progress_bar {
                    true => {
                        try_join!(
                            newick::get_newick(matches, unique_ids.clone()),
                            progress::progress_bar(&matches, "newick", unique_ids, IndexType::Taxon)
                        )?;
                    }
                    false => newick::get_newick(matches, unique_ids).await?,
                }
            }
            _ => {
                unreachable!()
            }
        },
        // and now assembly
        Some(("assembly", matches)) => match matches.subcommand() {
            // and the three implemented subcommands currently.
            Some(("search", matches)) => {
                let progress_bar = matches.is_present("progress-bar");
                let unique_ids = generate_unique_strings(matches, IndexType::Assembly)?;

                match progress_bar {
                    true => {
                        try_join!(
                            search::search(&matches, unique_ids.clone(), IndexType::Assembly),
                            progress::progress_bar(&matches, "search", unique_ids, IndexType::Assembly)
                        )?;
                    }
                    false => {
                        let _ = search::search(&matches, unique_ids, IndexType::Assembly).await?;
                    }
                }
            }
            Some(("count", matches)) => {
                let unique_ids = generate_unique_strings(matches, IndexType::Assembly)?;
                count::count(&matches, true, false, unique_ids, IndexType::Assembly).await?;
            }
            Some(("lookup", matches)) => {
                lookup::lookup(&matches, true, IndexType::Assembly).await?;
            }
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }

    Ok(())
}
