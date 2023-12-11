use goat_cli::error::Result;
use std::path::PathBuf;
use clap::{crate_version, Arg, Command, value_parser, ArgAction::SetTrue};
use futures::try_join;

use goat_cli::{
    count, lookup, progress,
    report::{self, report::ReportType},
    search,
    utils::utils::{generate_unique_strings, pretty_print_usize},
    IndexType,
    UPPER_CLI_FILE_LIMIT, UPPER_CLI_SIZE_LIMIT,
};

#[tokio::main]
async fn main() {
    let result = run().await;
    match result {
        Ok(_) => (),
        // format the errors nicely
        Err(e) => eprintln!("{}", e),
    }
}

async fn run() -> Result<()> {
    // global helps for both assembly/taxon subcommands.
    let upper_file_limit = pretty_print_usize(*UPPER_CLI_FILE_LIMIT);
    let upper_cli_limit = pretty_print_usize(*UPPER_CLI_SIZE_LIMIT);
    let taxon_file_or_lookup_help = format!("A file of NCBI taxonomy ID's (tips) and/or binomial names.\nEach line should contain a single entry.\nFile size is limited to {} entries.", upper_file_limit);
    let taxon_size_help = format!("The number of results to return. Max {} currently.", upper_cli_limit);

    let taxon_search_and_count = |name, about| {
        Command::new(name) 
            .about(about)
            .arg(
                Arg::new("taxon")
                    .short('t')
                    .long("taxon")
                    .required_unless_present_any(["file", "print-expression", "variables"])
                    .help("The taxon to search. An NCBI taxon ID, or the name of a taxon at any rank."),
            )
            .arg(
                Arg::new("file")
                    .short('f')
                    .long("file")
                    .value_parser(value_parser!(PathBuf))
                    .required_unless_present_any(["taxon", "print-expression", "variables"])
                    .help(taxon_file_or_lookup_help.clone()),
            )
            .arg(
                Arg::new("variables")
                    .short('v')
                    .long("variables")
                    .required_unless_present_any(["file", "print-expression", "taxon"])
                    .help("Variable parser. Input a comma separated string of variables.")
            )
            .arg(
                Arg::new("size")
                    .long("size")
                    .default_value("50")
                    .value_parser(value_parser!(u64))
                    .help(taxon_size_help.clone()),
            )
            .arg(
                Arg::new("ranks")
                    .short('R')
                    .long("ranks")
                    .value_parser(["none", "subspecies", "species", "genus", "family", "order", "class", "phylum", "kingdom", "superkingdom"])
                    .default_value("none")
                    .help("Choose a rank to display with the results. All ranks up to the given rank are displayed.")
            )
            .arg(
                Arg::new("expression")
                    .short('e')
                    .long("expression")
                    .required(false)
                    .help("Use an expression to filter results server-side.")
            )
            .arg(
                Arg::new("tax-rank")
                    .long("tax-rank")
                    .required(false)
                    .help("The taxonomic rank to return the results at.")
            )
            .arg(
                Arg::new("exclude")
                    .short('x')
                    .long("exclude")
                    .action(SetTrue)
                    .help("Exclude all missing and ancestral values, so that a returned table may contain only direct measures (excluding missing/ancestral). If multiple variables are requested, a row is only returned if all the variables have a direct value. Will only take effect if one or more variables are specified.")
            )
            // display level 1
            .arg(
                Arg::new("taxon-assembly")
                    .short('a')
                    .long("assembly")
                    .action(SetTrue)
                    .help("Print assembly data (assembly span, assembly level)."),
            )
            .arg(
                Arg::new("taxon-busco")
                    .short('b')
                    .long("busco")
                    .action(SetTrue)
                    .help("Print BUSCO estimates."),
            )
            .arg(
                Arg::new("taxon-gc-percent")
                    .short('g') 
                    .long("gc-percent")
                    .action(SetTrue)
                    .help("Print GC%.")
            )
            .arg(
                Arg::new("taxon-karyotype")
                    .short('k')
                    .long("karyotype")
                    .action(SetTrue)
                    .help("Print karyotype data (chromosome number & haploid number)."),
            )
            .arg(
                Arg::new("taxon-genome-size")
                    .short('G')
                    .long("genome-size")
                    .action(SetTrue)
                    .help("Print genome size data."),
            )
            // further display levels
            .arg(
                Arg::new("taxon-bioproject")
                    .short('B')
                    .long("bioproject")
                    .action(SetTrue)
                    .help("Print the bioproject and biosample ID of records.")
            )
            .arg(
                Arg::new("taxon-n50")
                    .short('N')
                    .long("n50")
                    .action(SetTrue)
                    .help("Print the contig & scaffold n50 of assemblies.")
            )
            .arg(
                Arg::new("taxon-date")
                    .short('D')
                    .long("date")
                    .action(SetTrue)
                    .help("Print EBP & assembly dates.")
            )
            .arg(
                Arg::new("taxon-gene-count")
                    .long("gene-count")
                    .action(SetTrue)
                    .help("Print gene count data.")
            )
            .arg(
                Arg::new("taxon-mitochondria")
                    .short('m')
                    .long("mitochondria")
                    .action(SetTrue)
                    .help("Print mitochondrial genome assembly size & GC%.")
            )
            .arg(
                Arg::new("taxon-plastid")
                    .short('p')
                    .long("plastid")
                    .action(SetTrue)
                    .help("Print plastid genome assembly size & GC%.")
            )
            .arg(
                Arg::new("taxon-sex-determination")
                    .short('S')
                    .long("sex-determination")
                    .action(SetTrue)
                    .help("Print sex determination data."),
            )
            .arg(
                Arg::new("taxon-ploidy")
                    .short('P')
                    .long("ploidy")
                    .action(SetTrue)
                    .help("Print ploidy estimates.")
            )
            .arg(
                Arg::new("taxon-c-values")
                    .short('c')
                    .long("c-values")
                    .action(SetTrue)
                    .help("Print c-value data."),
            )
            .arg(
                Arg::new("taxon-legislation")
                    .long("legislation")
                    .action(SetTrue)
                    .help("Print legislation data."),
            )
            .arg(
                Arg::new("lineage")
                    .short('l')
                    .long("lineage")
                    .action(SetTrue)
                    .conflicts_with("descendents")
                    .help("Displays lineage information. I.e. from this node in the tree go back and give all the nodes to the root. Conflicts with descendents."),
            )
            .arg(
                Arg::new("taxon-target-lists")
                    .long("target-lists")
                    .action(SetTrue)
                    .help("Print target list data associated with each taxon.")
            )
            .arg(
                Arg::new("taxon-country-list")
                    .short('C')
                    .long("country-list")
                    .action(SetTrue)
                    // what's the best description for this?
                    .help("Print list of countries where taxon is found.")
            )
            .arg(
                Arg::new("taxon-status")
                    .long("status")
                    .action(SetTrue)
                    .help("Print all data associated with how far this taxon has progressed with genomic sequencing.\nThis includes sample collection, acquisition, progress in sequencing, and whether submitted to INSDC.")
            )
            .arg(
                Arg::new("taxon-names")
                    .short('n')
                    .long("names")
                    .action(SetTrue)
                    .help("Print all associated name data (synonyms, Tree of Life ID, and common names)."),
            )
            .arg(
                Arg::new("taxon-raw")
                    .short('r')
                    .long("raw")
                    .action(SetTrue)
                    .help("Print raw values (i.e. no aggregation/summary)."),
            )
            .arg(
                Arg::new("descendents")
                    .short('d')
                    .long("descendents")
                    .action(SetTrue)
                    .help("Get information for all descendents of a common ancestor."),
            )
            .arg(
                Arg::new("taxon-tidy")
                    .long("tidy")
                    .short('T')
                    .action(SetTrue)
                    .help("Print data in tidy format.")
            )
            .arg(
                Arg::new("include-estimates")
                    .short('i')
                    .long("include-estimates")
                    .action(SetTrue)
                    .conflicts_with("raw")
                    .help("Include ancestral estimates. Omitting this flag includes only direct estimates from a taxon. Cannot be used with --raw.")
            )
            .arg(
                Arg::new("toggle-direct")
                    .long("toggle-direct")
                    .action(SetTrue)
                    .help("For each variable specified, return additional columns for direct measures, ancestral inferred, and descendent inferred.")
            )
            .arg(
                Arg::new("print-expression")
                    .long("print-expression")
                    .action(SetTrue)
                    .help("Print all variables in GoaT currently, with their associated variants.\nUseful for construction of expressions.")
            )
            .arg(
                Arg::new("progress-bar")
                    .long("progress-bar")
                    .action(SetTrue)
                    .help("Add a progress bar to large queries, to estimate time left.")
            )
            .arg(
                Arg::new("url")
                    .short('u')
                    .long("url")
                    .action(SetTrue)
                    .help("Print the underlying GoaT API URL(s). Useful for debugging."),
            )
            .arg(
                Arg::new("goat-ui-url")
                    .short('U')
                    .long("goat-ui-url")
                    .action(SetTrue)
                    .help("Print the underlying GoaT UI URL(s). View on the browser!"),
            )
        };

    let assembly_search_and_count = |name, about| {
        Command::new(name) 
            .about(about)
            .arg(
                Arg::new("taxon")
                    .short('t')
                    .long("taxon")
                    .required_unless_present_any(["file", "print-expression", "variables"])
                    .help("The taxon to search. An NCBI taxon ID, or the name of a taxon at any rank."),
            )
            .arg(
                Arg::new("file")
                    .short('f')
                    .long("file")
                    .value_parser(value_parser!(PathBuf))
                    .required_unless_present_any(["taxon", "print-expression", "variables"])
                    .help(taxon_file_or_lookup_help.clone()),
            )
            .arg(
                // will require a new database
                Arg::new("variables")
                    .short('v')
                    .long("variables")
                    .required_unless_present_any(["file", "print-expression", "taxon"])
                    .help("Variable parser. Input a comma separated string of variables.")
            )
            .arg(
                Arg::new("size")
                    .long("size")
                    .default_value("50")
                    .value_parser(value_parser!(u64))
                    .help(taxon_size_help.clone()),
            )
            .arg(
                Arg::new("ranks")
                    .short('R')
                    .long("ranks")
                    .value_parser(["none", "subspecies", "species", "genus", "family", "order", "class", "phylum", "kingdom", "superkingdom"])
                    .default_value("none")
                    .help("Choose a rank to display with the results. All ranks up to the given rank are displayed.")
            )
            // will also need another database.
            .arg(
                Arg::new("expression")
                    .short('e')
                    .long("expression")
                    .required(false)
                    .help("Use an expression to filter results server-side.")
            )
            .arg(
                Arg::new("tax-rank")
                    .long("tax-rank")
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
                Arg::new("exclude")
                    .short('x')
                    .long("exclude")
                    .action(SetTrue)
                    .help("Exclude all missing and ancestral values, so that a returned table may contain only direct measures (excluding missing/ancestral). If multiple variables are requested, a row is only returned if all the variables have a direct value. Will only take effect if one or more variables are specified.")
            )
            // flags
            .arg(
                Arg::new("assembly-assembly")
                    .short('a')
                    .long("assembly")
                    .action(SetTrue)
                    .help("Print assembly data (span & level)"),
            )
            .arg(
                Arg::new("assembly-karyotype")
                    .short('k')
                    .long("karyotype")
                    .action(SetTrue)
                    .help("Print karyotype data (chromosome number only)."),
            )
            .arg(
                Arg::new("assembly-contig")
                    .short('c')
                    .long("contig")
                    .action(SetTrue)
                    .help("Print contig data (count, l50, n50)."),
            )
            .arg(
                Arg::new("assembly-scaffold")
                    .short('s')
                    .long("scaffold")
                    .action(SetTrue)
                    .help("Print scaffold data (count, l50, n50)."),
            )
            .arg(
                Arg::new("assembly-gene-count")
                    .short('g')
                    .long("gene-count")
                    .action(SetTrue)
                    .help("Print gene count data (gene count, non-coding gene count)."),
            )
            .arg(
                Arg::new("assembly-busco")
                    .short('b')
                    .long("busco")
                    .action(SetTrue)
                    .help("Print BUSCO data (BUSCO completeness, lineage, and string)."),
            )
            .arg(
                Arg::new("assembly-btk")
                    .long("btk")
                    .action(SetTrue)
                    .help("Print BlobToolKit data (no-hit, target)."),
            )
            .arg(
                Arg::new("descendents")
                    .short('d')
                    .long("descendents")
                    .action(SetTrue)
                    .help("Get information for all descendents of a common ancestor."),
            )
            .arg(
                Arg::new("include-estimates")
                    .short('i')
                    .long("include-estimates")
                    .action(SetTrue)
                    .conflicts_with("raw")
                    .help("Include ancestral estimates. Omitting this flag includes only direct estimates from a taxon. Cannot be used with --raw.")
            )
            .arg(
                Arg::new("print-expression")
                    .long("print-expression")
                    .action(SetTrue)
                    .help("Print all variables in GoaT currently, with their associated variants.\nUseful for construction of expressions.")
            )
            .arg(
                Arg::new("progress-bar")
                    .long("progress-bar")
                    .action(SetTrue)
                    .help("Add a progress bar to large queries, to estimate time left.")
            )
            .arg(
                Arg::new("url")
                    .short('u')
                    .long("url")
                    .action(SetTrue)
                    .help("Print the underlying GoaT API URL(s). Useful for debugging."),
            )
            .arg(
                Arg::new("goat-ui-url")
                    .short('U')
                    .long("goat-ui-url")
                    .action(SetTrue)
                    .help("Print the underlying GoaT UI URL(s). View on the browser!"),
            )
    };

    // actually parse the matches.
    let matches = Command::new("goat-cli")
        // to fix the binary name in the help messages
        .bin_name("goat-cli")
        .version(crate_version!())
        .propagate_version(true)
        .arg_required_else_help(true)
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
                                    .required_unless_present("file")
                                    .help("The taxon to search. An NCBI taxon ID, or the name of a taxon at any rank."),
                            )
                            .arg(
                                Arg::new("file")
                                    .short('f')
                                    .long("file")
                                    .value_parser(value_parser!(PathBuf))
                                    .required_unless_present_any(["taxon"])
                                    .help(taxon_file_or_lookup_help.clone()),
                            )
                            .arg(
                                Arg::new("url")
                                    .short('u')
                                    .long("url")
                                    .action(SetTrue)
                                    .help("Print lookup URL.")
                            )
                            .arg(
                                Arg::new("size")
                                    .short('s')
                                    .long("size")
                                    .default_value("10")
                                    .value_parser(value_parser!(u64))
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
                                    .required_unless_present("file")
                                    .help("The taxon to return a newick of. Multiple taxa will return the joint tree."),
                            )
                            .arg(
                                Arg::new("url")
                                    .short('u')
                                    .long("url")
                                    .action(SetTrue)
                                    .help("Print report URL.")
                            )
                            .arg(
                                Arg::new("rank")
                                    .short('r')
                                    .long("rank")
                                    .default_value("species")
                                    .value_parser(["species", "genus", "family", "order"])
                                    .help("The rank of the results to return."),
                            )
                            .arg(
                                Arg::new("progress-bar")
                                    .long("progress-bar")
                                    .action(SetTrue)
                                    .help("Add a progress bar to large queries, to estimate time left.")
                            ),
                    )
                    .subcommand(
                        Command::new("hist")
                            .about("Generate a histogram report from input taxa.")
                            .arg(
                                Arg::new("taxon")
                                    .short('t')
                                    .long("taxon")
                                    .required_unless_present("file")
                                    .help("The taxon to return a histogram of. Multiple taxa will return the joint histogram."),
                            )
                            .arg(
                                Arg::new("url")
                                    .short('u')
                                    .long("url")
                                    .action(SetTrue)
                                    .help("Print report URL.")
                            )
                            .arg(
                                Arg::new("no-descendents")
                                    .short('n')
                                    .long("no-descendents")
                                    .action(SetTrue)
                                    .help("If a taxon is supplied, do not return values for its descendents (i.e. a tax_name() call).")
                            )
                            .arg(
                                Arg::new("rank")
                                    .short('r')
                                    .long("rank")
                                    .default_value("species")
                                    .value_parser(["species", "genus", "family", "order"])
                                    .help("The number of results to return."),
                            )
                            .arg(
                                Arg::new("x-variable")
                                    .short('x')
                                    .long("x-variable")
                                    .required(true)
                                    .help("The name of the x variable."),
                            )
                            .arg(
                                Arg::new("size")
                                    .short('s')
                                    .long("size")
                                    .default_value("10")
                                    .value_parser(value_parser!(usize))
                                    .help("The number of category levels to return."),
                            )
                            .arg(
                                Arg::new("x-opts")
                                    .short('o')
                                    .long("opts")
                                    .required(false)
                                    .help("The options for the variable axis. A comma separated string of options in the order:
\t1. minimum value
\t2. maximum value
\t3. tick count
\t4. scale (linear, sqrt, log10, log2, log, proportion, or ordinal)
\t5. axis title\nE.g. ',,20' is 20 bins. '1,10,5' is start at 1, end at 10, with 5 bins."),
                            )
                    )
                    .subcommand(
                        Command::new("cat-hist")
                            .about("Generate a categorical histogram report from input taxa.")
                            .arg(
                                Arg::new("taxon")
                                    .short('t')
                                    .long("taxon")
                                    .required_unless_present("file")
                                    .help("The taxon to return a categorical histogram of. Multiple taxa will return the joint histogram."),
                            )
                            .arg(
                                Arg::new("url")
                                    .short('u')
                                    .long("url")
                                    .action(SetTrue)
                                    .help("Print report URL.")
                            )
                            .arg(
                                Arg::new("no-descendents")
                                    .short('n')
                                    .long("no-descendents")
                                    .action(SetTrue)
                                    .help("If a taxon is supplied, do not return values for its descendents (i.e. a tax_name() call).")
                            )
                            .arg(
                                Arg::new("rank")
                                    .short('r')
                                    .long("rank")
                                    .default_value("species")
                                    .value_parser(["species", "genus", "family", "order"])
                                    .help("The rank of the results to return."),
                            )
                            .arg(
                                Arg::new("x-variable")
                                    .short('x')
                                    .long("x-variable")
                                    .required(true)
                                    .help("The name of the x variable."),
                            )
                            .arg(
                                Arg::new("category")
                                    .short('c')
                                    .long("category")
                                    .required(true)
                                    .help("The category with which to group the histogram over."),
                            )
                            .arg(
                                Arg::new("size")
                                    .short('s')
                                    .long("size")
                                    .default_value("10")
                                    .value_parser(value_parser!(usize))
                                    .help("The number of category levels to return."),
                            )
                            .arg(
                                Arg::new("x-opts")
                                    .short('o')
                                    .long("opts")
                                    .required(false)
                                    .help("The options for the variable axis. A comma separated string of options in the order:
\t1. minimum value
\t2. maximum value
\t3. tick count
\t4. scale (linear, sqrt, log10, log2, log, proportion, or ordinal)
\t5. axis title\nE.g. ',,20' is 20 bins. '1,10,5' is start at 1, end at 10, with 5 bins.
"),
                            )
                    )
                    .subcommand(
                        Command::new("scatter")
                            .about("Generate a scatter (bivariate) report.")
                            .arg(
                                Arg::new("taxon")
                                    .short('t')
                                    .long("taxon")
                                    .required_unless_present("file")
                                    .help("The taxon to return a scatter of. Multiple taxa will return the joint scatter."),
                            )
                            .arg(
                                Arg::new("url")
                                    .short('u')
                                    .long("url")
                                    .action(SetTrue)
                                    .help("Print report URL.")
                            )
                            .arg(
                                Arg::new("no-descendents")
                                    .short('n')
                                    .long("no-descendents")
                                    .action(SetTrue)
                                    .help("If a taxon is supplied, do not return values for its descendents (i.e. a tax_name() call).")
                            )
                            .arg(
                                Arg::new("rank")
                                    .short('r')
                                    .long("rank")
                                    .default_value("species")
                                    .value_parser(["species", "genus", "family", "order"])
                                    .help("The rank of the results to return."),
                            )
                            .arg(
                                Arg::new("x-variable")
                                    .short('x')
                                    .long("x-variable")
                                    .required(true)
                                    .help("The name of the x variable."),
                            )
                            .arg(
                                Arg::new("y-variable")
                                    .short('y')
                                    .long("y-variable")
                                    .required(true)
                                    .help("The name of the y variable."),
                            )
                            .arg(
                                Arg::new("size")
                                    .short('s')
                                    .long("size")
                                    .default_value("10")
                                    .value_parser(value_parser!(usize))
                                    .help("The number of category levels to return."),
                            )
                            .arg(
                                Arg::new("x-opts")
                                    .long("x-opts")
                                    .required(false)
                                    .help("The options for the variable axis. A comma separated string of options in the order:
\t1. minimum value
\t2. maximum value
\t3. tick count
\t4. scale (linear, sqrt, log10, log2, log, proportion, or ordinal)
\t5. axis title\nE.g. ',,20' is 20 bins. '1,10,5' is start at 1, end at 10, with 5 bins.
"),
                            )
                            .arg(
                                Arg::new("y-opts")
                                    .long("y-opts")
                                    .required(false)
                                    .help("As for x options."),
                    )
                )
            )
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
                                        .required_unless_present("file")
                                        .help("The taxon to search. An NCBI taxon ID, or the name of a taxon at any rank."),
                                )
                                .arg(
                                    Arg::new("file")
                                        .short('f')
                                        .long("file")
                                        .value_parser(value_parser!(PathBuf))
                                        .required_unless_present_any(["taxon"])
                                        .help(taxon_file_or_lookup_help),
                                )
                                .arg(
                                    Arg::new("url")
                                        .short('u')
                                        .long("url")
                                        .action(SetTrue)
                                        .help("Print lookup URL.")
                                )
                                .arg(
                                    Arg::new("size")
                                        .short('s')
                                        .long("size")
                                        .default_value("10")
                                        .value_parser(value_parser!(u64))
                                        .help("The number of results to return."),
                                )
                    )
            )
        .get_matches();

    // nested matching on subcommands
    match matches.subcommand() {
        // outer == taxon/assembly
        Some(("taxon", taxon_matches)) => match taxon_matches.subcommand() {
            // inner are all the taxon matches here.
            Some(("search", taxon_search_matches)) => {
                let progress_bar = *taxon_search_matches.get_one::<bool>("progress-bar").expect("cli default false");
                let unique_ids = generate_unique_strings(taxon_search_matches, IndexType::Taxon)?;

                match progress_bar {
                    true => {
                        try_join!(
                            search::search(taxon_search_matches, unique_ids.clone(), IndexType::Taxon),
                            progress::progress_bar(taxon_search_matches, "search", unique_ids, IndexType::Taxon)
                        )?;
                    }
                    false => {
                        search::search(taxon_search_matches, unique_ids, IndexType::Taxon).await?;
                    }
                }
            }
            Some(("count", taxon_count_matches)) => {
                let unique_ids = generate_unique_strings(taxon_count_matches, IndexType::Taxon)?;
                count::count(taxon_count_matches, true, false, unique_ids, IndexType::Taxon).await?;
            }
            Some(("lookup", taxon_lookup_matches)) => {
                lookup::lookup(taxon_lookup_matches, true, IndexType::Taxon).await?;
            }
            Some(("hist", taxon_hist_matches)) => {
                let unique_ids = generate_unique_strings(taxon_hist_matches, IndexType::Taxon)?;
                report::fetch::fetch_report(taxon_hist_matches, unique_ids, ReportType::Histogram).await?;
            }
            Some(("cat-hist", taxon_cat_hist_matches)) => {
                let unique_ids = generate_unique_strings(taxon_cat_hist_matches, IndexType::Taxon)?;
                report::fetch::fetch_report(taxon_cat_hist_matches, unique_ids, ReportType::CategoricalHistogram).await?;
            }
            Some(("scatter", scatter_matches)) => {
                let unique_ids = generate_unique_strings(scatter_matches, IndexType::Taxon)?;
                report::fetch::fetch_report(scatter_matches, unique_ids, ReportType::Scatterplot).await?;
            }
            Some(("newick", taxon_newick_matches)) => {
                let progress_bar = *taxon_newick_matches.get_one::<bool>("progress-bar").expect("cli detault false");
                let unique_ids = generate_unique_strings(taxon_newick_matches, IndexType::Taxon)?;

                match progress_bar {
                    true => {
                        try_join!(
                            report::fetch::fetch_report(taxon_newick_matches, unique_ids.clone(), ReportType::Newick),
                            progress::progress_bar(taxon_newick_matches, "newick", unique_ids, IndexType::Taxon)
                        )?;
                    }
                    false => report::fetch::fetch_report(taxon_newick_matches, unique_ids, ReportType::Newick).await?,
                }
            }
            _ => {
                unreachable!()
            }
        },
        // and now assembly
        Some(("assembly", assembly_matches)) => match assembly_matches.subcommand() {
            // and the three implemented subcommands currently.
            Some(("search", assembly_search_matches)) => {
                let progress_bar = *assembly_search_matches.get_one::<bool>("progress-bar").expect("cli detault false");
                let unique_ids = generate_unique_strings(assembly_search_matches, IndexType::Assembly)?;

                match progress_bar {
                    true => {
                        try_join!(
                            search::search(assembly_search_matches, unique_ids.clone(), IndexType::Assembly),
                            progress::progress_bar(assembly_search_matches, "search", unique_ids, IndexType::Assembly)
                        )?;
                    }
                    false => {
                        search::search(assembly_search_matches, unique_ids, IndexType::Assembly).await?;
                    }
                }
            }
            Some(("count", assembly_count_matches)) => {
                let unique_ids = generate_unique_strings(assembly_count_matches, IndexType::Assembly)?;
                count::count(assembly_count_matches, true, false, unique_ids, IndexType::Assembly).await?;
            }
            Some(("lookup", assembly_lookup_matches)) => {
                lookup::lookup(assembly_lookup_matches, true, IndexType::Assembly).await?;
            }
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }

    Ok(())
}
