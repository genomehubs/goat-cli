use anyhow::{bail, Result};
use clap::{App, AppSettings, Arg};
use futures::try_join;
use tokio;

use goat::count::count;
use goat::lookup::lookup;
use goat::progress::progress;
use goat::record::newick;
use goat::search::run;
use goat::utils::{
    cli_matches::{UPPER_CLI_FILE_LIMIT, UPPER_CLI_SIZE_LIMIT},
    utils::pretty_print_usize,
};

#[tokio::main]
async fn main() -> Result<()> {
    let matches = App::new("goat")
        .version("0.1.4")
        .global_setting(AppSettings::PropagateVersion)
        .global_setting(AppSettings::UseLongFormatForHelpSubcommand)
        .author("Max Brown <mb39@sanger.ac.uk>")
        .about("GoaTs on a terminal. Combine flags to query metadata for any species.")
        .subcommand(
            App::new("search")
                .about("Query the GoaT search API.")
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
                    Arg::new("raw")
                        .short('r')
                        .long("raw")
                        .help("Print raw values (i.e. no aggregation/summary)."),
                )
                .arg(
                    Arg::new("all")
                        .short('A')
                        .long("all")
                        .help("Print all currently implemented GoaT variables."),
                )
                .arg(
                    Arg::new("assembly")
                        .short('a')
                        .long("assembly")
                        .help("Print assembly data (span & level)"),
                )
                .arg(
                    Arg::new("c-values")
                        .short('c')
                        .long("c-values")
                        .help("Print c-value data."),
                )
                .arg(
                    Arg::new("karyotype")
                        .short('k')
                        .long("karyotype")
                        .help("Print karyotype data (chromosome number & haploid number)."),
                )
                .arg(
                    Arg::new("ploidy")
                        .short('P')
                        .long("ploidy")
                        .help("Print ploidy estimates.")
                )
                .arg(
                    Arg::new("sex-determination")
                        .short('S')
                        .long("sex-determination")
                        .help("Print sex determination data."),
                )
                .arg(
                    Arg::new("genome-size")
                        .short('g')
                        .long("genome-size")
                        .help("Print genome size data."),
                )
                .arg(
                    Arg::new("legislation")
                        .short('l')
                        .long("legislation")
                        .help("Print legislation data."),
                )
                .arg(
                    Arg::new("names")
                        .short('n')
                        .long("names")
                        .help("Print all associated name data (synonyms, Tree of Life ID, and common names)."),
                )
                .arg(
                    Arg::new("url")
                        .short('u')
                        .long("url")
                        .help("Print the underlying GoaT API URL(s). Useful for debugging."),
                )
                .arg(
                    Arg::new("descendents")
                        .short('d')
                        .long("descendents")
                        .help("Get information for all descendents of a common ancestor."),
                )
                .arg(
                    Arg::new("busco")
                        .short('b')
                        .long("busco")
                        .help("Print BUSCO estimates."),
                )
                .arg(
                    Arg::new("size")
                        .short('s')
                        .long("size")
                        .default_value("50")
                        .help(&format!("The number of results to return. Max {} currently.", pretty_print_usize(*UPPER_CLI_SIZE_LIMIT))[..]),
                )
                .arg(
                    Arg::new("mitochondria")
                        .short('m')
                        .long("mitochondria")
                        .help("Print mitochondrial genome size & GC%.")
                )
                .arg(
                    Arg::new("plastid")
                        .short('p')
                        .long("plastid")
                        .help("Print plastid genome size & GC%.")
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
                    Arg::new("target-lists")
                        .long("target-lists")
                        .help("Print target list data associated with each taxon.")
                )
                .arg(
                    Arg::new("n50")
                        .short('N')
                        .long("n50")
                        .help("Print the contig & scaffold n50 of assemblies.")
                )
                .arg(
                    Arg::new("bioproject")
                        .short('B')
                        .long("bioproject")
                        .help("Print the bioproject and biosample ID of records.")
                )
                .arg(
                    Arg::new("tidy")
                        .long("tidy")
                        .short('T')
                        .help("Print data in tidy format.")
                )
                .arg(
                    Arg::new("gene-count")
                        .short('G')
                        .long("gene-count")
                        .help("Print gene count data.")
                )
                .arg(
                    Arg::new("date")
                        .short('D')
                        .long("date")
                        .help("Print EBP & assembly dates.")
                )
                .arg(
                    Arg::new("country-list")
                        .short('C')
                        .long("country-list")
                        // what's the best description for this?
                        .help("Print list of countries where taxon is found.")
                )
                .arg(
                    Arg::new("include-estimates")
                        .short('i')
                        .long("include-estimates")
                        .conflicts_with("raw")
                        .help("Include ancestral estimates. Omitting this flag includes only direct estimates from a taxon. Cannot be used with --raw.")
                )
                .arg(
                    Arg::new("status")
                        .long("status")
                        .help("Print all data associated with how far this taxon has progressed with genomic sequencing.\nThis includes sample collection, acquisition, progress in sequencing, and whether submitted to INSDC.")
                )
                .arg(
                    Arg::new("expression")
                        .short('e')
                        .long("expression")
                        .takes_value(true)
                        .required(false)
                        .help("Experimental expression parser.")
                )
                .arg(
                    Arg::new("print-expression")
                        .long("print-expression")
                        .help("Print all variables in GoaT currently, with their associated variants.\nUseful for construction of expressions.")
                )
                .arg(
                    Arg::new("variables")
                        .short('v')
                        .long("variables")
                        .takes_value(true)
                        .required_unless_present_any(["file", "print-expression", "taxon"])
                        .help("Variable parser. Input a comma separated string of variables.")
                ),
        )
        // copy of the above.
        .subcommand(
            App::new("count")
                .about("Query the GoaT count API. Return the number of hits from any search.")
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
                    Arg::new("raw")
                        .short('r')
                        .long("raw")
                        .help("Print raw values (i.e. no aggregation/summary)."),
                )
                .arg(
                    Arg::new("all")
                        .short('A')
                        .long("all")
                        .help("Print all currently implemented GoaT variables."),
                )
                .arg(
                    Arg::new("assembly")
                        .short('a')
                        .long("assembly")
                        .help("Print assembly data (span & level)"),
                )
                .arg(
                    Arg::new("c-values")
                        .short('c')
                        .long("c-values")
                        .help("Print c-value data."),
                )
                .arg(
                    Arg::new("karyotype")
                        .short('k')
                        .long("karyotype")
                        .help("Print karyotype data (chromosome number & haploid number)."),
                )
                .arg(
                    Arg::new("ploidy")
                        .short('P')
                        .long("ploidy")
                        .help("Print ploidy estimates.")
                )
                .arg(
                    Arg::new("sex-determination")
                        .short('S')
                        .long("sex-determination")
                        .help("Print sex determination data."),
                )
                .arg(
                    Arg::new("genome-size")
                        .short('g')
                        .long("genome-size")
                        .help("Print genome size data."),
                )
                .arg(
                    Arg::new("legislation")
                        .short('l')
                        .long("legislation")
                        .help("Print legislation data."),
                )
                .arg(
                    Arg::new("names")
                        .short('n')
                        .long("names")
                        .help("Print all associated name data (synonyms, Tree of Life ID, and common names)."),
                )
                .arg(
                    Arg::new("url")
                        .short('u')
                        .long("url")
                        .help("Print the underlying GoaT API URL(s). Useful for debugging."),
                )
                .arg(
                    Arg::new("descendents")
                        .short('d')
                        .long("descendents")
                        .help("Get information for all descendents of a common ancestor."),
                )
                .arg(
                    Arg::new("busco")
                        .short('b')
                        .long("busco")
                        .help("Print BUSCO estimates."),
                )
                .arg(
                    Arg::new("size")
                        .short('s')
                        .long("size")
                        .default_value("50")
                        .help(&format!("The number of results to return. Max {} currently.", pretty_print_usize(*UPPER_CLI_SIZE_LIMIT))[..]),
                )
                .arg(
                    Arg::new("mitochondria")
                        .short('m')
                        .long("mitochondria")
                        .help("Print mitochondrial genome size & GC%.")
                )
                .arg(
                    Arg::new("plastid")
                        .short('p')
                        .long("plastid")
                        .help("Print plastid genome size & GC%.")
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
                    Arg::new("target-lists")
                        .long("target-lists")
                        .help("Print target list data associated with each taxon.")
                )
                .arg(
                    Arg::new("n50")
                        .short('N')
                        .long("n50")
                        .help("Print the contig & scaffold n50 of assemblies.")
                )
                .arg(
                    Arg::new("bioproject")
                        .short('B')
                        .long("bioproject")
                        .help("Print the bioproject and biosample ID of records.")
                )
                .arg(
                    Arg::new("tidy")
                        .long("tidy")
                        .short('T')
                        .help("Print data in tidy format.")
                )
                .arg(
                    Arg::new("gene-count")
                        .short('G')
                        .long("gene-count")
                        .help("Print gene count data.")
                )
                .arg(
                    Arg::new("date")
                        .short('D')
                        .long("date")
                        .help("Print EBP & assembly dates.")
                )
                .arg(
                    Arg::new("country-list")
                        .short('C')
                        .long("country-list")
                        // what's the best description for this?
                        .help("Print list of countries where taxon is found.")
                )
                .arg(
                    Arg::new("include-estimates")
                        .short('i')
                        .long("include-estimates")
                        .conflicts_with("raw")
                        .help("Include ancestral estimates. Omitting this flag includes only direct estimates from a taxon. Cannot be used with --raw.")
                )
                .arg(
                    Arg::new("status")
                        .long("status")
                        .help("Print all data associated with how far this taxon has progressed with genomic sequencing.\nThis includes sample collection, acquisition, progress in sequencing, and whether submitted to INSDC.")
                )
                .arg(
                    Arg::new("expression")
                        .short('e')
                        .long("expression")
                        .takes_value(true)
                        .required(false)
                        .help("Experimental expression parser.")
                )
                .arg(
                    Arg::new("print-expression")
                        .long("print-expression")
                        .help("Print all variables in GoaT currently, with their associated variants.\nUseful for construction of expressions.")
                )
                .arg(
                    Arg::new("variables")
                        .short('v')
                        .long("variables")
                        .takes_value(true)
                        .required_unless_present_any(["file", "print-expression", "taxon"])
                        .help("Variable parser. Input a comma separated string of variables.")
                ),        
        )
        .subcommand(
            App::new("lookup")
                .about("Query the GoaT lookup API.")
                .arg(
                    Arg::new("taxon")
                        .short('t')
                        .long("taxon")
                        .takes_value(true)
                        .required_unless_present("file")
                        .help("The taxon to search. An NCBI taxon ID, or the name of a taxon at any rank."),
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
            App::new("newick")
                .about("Query the GoaT record API, and return a newick.")
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
        )
        .get_matches();

    match matches.subcommand() {
        Some(("search", matches)) => {
            try_join!(
                run::search(&matches),
                progress::progress_bar(&matches, "search")
            )?;
        }
        Some(("count", matches)) => {
            count::count(&matches, true, false).await?;
        }
        Some(("lookup", matches)) => {
            lookup::lookup(&matches, true).await?;
        }
        Some(("newick", matches)) => {
            try_join!(
                newick::get_newick(matches),
                progress::progress_bar(&matches, "newick")
            )?;
        }
        _ => {
            bail!(goat::error::error::NotYetImplemented::CLIError)
        }
    }

    Ok(())
}
