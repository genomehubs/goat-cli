use anyhow::{bail, Result};
use clap::{App, Arg};
use tokio;

// use goat::count::count;
use goat::search::run;

// would be nice to add ranks
// this would add the ranks to each formatted output.

#[tokio::main]
async fn main() -> Result<()> {
    let matches = App::new("goat")
        .version(clap::crate_version!())
        .author("Max Brown <mb39@sanger.ac.uk>")
        .about("GoaTs on a terminal.")
        .subcommand(
            clap::SubCommand::with_name("search")
                .about("Query the search API.")
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
                        .help("This flag indicates raw values should be all listed out. So you can do your own aggregations for example."),
                )
                .arg(
                    Arg::with_name("all")
                        .short("z")
                        .long("all")
                        .help("This flag indicates all data should be printed."),
                )
                .arg(
                    Arg::with_name("assembly")
                        .short("a")
                        .long("assembly")
                        .help("This flag indicates assembly data should be printed."),
                )
                .arg(
                    Arg::with_name("c-values")
                        .short("c")
                        .long("c-values")
                        .help("This flag indicates C-value data should be printed."),
                )
                .arg(
                    Arg::with_name("karyotype")
                        .short("k")
                        .long("karyotype")
                        .help("This flag indicates karyotype data should be printed."),
                )
                .arg(
                    Arg::with_name("genome-size")
                        .short("g")
                        .long("genome-size")
                        .help("This flag indicates genome size data should be printed."),
                )
                .arg(
                    Arg::with_name("url")
                        .short("u")
                        .long("url")
                        .help("Print the underlying GoaT API URL. Useful for debugging."),
                )
                .arg(
                    Arg::with_name("phylogeny")
                        .short("p")
                        .long("phylogeny")
                        .help("Get information for all descendents of a common ancestor."),
                )
                .arg(
                    Arg::with_name("busco")
                        .short("b")
                        .long("busco")
                        .help("Include BUSCO estimates?"),
                )
                .arg(
                    Arg::with_name("size")
                        .long("size")
                        .default_value("50")
                        .help("The number of results to return."),
                )
                .arg(
                    Arg::with_name("ranks")
                        .long("ranks")
                        .possible_values(&["none", "subspecies", "species", "genus", "family", "order", "class", "phylum", "kingdom", "superkingdom"])
                        .default_value("none")
                        .help("Choose a rank to display with the results. All values up to the given rank are displayed.")
                ),
        )
        .subcommand(
            clap::SubCommand::with_name("count")
                .about("Query the count API.")
        )
        .get_matches();

    let subcommand = matches.subcommand();
    match subcommand.0 {
        "search" => {
            let matches = subcommand.1.unwrap();
            run::search(&matches).await?;
        }
        "count" => {
            bail!(goat::error::error::NotYetImplemented::CLIError);
            // call the count function here
            // count();
        }
        _ => {
            bail!(goat::error::error::NotYetImplemented::CLIError)
        }
    }

    Ok(())
}
