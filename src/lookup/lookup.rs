use crate::utils::url::{GOAT_URL, TAXONOMY};
use crate::utils::utils::{parse_multiple_taxids, some_kind_of_uppercase_first_letter};

// this struct will contain all the lookup data
// but only from the first (best) hit.
use anyhow::{bail, Result};
use futures::StreamExt;
use reqwest;
use reqwest::header::ACCEPT;
use serde_json::Value;

#[derive(Clone, Debug)]
pub struct Lookup {
    // the user search
    pub search: String,
}

// an example
// https://goat.genomehubs.org/api/v0.0.1/lookup?searchTerm=english%20oak&result=taxon&taxonomy=ncbi

impl Lookup {
    pub fn make_url(&self) -> String {
        let mut url = String::new();
        // add the base
        url += &GOAT_URL;
        // add lookup
        url += &"lookup?";
        // add the search term
        let search_term = format!("searchTerm={}", self.search);
        url += &search_term;
        // hardcode the rest for now
        url += &format!("&result=taxon&taxonomy={}", &*TAXONOMY);
        url
    }
}

pub struct Lookups {
    pub entries: Vec<Lookup>,
}

// throw warnings if there are no hits
impl Lookups {
    fn new<'a>(matches: &clap::ArgMatches<'a>) -> Self {
        // currently the only flag.
        // unwrap should be safe, as it's caught by clap otherwise
        let tax_name = matches.value_of("taxon").unwrap();
        let tax_name_vector = parse_multiple_taxids(tax_name);

        let mut res = Vec::new();

        for el in tax_name_vector {
            res.push(Lookup { search: el })
        }

        Self { entries: res }
    }

    // make urls, these are slightly different, and simpler than those
    // made for the main search program

    pub fn make_urls(&self) -> Vec<String> {
        let mut url_vector = Vec::new();
        for el in &self.entries {
            // println!("{}", el.make_url())
            url_vector.push(el.make_url());
        }
        url_vector
    }
}

#[derive(Clone)]
pub struct Collector {
    // the user search
    pub search: Option<String>,
    // the taxon id, that we fetch
    pub taxon_id: Option<String>,
    // the taxon rank
    pub taxon_rank: Option<String>,
    // maybe a map of name: class pairs? Might be empty
    pub taxon_names: Option<Vec<(String, String)>>,
    // suggestion.
    pub suggestions: Option<Vec<Option<String>>>,
}

impl Collector {
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
                            Ok(eprintln!("Did you mean: {}?", suggestion_str))
                        } else {
                            Ok(eprintln!("There are no results."))
                        }
                    }
                    // no suggestion, so we got a hit
                    None => {
                        // let taxon_rank = match &self.taxon_rank {
                        //     Some(t) => t,
                        //     None => bail!("No results found."),
                        // };
                        let taxon_id = match &self.taxon_id {
                            Some(t) => t,
                            None => "No taxon ID",
                        };
                        if index == 0 {
                            println!("taxon\tsearch_query\tname\ttype");
                        }
                        match &self.taxon_names {
                            Some(n) => {
                                // this may not be the best way to print
                                // as everything has to be loaded into mem
                                // however, each result string should be small.
                                let mut whole_res_string = String::new();
                                let mut peekable_iter = n.iter().peekable();
                                while let Some(el) = peekable_iter.next() {
                                    if peekable_iter.peek().is_some() {
                                        let row = format!(
                                            "{}\t{}\t{}\t{}\n",
                                            taxon_id, search, el.0, el.1
                                        );
                                        whole_res_string += &row;
                                    } else {
                                        let row =
                                            format!("{}\t{}\t{}\t{}", taxon_id, search, el.0, el.1);
                                        whole_res_string += &row;
                                    }
                                }

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
    // another function for internal use should probably go here.
    // take matches from search cli
    // generate ncbi taxid
    // give user warning for spelling mistakes.
    // ncbi taxid can then be passed 
}

pub async fn lookup<'a>(matches: &clap::ArgMatches<'a>, _cli: bool) -> Result<()> {
    let lookups = Lookups::new(matches);
    let url_vector_api = lookups.make_urls();
    let print_url = matches.is_present("url");

    if print_url {
        for (index, url) in url_vector_api.iter().enumerate() {
            println!("{}.\tGoaT lookup API URL: {}", index, url);
        }
        std::process::exit(0);
    }
    // so we can make as many concurrent requests
    let concurrent_requests = url_vector_api.len();

    let fetches = futures::stream::iter(url_vector_api.into_iter().map(|path| async move {
        // possibly make a again::RetryPolicy
        // to catch all the values in a *very* large request.
        let client = reqwest::Client::new();

        match again::retry(|| client.get(&path).header(ACCEPT, "application/json").send()).await {
            Ok(resp) => match resp.text().await {
                Ok(body) => {
                    let v: Value = serde_json::from_str(&body)?;

                    // this bit is a bit horrible.
                    // get the suggestions first
                    // suggestion search will be the same for each element in the array
                    let suggestion_search = &v["suggestions"][0]["text"].as_str();
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
                    let taxon_search = &v["results"][0]["result"]["scientific_name"].as_str();
                    let taxon_id = &v["results"][0]["result"]["taxon_id"].as_str();
                    let taxon_rank = &v["results"][0]["result"]["taxon_rank"].as_str();
                    let taxon_names_array_op = &v["results"][0]["result"]["taxon_names"].as_array();

                    let taxon_names_array = match taxon_names_array_op {
                        Some(vec) => {
                            let mut collect_names = Vec::new();
                            for el in vec.into_iter() {
                                let key = el["name"].as_str().unwrap_or("-");
                                let value = el["class"].as_str().unwrap_or("-");
                                collect_names.push((key.to_string(), value.to_string()));
                            }
                            Some(collect_names)
                        }
                        None => None,
                    };
                    // sort out the search name
                    let search;
                    match suggestion_search {
                        Some(s) => search = s,
                        None => match taxon_search {
                            Some(s) => search = s,
                            None => search = &"No match",
                        },
                    }
                    // clone the relevant bits
                    let tax_id = taxon_id.clone();
                    let taxon_rank = taxon_rank.clone().map(String::from);
                    let taxon_id = tax_id.clone().map(String::from);

                    Ok(Collector {
                        search: Some(search.to_string()),
                        suggestions: suggestions_text,
                        taxon_id,
                        taxon_names: taxon_names_array,
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

    for (index, el) in awaited_fetches.iter().enumerate() {
        match el {
            Ok(e) => e.print_result(index)?,
            Err(_) => bail!("No results found."),
        }
    }

    Ok(())
}
