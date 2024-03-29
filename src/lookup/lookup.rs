use crate::error::{Error, ErrorKind, Result};
use crate::utils::utils::{
    lines_from_file, parse_comma_separated, some_kind_of_uppercase_first_letter,
};
use crate::{IndexType, GOAT_URL, TAXONOMY, UPPER_CLI_FILE_LIMIT};
use std::path::PathBuf;

/// The lookup struct
#[derive(Clone, Debug)]
pub struct Lookup {
    /// the users search
    pub search: String,
    /// The size for each search (default = 10)
    pub size: u64,
    /// The index type, currently taxon or
    /// assembly
    pub index_type: IndexType,
}

impl Lookup {
    /// From our lookup struct we can make an individual URL.
    pub fn make_url(&self) -> String {
        let mut url = String::new();
        // add the base
        url += &GOAT_URL;
        // add lookup
        url += "lookup?";
        // add the search term
        let search_term = format!("searchTerm={}", self.search);
        url += &search_term;
        // add size
        let size = format!("&size={}", self.size);
        url += &size;
        // hardcode the rest for now
        url += &format!("&result={}&taxonomy={}", self.index_type, &*TAXONOMY);
        url
    }
}

/// A vector of [`Lookup`] structs.
#[derive(Debug)]
pub struct Lookups {
    /// The entries in [`Lookups`].
    pub entries: Vec<Lookup>,
}

// throw warnings if there are no hits
impl Lookups {
    /// Constructor which takes the CLI args and returns
    /// `Self`.
    pub fn new(matches: &clap::ArgMatches, index_type: IndexType) -> Result<Self> {
        let tax_name_op = matches.get_one::<String>("taxon");
        let filename_op = matches.get_one::<PathBuf>("file");
        // safe to unwrap, as default is defined.
        let no_hits = *matches.get_one::<u64>("size").expect("cli default = 10");

        let tax_name_vector: Vec<String>;
        match tax_name_op {
            Some(s) => tax_name_vector = parse_comma_separated(s),
            None => match filename_op {
                Some(s) => {
                    tax_name_vector = lines_from_file(s)?;
                    // check length of vector and bail if > 1000
                    if tax_name_vector.len() > *UPPER_CLI_FILE_LIMIT {
                        return Err(Error::new(ErrorKind::GenericCli(format!(
                            "number of taxa specified cannot exceed {}.",
                            *UPPER_CLI_FILE_LIMIT
                        ))));
                    }
                }
                None => {
                    return Err(Error::new(ErrorKind::GenericCli(format!(
                        "one of -f (--file) or -t (--taxon) should be specified."
                    ))))
                }
            },
        }

        let mut res = Vec::new();

        for el in tax_name_vector {
            res.push(Lookup {
                search: el,
                size: no_hits,
                index_type,
            })
        }

        Ok(Self { entries: res })
    }

    // make urls, these are slightly different, and simpler than those
    // made for the main search program

    /// Make URLs calls [`Lookup::make_url`] on each element.
    pub fn make_urls(&self) -> Vec<(String, String)> {
        let mut url_vector = Vec::new();
        for el in &self.entries {
            let id = el.search.clone();
            url_vector.push((el.make_url(), id));
        }
        url_vector
    }
}

/// Took this out of `print_result` as
fn format_suggestion_string(suggestions: &Vec<Option<String>>) -> Result<()> {
    let mut suggestion_str = String::new();
    for el in suggestions {
        match el {
            Some(s) => {
                suggestion_str += &some_kind_of_uppercase_first_letter(&s[..]);
                suggestion_str += ", ";
            }
            None => {}
        }
    }
    // remove last comma
    if suggestion_str.len() > 2 {
        suggestion_str.drain(suggestion_str.len() - 2..);
        return Err(Error::new(ErrorKind::GenericCli(format!(
            "did you mean: {}?",
            suggestion_str
        ))));
    } else {
        return Err(Error::new(ErrorKind::GenericCli(format!(
            "there are no results."
        ))));
    }
}

/// Collect the results from concurrent `goat-cli taxon lookup`
/// queries.
#[derive(Clone)]
pub struct TaxonCollector {
    /// User search value.
    pub search: Option<String>,
    /// The taxon id that we fetch.
    /// Can return multiple taxon id's.
    pub taxon_id: Vec<Option<String>>,
    /// The taxon rank.
    pub taxon_rank: Vec<Option<String>>,
    /// A vector of optional taxon names.
    ///
    /// Decomposed this is a vector of Some vector of a
    /// two-element tuple:
    /// - The name of the taxon
    /// - The class of the taxon name
    pub taxon_names: Vec<Option<Vec<(String, String)>>>,
    /// The suggestions vector.
    pub suggestions: Option<Vec<Option<String>>>,
}

