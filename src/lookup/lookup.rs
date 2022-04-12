use crate::utils::cli_matches::UPPER_CLI_FILE_LIMIT;
use crate::utils::url::{GOAT_URL, TAXONOMY};
use crate::utils::utils::{
    lines_from_file, parse_comma_separated, some_kind_of_uppercase_first_letter,
};

use anyhow::{bail, Result};
use futures::StreamExt;
use reqwest;
use reqwest::header::ACCEPT;
use serde_json::Value;

/// The lookup struct
#[derive(Clone, Debug)]
pub struct Lookup {
    /// the users search
    pub search: String,
    /// The size for each search (default = 10)
    pub size: u64,
}

// an example
// https://goat.genomehubs.org/api/v0.0.1/lookup?searchTerm=english%20oak&result=taxon&taxonomy=ncbi

impl Lookup {
    /// From our lookup struct we can make an individual URL.
    pub fn make_url(&self) -> String {
        let mut url = String::new();
        // add the base
        url += &GOAT_URL;
        // add lookup
        url += &"lookup?";
        // add the search term
        let search_term = format!("searchTerm={}", self.search);
        url += &search_term;
        // add size
        let size = format!("&size={}", self.size);
        url += &size;
        // hardcode the rest for now
        url += &format!("&result=taxon&taxonomy={}", &*TAXONOMY);
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
    fn new(matches: &clap::ArgMatches) -> Result<Self> {
        let tax_name_op = matches.value_of("taxon");
        let filename_op = matches.value_of("file");
        // safe to unwrap, as default is defined.
        let no_hits = matches.value_of("size").unwrap();
        let no_hits = no_hits.parse::<u64>().unwrap_or(10);

        let tax_name_vector: Vec<String>;
        match tax_name_op {
            Some(s) => tax_name_vector = parse_comma_separated(s),
            None => match filename_op {
                Some(s) => {
                    tax_name_vector = lines_from_file(s)?;
                    // check length of vector and bail if > 1000
                    if tax_name_vector.len() > *UPPER_CLI_FILE_LIMIT {
                        bail!(
                            "[-]\tNumber of taxa specified cannot exceed {}.",
                            *UPPER_CLI_FILE_LIMIT
                        )
                    }
                }
                None => bail!("[-]\tOne of -f (--file) or -t (--tax-id) should be specified."),
            },
        }

        let mut res = Vec::new();

        for el in tax_name_vector {
            res.push(Lookup {
                search: el,
                size: no_hits,
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

/// Collect the results from concurrent `goat-cli lookup`
/// queries.
#[derive(Clone)]
pub struct Collector {
    /// User search value.
    pub search: Option<String>,
    /// The taxon id that we fetch.
    /// Can return multiple taxon id's.
    pub taxon_id: Vec<Option<String>>,
    /// The taxon rank.
    pub taxon_rank: Vec<Option<String>>,
    /// A vector of optional taxon names.
    pub taxon_names: Vec<Option<Vec<(String, String)>>>,
    /// The suggestions vector.
    pub suggestions: Option<Vec<Option<String>>>,
}

impl Collector {
    /// Print the result from a collector struct.
    /// add an index, so we don't repeat headers
    pub fn print_result(&self, index: usize) -> Result<()> {
        // if we got a hit
        match &self.search {
            Some(search) => {
                // if we got a suggestion
                match &self.suggestions {
                    // we end up here even if there are no *actual* suggestions.
                    Some(suggestions) => {
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
                            Ok(eprintln!("[-]\tDid you mean: {}?", suggestion_str))
                        } else {
                            Ok(eprintln!("[-]\tThere are no results."))
                        }
                    }
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
                                let mut zipped_taxon_vectors =
                                    taxon_ids.iter().zip(taxon_ranks.iter()).zip(n.iter());

                                // this may not be the best way to print
                                // as everything has to be loaded into mem
                                // however, each result string should be small.
                                let mut whole_res_string = String::new();

                                while let Some(((taxon_id, taxon_rank), taxon_ranks)) =
                                    zipped_taxon_vectors.next()
                                {
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
                            None => Ok(eprintln!("There were no taxon names.")),
                        }
                    }
                }
            }
            None => Ok(eprintln!("No results.")),
        }
    }
    // take matches from search cli
    // generate ncbi taxid
    // give user warning for spelling mistakes.
    // ncbi taxid can then be passed
    // the output type must be Option<String> (= taxid in `search` & `count`)

    // currently deprecated - don't use this in `search` or `count`
    // there is the possibility of URL's being too long...
    // might be useful for `record` API at some point?

    #[deprecated(
        note = "Currently deprecated. Might be used in future to suggest spelling corrections in search."
    )]
    pub fn return_taxid_vec(&self) -> Result<Option<String>> {
        // if we got a hit
        match &self.search {
            Some(search) => {
                // if we got a suggestion
                match &self.suggestions {
                    // we end up here even if there are no *actual* suggestions.
                    Some(suggestions) => {
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
                            eprintln!(
                                "[-]\tYou searched {}. Did you mean: {}?",
                                search, suggestion_str
                            );
                            // no taxid here
                            Ok(None)
                        } else {
                            eprintln!("[-]\tThere are no results for the search: {}", search);
                            Ok(None)
                        }
                    }
                    // no suggestion, so we got a hit
                    None => {
                        let taxon_id = self.taxon_id.clone();
                        let taxon_ids_op: Option<Vec<String>> = taxon_id.into_iter().collect();

                        let taxon_id = match taxon_ids_op {
                            Some(t) => t.join("%2C"),
                            None => "No taxon ID".to_string(),
                        };
                        Ok(Some(taxon_id))
                    }
                }
            }
            None => bail!("No results."),
        }
    }
}

/// Main entry point for `goat-cli lookup`.
pub async fn lookup(matches: &clap::ArgMatches, cli: bool) -> Result<Option<Vec<String>>> {
    let lookups = Lookups::new(matches)?;
    let url_vector_api = lookups.make_urls();
    let print_url = matches.is_present("url");
    let size = matches.value_of("size").unwrap();

    if print_url {
        for (index, (url, _)) in url_vector_api.iter().enumerate() {
            println!("{}.\tGoaT lookup API URL: {}", index, url);
        }
        // don't exit here internally; we'll exit later
        if cli {
            std::process::exit(0);
        }
    }
    // so we can make as many concurrent requests
    let concurrent_requests = url_vector_api.len();

    let fetches = futures::stream::iter(url_vector_api.into_iter().map(|(path, search_query)| async move {
        // possibly make a again::RetryPolicy
        // to catch all the values in a *very* large request.
        let client = reqwest::Client::new();

        match again::retry(|| client.get(&path).header(ACCEPT, "application/json").send()).await {
            Ok(resp) => match resp.text().await {
                Ok(body) => {
                    let v: Value = serde_json::from_str(&body)?;
                    // print a warning if number of hits > size specified.
                    let request_size_op = &v["status"]["hits"].as_u64();
                    match request_size_op {
                        Some(s) => {
                            if size.parse::<u64>()? < *s {
                                eprintln!(
                                "[-]\tFor seach query {}, size specified ({}) was less than the number of results returned, ({}).",
                                search_query, size, s
                            )
                        }
                    },
                        None => (),
                    }

                    // get all the suggestions
                    let suggestions_text_op = &v["suggestions"].as_array();
                    // collect into a vec
                    let mut suggestions_vec = Vec::new();
                    let suggestions_text = match suggestions_text_op {
                        Some(suggestions) => {
                            for el in *suggestions {
                                let sug_str = el["suggestion"]["text"].as_str();
                                let sug_string_op = sug_str.map(String::from);
                                suggestions_vec.push(sug_string_op);
                            }
                            Some(suggestions_vec.clone())
                        }
                        None => None,
                    };
                    // and the taxon ID
                    // we need to iterate over the array of results.
                    // potentially look at the scores, and keep those over a certain amount
                    // or keep everything. Currently I'm missing synonymous genera.

                    let mut taxon_id_vec = Vec::new();
                    let mut taxon_rank_vec = Vec::new();
                    let mut taxon_names_array_vec = Vec::new();

                    let results_array = v["results"].as_array();
                    // unwrap safely here
                    match results_array {
                        Some(arr) => {
                            for el in arr {
                                let taxon_id = el["result"]["taxon_id"].as_str();
                                let taxon_rank = el["result"]["taxon_rank"].as_str();
                                let taxon_names_array_op = el["result"]["taxon_names"].as_array();

                                let taxon_names_array = match taxon_names_array_op {
                                    Some(vec) => {
                                        let mut collect_names = Vec::new();
                                        for el in vec.into_iter() {
                                            let key = el["name"].as_str().unwrap_or("-");
                                            let value = el["class"].as_str().unwrap_or("-");
                                            collect_names
                                                .push((key.to_string(), value.to_string()));
                                        }
                                        Some(collect_names)
                                    }
                                    None => None,
                                };

                                // gather results into the vecs
                                taxon_id_vec.push(taxon_id);
                                taxon_rank_vec.push(taxon_rank);
                                taxon_names_array_vec.push(taxon_names_array);
                            }
                        }
                        None => {}
                    }

                    // Vec<Option<&str>> -> Vec<Option<String>>
                    let taxon_id = taxon_id_vec.iter().map(|e| e.map(String::from)).collect();
                    let taxon_rank = taxon_rank_vec.iter().map(|e| e.map(String::from)).collect();

                    Ok(Collector {
                        search: Some(search_query.to_string()),
                        suggestions: suggestions_text,
                        taxon_id,
                        taxon_names: taxon_names_array_vec,
                        taxon_rank,
                    })
                }
                Err(_) => bail!("ERROR reading {}", path),
            },
            Err(_) => bail!("ERROR downloading {}", path),
        }
    }))
    .buffer_unordered(concurrent_requests)
    .collect::<Vec<_>>();

    let awaited_fetches = fetches.await;

    let mut return_taxid_vec: Vec<String> = Vec::new();

    for (index, el) in awaited_fetches.iter().enumerate() {
        match el {
            Ok(e) => {
                if cli {
                    e.print_result(index)?;
                } else {
                    // dead code currently
                    let taxid_op = e.return_taxid_vec()?;
                    match taxid_op {
                        Some(s) => return_taxid_vec.push(s),
                        // the None variant can't push a "",
                        // otherwise the URL hangs.
                        None => return_taxid_vec.push("-".to_string()),
                    }
                }
            }
            Err(_) => bail!("No results found."),
        }
    }

    Ok(Some(return_taxid_vec))
}
