// Max Brown
// Wellcome Sanger Institute: 2022

use anyhow::Result;
use clap::{crate_version, AppSettings, Arg, Command};
// I think I am going to have to generate the unique ID's based on number of
// input queries, then pass these to both the search/newick and then the
// progress bar functions in a try_join?
use futures::try_join;
use tokio;

use goat_cli::{
    count, lookup, progress,
    report::newick,
    search,
    utils::utils::{generate_unique_strings, pretty_print_usize},
    UPPER_CLI_FILE_LIMIT, UPPER_CLI_SIZE_LIMIT,
};

#[tokio::main]
async fn main() -> Result<()> {
    let matches = Command::new("goat")
        // to fix the binary name in the help messages
        .bin_name("goat")
        .version(crate_version!())
        .propagate_version(true)
        .arg_required_else_help(true)
        .global_setting(AppSettings::DeriveDisplayOrder)
        .author("Max Brown, Richard Challis, Sujai Kumar, Cibele Sotero-Caio <goat@genomehubs.org>")
        .about("Genomes on a Tree. Query metadata across the tree of life.\n\nFor a tutorial on usage, visit: https://github.com/genomehubs/goat-cli/wiki\nVisit the GoaT website here: https://goat.genomehubs.org/")
        .subcommand(
            Command::new("search")
                .about("Query metadata for any taxon across the tree of life.")
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
                        .help(&format!("A file of NCBI taxonomy ID's (tips) and/or binomial names.\nEach line should contain a single entry.\nFile size is limited to {} entries.", pretty_print_usize(*UPPER_CLI_FILE_LIMIT))[..]),
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
                        .short('s')
                        .long("size")
                        .default_value("50")
                        .help(&format!("The number of results to return. Max {} currently.", pretty_print_usize(*UPPER_CLI_SIZE_LIMIT))[..]),
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
                    Arg::new("assembly")
                        .short('a')
                        .long("assembly")
                        .help("Print assembly data (span & level)"),
                )
                .arg(
                    Arg::new("busco")
                        .short('b')
                        .long("busco")
                        .help("Print BUSCO estimates."),
                )
                .arg(
                    Arg::new("gc-percent")
                        .short('g') // CHANGE THE OTHER G
                        .long("gc-percent")
                        .help("Print GC%.")
                )
                .arg(
                    Arg::new("karyotype")
                        .short('k')
                        .long("karyotype")
                        .help("Print karyotype data (chromosome number & haploid number)."),
                )
                .arg(
                    Arg::new("genome-size")
                        .short('G')
                        .long("genome-size")
                        .help("Print genome size data."),
                )
                // further display levels
                .arg(
                    Arg::new("bioproject")
                        .short('B')
                        .long("bioproject")
                        .help("Print the bioproject and biosample ID of records.")
                )
                .arg(
                    Arg::new("n50")
                        .short('N')
                        .long("n50")
                        .help("Print the contig & scaffold n50 of assemblies.")
                )
                .arg(
                    Arg::new("date")
                        .short('D')
                        .long("date")
                        .help("Print EBP & assembly dates.")
                )
                .arg(
                    Arg::new("gene-count")
                        .long("gene-count")
                        .help("Print gene count data.")
                )
                .arg(
                    Arg::new("mitochondria")
                        .short('m')
                        .long("mitochondria")
                        .help("Print mitochondrial genome assembly size & GC%.")
                )
                .arg(
                    Arg::new("plastid")
                        .short('p')
                        .long("plastid")
                        .help("Print plastid genome assembly size & GC%.")
                )
                .arg(
                    Arg::new("sex-determination")
                        .short('S')
                        .long("sex-determination")
                        .help("Print sex determination data."),
                )
                .arg(
                    Arg::new("ploidy")
                        .short('P')
                        .long("ploidy")
                        .help("Print ploidy estimates.")
                )
                .arg(
                    Arg::new("c-values")
                        .short('c')
                        .long("c-values")
                        .help("Print c-value data."),
                )
                .arg(
                    Arg::new("legislation")
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
                    Arg::new("target-lists")
                        .long("target-lists")
                        .help("Print target list data associated with each taxon.")
                )
                .arg(
                    Arg::new("country-list")
                        .short('C')
                        .long("country-list")
                        // what's the best description for this?
                        .help("Print list of countries where taxon is found.")
                )
                .arg(
                    Arg::new("status")
                        .long("status")
                        .help("Print all data associated with how far this taxon has progressed with genomic sequencing.\nThis includes sample collection, acquisition, progress in sequencing, and whether submitted to INSDC.")
                )
                .arg(
                    Arg::new("names")
                        .short('n')
                        .long("names")
                        .help("Print all associated name data (synonyms, Tree of Life ID, and common names)."),
                )
                .arg(
                    Arg::new("all")
                        .short('A')
                        .long("all")
                        .help("Print all currently implemented GoaT-CLI variables."),
                )
                .arg(
                    Arg::new("raw")
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
                    Arg::new("tidy")
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
                ),
        )
        // copy of the above.
        .subcommand(
            Command::new("count")
                .about("Return the number of hits from any \"goat search\" query.")
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
                        .help(&format!("A file of NCBI taxonomy ID's (tips) and/or binomial names.\nEach line should contain a single entry.\nFile size is limited to {} entries.", pretty_print_usize(*UPPER_CLI_FILE_LIMIT))[..]),
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
                        .short('s')
                        .long("size")
                        .default_value("50")
                        .help(&format!("The number of results to return. Max {} currently.", pretty_print_usize(*UPPER_CLI_SIZE_LIMIT))[..]),
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
                    Arg::new("assembly")
                        .short('a')
                        .long("assembly")
                        .help("Print assembly data (span & level)"),
                )
                .arg(
                    Arg::new("busco")
                        .short('b')
                        .long("busco")
                        .help("Print BUSCO estimates."),
                )
                .arg(
                    Arg::new("gc-percent")
                        .short('g') // CHANGE THE OTHER G
                        .long("gc-percent")
                        .help("Print GC%.")
                )
                .arg(
                    Arg::new("karyotype")
                        .short('k')
                        .long("karyotype")
                        .help("Print karyotype data (chromosome number & haploid number)."),
                )
                .arg(
                    Arg::new("genome-size")
                        .short('G')
                        .long("genome-size")
                        .help("Print genome size data."),
                )
                // further display levels
                .arg(
                    Arg::new("bioproject")
                        .short('B')
                        .long("bioproject")
                        .help("Print the bioproject and biosample ID of records.")
                )
                .arg(
                    Arg::new("n50")
                        .short('N')
                        .long("n50")
                        .help("Print the contig & scaffold n50 of assemblies.")
                )
                .arg(
                    Arg::new("date")
                        .short('D')
                        .long("date")
                        .help("Print EBP & assembly dates.")
                )
                .arg(
                    Arg::new("gene-count")
                        .long("gene-count")
                        .help("Print gene count data.")
                )
                .arg(
                    Arg::new("mitochondria")
                        .short('m')
                        .long("mitochondria")
                        .help("Print mitochondrial genome assembly size & GC%.")
                )
                .arg(
                    Arg::new("plastid")
                        .short('p')
                        .long("plastid")
                        .help("Print plastid genome assembly size & GC%.")
                )
                .arg(
                    Arg::new("sex-determination")
                        .short('S')
                        .long("sex-determination")
                        .help("Print sex determination data."),
                )
                .arg(
                    Arg::new("ploidy")
                        .short('P')
                        .long("ploidy")
                        .help("Print ploidy estimates.")
                )
                .arg(
                    Arg::new("c-values")
                        .short('c')
                        .long("c-values")
                        .help("Print c-value data."),
                )
                .arg(
                    Arg::new("legislation")
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
                    Arg::new("target-lists")
                        .long("target-lists")
                        .help("Print target list data associated with each taxon.")
                )
                .arg(
                    Arg::new("country-list")
                        .short('C')
                        .long("country-list")
                        // what's the best description for this?
                        .help("Print list of countries where taxon is found.")
                )
                .arg(
                    Arg::new("status")
                        .long("status")
                        .help("Print all data associated with how far this taxon has progressed with genomic sequencing.\nThis includes sample collection, acquisition, progress in sequencing, and whether submitted to INSDC.")
                )
                .arg(
                    Arg::new("names")
                        .short('n')
                        .long("names")
                        .help("Print all associated name data (synonyms, Tree of Life ID, and common names)."),
                )
                .arg(
                    Arg::new("all")
                        .short('A')
                        .long("all")
                        .help("Print all currently implemented GoaT-CLI variables."),
                )
                .arg(
                    Arg::new("raw")
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
                    Arg::new("tidy")
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
                ),
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
                        .help(&format!("A file of NCBI taxonomy ID's (tips) and/or binomial names.\nEach line should contain a single entry.\nFile size is limited to {} entries.", pretty_print_usize(*UPPER_CLI_FILE_LIMIT))[..]),
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
        .get_matches();

    match matches.subcommand() {
        Some(("search", matches)) => {
            let progress_bar = matches.is_present("progress-bar");
            let unique_ids = generate_unique_strings(matches)?;

            match progress_bar {
                true => {
                    try_join!(
                        search::search(&matches, unique_ids.clone()),
                        progress::progress_bar(&matches, "search", unique_ids)
                    )?;
                }
                false => {
                    let _ = search::search(&matches, unique_ids).await?;
                }
            }
        }
        Some(("count", matches)) => {
            let unique_ids = generate_unique_strings(matches)?;
            count::count(&matches, true, false, unique_ids).await?;
        }
        Some(("lookup", matches)) => {
            lookup::lookup(&matches, true).await?;
        }
        Some(("newick", matches)) => {
            let progress_bar = matches.is_present("progress-bar");
            let unique_ids = generate_unique_strings(matches)?;

            match progress_bar {
                true => {
                    try_join!(
                        newick::get_newick(matches, unique_ids.clone()),
                        progress::progress_bar(&matches, "newick", unique_ids)
                    )?;
                }
                false => newick::get_newick(matches, unique_ids).await?,
            }
        }
        _ => {
            unreachable!()
        }
    }

    Ok(())
}
