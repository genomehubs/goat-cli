use anyhow::{bail, Result};
use clap::{App, Arg};
use tokio;

use goat::count::count;
use goat::lookup::lookup::lookup;
use goat::record::newick;
use goat::search::run;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = App::new("goat")
        .version(clap::crate_version!())
        .author("Max Brown <mb39@sanger.ac.uk>")
        .about("GoaTs on a terminal. Combine flags to query metadata for any species.")
        .subcommand(
            clap::SubCommand::with_name("search")
                .about("Query the GoaT search API.")
                .arg(
                    Arg::with_name("taxon")
                        .short("t")
                        .long("taxon")
                        .takes_value(true)
                        .required_unless("file")
                        .help("The taxon to search. An NCBI taxon ID, or the name of a taxon at any rank."),
                )
                .arg(
                    Arg::with_name("file")
                        .short("f")
                        .long("file")
                        .takes_value(true)
                        .required_unless("taxon")
                        .help("A file of NCBI taxonomy ID's (tips) and/or binomial names.\nEach line should contain a single entry."),
                )
                .arg(
                    Arg::with_name("raw")
                        .short("r")
                        .long("raw")
                        .help("Print raw values (i.e. no aggregation/summary)."),
                )
                .arg(
                    Arg::with_name("all")
                        .short("A")
                        .long("all")
                        .help("Print all currently implemented GoaT variables."),
                )
                .arg(
                    Arg::with_name("assembly")
                        .short("a")
                        .long("assembly")
                        .help("Print assembly data (span & level)"),
                )
                .arg(
                    Arg::with_name("c-values")
                        .short("c")
                        .long("c-values")
                        .help("Print c-value data."),
                )
                .arg(
                    Arg::with_name("karyotype")
                        .short("k")
                        .long("karyotype")
                        .help("Print karyotype data (chromosome number & haploid number)."),
                )
                .arg(
                    Arg::with_name("ploidy")
                        .short("P")
                        .long("ploidy")
                        .help("Print ploidy estimates.")
                )
                .arg(
                    Arg::with_name("sex-determination")
                        .short("S")
                        .long("sex-determination")
                        .help("Print sex determination data."),
                )
                .arg(
                    Arg::with_name("genome-size")
                        .short("g")
                        .long("genome-size")
                        .help("Print genome size data."),
                )
                .arg(
                    Arg::with_name("legislation")
                        .short("l")
                        .long("legislation")
                        .help("Print legislation data."),
                )
                .arg(
                    Arg::with_name("names")
                        .short("n")
                        .long("names")
                        .help("Print all associated name data (synonyms, Tree of Life ID, and common names)."),
                )
                .arg(
                    Arg::with_name("url")
                        .short("u")
                        .long("url")
                        .help("Print the underlying GoaT API URL(s). Useful for debugging."),
                )
                .arg(
                    Arg::with_name("descendents")
                        .short("d")
                        .long("descendents")
                        .help("Get information for all descendents of a common ancestor."),
                )
                .arg(
                    Arg::with_name("busco")
                        .short("b")
                        .long("busco")
                        .help("Print BUSCO estimates."),
                )
                .arg(
                    Arg::with_name("size")
                        .short("s")
                        .long("size")
                        .default_value("50")
                        .help("The number of results to return. Max 10,000 currently."),
                )
                .arg(
                    Arg::with_name("mitochondria")
                        .short("m")
                        .long("mitochondria")
                        .help("Print mitochondrial genome size & GC%.")
                )
                .arg(
                    Arg::with_name("plastid")
                        .short("p")
                        .long("plastid")
                        .help("Print plastid genome size & GC%.")
                )
                .arg(
                    Arg::with_name("ranks")
                        .short("R")
                        .long("ranks")
                        .possible_values(&["none", "subspecies", "species", "genus", "family", "order", "class", "phylum", "kingdom", "superkingdom"])
                        .default_value("none")
                        .help("Choose a rank to display with the results. All ranks up to the given rank are displayed.")
                )
                .arg(
                    Arg::with_name("target-lists")
                        .short("t")
                        .long("target-lists")
                        .help("Print target list data associated with each taxon.")
                )
                .arg(
                    Arg::with_name("n50")
                        .short("N")
                        .long("n50")
                        .help("Print the contig & scaffold n50 of assemblies.")
                )
                .arg(
                    Arg::with_name("bioproject")
                        .short("B")
                        .long("bioproject")
                        .help("Print the bioproject and biosample ID of records.")
                )
                .arg(
                    Arg::with_name("tidy")
                        .long("tidy")
                        .short("T")
                        .help("Print data in tidy format.")
                )
                .arg(
                    Arg::with_name("gene-count")
                        .short("G")
                        .long("gene-count")
                        .help("Print gene count data.")
                )
                .arg(
                    Arg::with_name("date")
                        .short("D")
                        .long("date")
                        .help("Print EBP & assembly dates.")
                )
                .arg(
                    Arg::with_name("country-list")
                        .short("C")
                        .long("country-list")
                        // what's the best description for this?
                        .help("Print list of countries where taxon is found.")
                )
                .arg(
                    Arg::with_name("include-estimates")
                        .short("i")
                        .long("include-estimates")
                        .conflicts_with("raw")
                        .help("Include ancestral estimates. Omitting this flag includes only direct estimates from a taxon. Cannot be used with --raw.")
                ),
        )
        // copy of the above.
        .subcommand(
            clap::SubCommand::with_name("count")
                .about("Query the GoaT count API. Return the number of hits from any search.")
                .arg(
                    Arg::with_name("taxon")
                        .short("t")
                        .long("taxon")
                        .takes_value(true)
                        .required_unless("file")
                        .help("The taxon to search. An NCBI taxon ID, or the name of a taxon at any rank."),
                )
                .arg(
                    Arg::with_name("file")
                        .short("f")
                        .long("file")
                        .takes_value(true)
                        .required_unless("taxon")
                        .help("A file of NCBI taxonomy ID's (tips) and/or binomial names.\nEach line should contain a single entry."),
                )
                .arg(
                    Arg::with_name("raw")
                        .short("r")
                        .long("raw")
                        .help("Print raw values (i.e. no aggregation/summary)."),
                )
                .arg(
                    Arg::with_name("all")
                        .short("A")
                        .long("all")
                        .help("Print all currently implemented GoaT variables."),
                )
                .arg(
                    Arg::with_name("assembly")
                        .short("a")
                        .long("assembly")
                        .help("Print assembly data (span & level)"),
                )
                .arg(
                    Arg::with_name("c-values")
                        .short("c")
                        .long("c-values")
                        .help("Print c-value data."),
                )
                .arg(
                    Arg::with_name("karyotype")
                        .short("k")
                        .long("karyotype")
                        .help("Print karyotype data (chromosome number & haploid number)."),
                )
                .arg(
                    Arg::with_name("ploidy")
                        .short("P")
                        .long("ploidy")
                        .help("Print ploidy estimates.")
                )
                .arg(
                    Arg::with_name("sex-determination")
                        .short("S")
                        .long("sex-determination")
                        .help("Print sex determination data."),
                )
                .arg(
                    Arg::with_name("genome-size")
                        .short("g")
                        .long("genome-size")
                        .help("Print genome size data."),
                )
                .arg(
                    Arg::with_name("legislation")
                        .short("l")
                        .long("legislation")
                        .help("Print legislation data."),
                )
                .arg(
                    Arg::with_name("names")
                        .short("n")
                        .long("names")
                        .help("Print all associated name data (synonyms, Tree of Life ID, and common names)."),
                )
                .arg(
                    Arg::with_name("url")
                        .short("u")
                        .long("url")
                        .help("Print the underlying GoaT API URL(s). Useful for debugging."),
                )
                .arg(
                    Arg::with_name("descendents")
                        .short("d")
                        .long("descendents")
                        .help("Get information for all descendents of a common ancestor."),
                )
                .arg(
                    Arg::with_name("busco")
                        .short("b")
                        .long("busco")
                        .help("Print BUSCO estimates."),
                )
                .arg(
                    Arg::with_name("size")
                        .short("s")
                        .long("size")
                        .default_value("50")
                        .help("The number of results to return. Max 10,000 currently."),
                )
                .arg(
                    Arg::with_name("mitochondria")
                        .short("m")
                        .long("mitochondria")
                        .help("Print mitochondrial genome size & GC%.")
                )
                .arg(
                    Arg::with_name("plastid")
                        .short("p")
                        .long("plastid")
                        .help("Print plastid genome size & GC%.")
                )
                .arg(
                    Arg::with_name("ranks")
                        .short("R")
                        .long("ranks")
                        .possible_values(&["none", "subspecies", "species", "genus", "family", "order", "class", "phylum", "kingdom", "superkingdom"])
                        .default_value("none")
                        .help("Choose a rank to display with the results. All ranks up to the given rank are displayed.")
                )
                .arg(
                    Arg::with_name("target-lists")
                        .long("target-lists")
                        .help("Print target list data associated with each taxon.")
                )
                .arg(
                    Arg::with_name("n50")
                        .short("N")
                        .long("n50")
                        .help("Print the contig & scaffold n50 of assemblies.")
                )
                .arg(
                    Arg::with_name("bioproject")
                        .short("B")
                        .long("bioproject")
                        .help("Print the bioproject and biosample ID of records.")
                )
                .arg(
                    Arg::with_name("tidy")
                        .long("tidy")
                        .short("T")
                        .help("Print data in tidy format.")
                )
                .arg(
                    Arg::with_name("gene-count")
                        .short("G")
                        .long("gene-count")
                        .help("Print gene count data.")
                )
                .arg(
                    Arg::with_name("date")
                        .short("D")
                        .long("date")
                        .help("Print EBP & assembly dates.")
                )
                .arg(
                    Arg::with_name("country-list")
                        .short("C")
                        .long("country-list")
                        // what's the best description for this?
                        .help("Print list of countries where taxon is found.")
                )
                .arg(
                    Arg::with_name("include-estimates")
                        .short("i")
                        .long("include-estimates")
                        .conflicts_with("raw")
                        .help("Include ancestral estimates. Omitting this flag includes only direct estimates from a taxon. Cannot be used with --raw.")
                ),
        )
        .subcommand(
            clap::SubCommand::with_name("lookup")
                .about("Query the GoaT lookup API.")
                .arg(
                    Arg::with_name("taxon")
                        .short("t")
                        .long("taxon")
                        .takes_value(true)
                        .required_unless("file")
                        .help("The taxon to search. An NCBI taxon ID, or the name of a taxon at any rank."),
                )
                .arg(
                    Arg::with_name("url")
                        .short("u")
                        .long("url")
                        .help("Print lookup URL.")
                )
                .arg(
                    Arg::with_name("size")
                        .short("s")
                        .long("size")
                        .default_value("10")
                        .help("The number of results to return."),
                )
        )
        .subcommand(
            clap::SubCommand::with_name("newick")
                .about("Query the GoaT record API, and return a newick.")
                .arg(
                    Arg::with_name("taxon")
                        .short("t")
                        .long("taxon")
                        .takes_value(true)
                        .required_unless("file")
                        .help("The taxon to return a newick of. Multiple taxa will return the joint tree."),
                )
                .arg(
                    Arg::with_name("url")
                        .short("u")
                        .long("url")
                        .help("Print lookup URL.")
                )
                .arg(
                    Arg::with_name("rank")
                        .short("r")
                        .long("rank")
                        .default_value("species")
                        .possible_values(&["species", "genus", "family", "order"])
                        .help("The number of results to return."),
                )
        )
        .get_matches();

    let subcommand = matches.subcommand();
    match subcommand.0 {
        "search" => {
            let matches = subcommand.1.unwrap();
            run::search(&matches).await?;
        }
        "count" => {
            let matches = subcommand.1.unwrap();
            count::count(&matches, true).await?;
        }
        "lookup" => {
            let matches = subcommand.1.unwrap();
            lookup(&matches, true).await?;
        }
        "newick" => {
            let matches = subcommand.1.unwrap();
            newick::get_newick(matches).await?;
        }
        _ => {
            bail!(goat::error::error::NotYetImplemented::CLIError)
        }
    }

    Ok(())
}