impl TaxonCollector {
    /// Print the result from a collector struct.
    /// add an index, so we don't repeat headers
    pub fn print_result(&self, index: usize) -> Result<()> {
        // if we got a hit
        match &self.search {
            Some(search) => {
                // if we got a suggestion
                match &self.suggestions {
                    // we end up here even if there are no *actual* suggestions.
                    Some(suggestions) => format_suggestion_string(suggestions),
                    // no suggestion, so we got a hit
                    None => {
                        // Vec<Option<String>> -> Option<Vec<String>>
                        // these vecs should all be the same length?
                        let taxon_id = self.taxon_id.clone();
                        let taxon_rank = self.taxon_rank.clone();
                        let taxon_names = self.taxon_names.clone();

                        let taxon_ids_op: Option<Vec<String>> = taxon_id.into_iter().collect();
                        let taxon_ranks_op: Option<Vec<String>> = taxon_rank.into_iter().collect();
                        // same but for nested vec.
                        let taxon_names_op: Option<Vec<Vec<(String, String)>>> =
                            taxon_names.into_iter().collect();

                        // print headers for first result only.
                        if index == 0 {
                            println!("taxon\trank\tsearch_query\tname\ttype");
                        }
                        match taxon_names_op {
                            Some(n) => {
                                // get taxon_ids and taxon_ranks
                                let taxon_ids = match taxon_ids_op {
                                    Some(t) => t,
                                    // empty vec
                                    None => vec![],
                                };
                                let taxon_ranks = match taxon_ranks_op {
                                    Some(t) => t,
                                    // empty vec
                                    None => vec![],
                                };
                                // zip these vectors together
                                let zipped_taxon_vectors =
                                    taxon_ids.iter().zip(taxon_ranks.iter()).zip(n.iter());

                                // this may not be the best way to print
                                // as everything has to be loaded into mem
                                // however, each result string should be small.
                                let mut whole_res_string = String::new();

                                for ((taxon_id, taxon_rank), taxon_ranks) in zipped_taxon_vectors {
                                    for el in taxon_ranks {
                                        let row = format!(
                                            "{}\t{}\t{}\t{}\t{}\n",
                                            taxon_id, taxon_rank, search, el.0, el.1
                                        );
                                        whole_res_string += &row;
                                    }
                                }
                                // remove trailing newline
                                whole_res_string.pop();
                                Ok(println!("{}", whole_res_string))
                            }
                            None => {
                                return Err(Error::new(ErrorKind::GenericCli(format!(
                                    "there were no taxon names."
                                ))))
                            }
                        }
                    }
                }
            }
            None => return Err(Error::new(ErrorKind::GenericCli(format!("no results.")))),
        }
    }
}

/// Collect the results from concurrent `goat-cli taxon lookup`
/// queries.
#[derive(Clone)]
pub struct AssemblyCollector {
    /// User search value.
    pub search: Option<String>,
    /// The taxon id that we fetch.
    /// Can return multiple taxon id's.
    pub taxon_id: Vec<Option<String>>,
    /// The identifiers, which is an enumeration of all
    /// of the identifier:class pairs. This could be a Map.
    pub identifiers: Vec<Option<Vec<(String, String)>>>,
    /// The suggestions vector.
    pub suggestions: Option<Vec<Option<String>>>,
}

impl AssemblyCollector {
    /// Print the result from a collector struct.
    /// add an index, so we don't repeat headers
    pub fn print_result(&self, index: usize) -> Result<()> {
        // if we got a hit
        match &self.search {
            Some(search) => {
                // if we got a suggestion
                match &self.suggestions {
                    // we end up here even if there are no *actual* suggestions.
                    Some(suggestions) => format_suggestion_string(suggestions),
                    // no suggestion, so we got a hit
                    None => {
                        // Vec<Option<String>> -> Option<Vec<String>>
                        // these vecs should all be the same length?
                        let taxon_id = self.taxon_id.clone();
                        let assembly_identifiers = self.identifiers.clone();

                        let taxon_ids_op: Option<Vec<String>> = taxon_id.into_iter().collect();
                        // same but for nested vec.
                        let assembly_identifiers_op: Option<Vec<Vec<(String, String)>>> =
                            assembly_identifiers.into_iter().collect();

                        // print headers for first result only.
                        if index == 0 {
                            println!("taxon\tsearch_query\tidentifier\ttype");
                        }
                        match assembly_identifiers_op {
                            Some(n) => {
                                // get taxon_ids and taxon_ranks
                                let taxon_ids = match taxon_ids_op {
                                    Some(t) => t,
                                    // empty vec
                                    None => vec![],
                                };
                                // zip these vectors together
                                let zipped_taxon_vectors = taxon_ids.iter().zip(n.iter());

                                // this may not be the best way to print
                                // as everything has to be loaded into mem
                                // however, each result string should be small.
                                let mut whole_res_string = String::new();

                                for (taxon_id, taxon_ranks) in zipped_taxon_vectors {
                                    for el in taxon_ranks {
                                        let row = format!(
                                            "{}\t{}\t{}\t{}\n",
                                            taxon_id, search, el.0, el.1
                                        );
                                        whole_res_string += &row;
                                    }
                                }
                                // remove trailing newline
                                whole_res_string.pop();
                                Ok(println!("{}", whole_res_string))
                            }
                            None => {
                                return Err(Error::new(ErrorKind::GenericCli(format!(
                                    "there were no taxon names."
                                ))))
                            }
                        }
                    }
                }
            }
            None => {
                return Err(Error::new(ErrorKind::GenericCli(format!(
                    "there are no results."
                ))))
            }
        }
    }
}

/// A wrapper so we can return the same from our request.
/// Otherwise I am going to have to do extensive changes above
/// which I decided against.
pub enum Collector {
    /// The taxon results.
    Taxon(TaxonCollector),
    /// The assembly results.
    Assembly(AssemblyCollector),
}
