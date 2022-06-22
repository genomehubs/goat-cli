use crate::utils::utils;
use crate::{GOAT_URL, TAXONOMY};
use anyhow::{bail, Result};

// might re-jig this later,
// enumerate types to return
// currently only newick implemented

/// The record type to return. Currently only
/// Newick is supported.
pub enum ReportType {
    Newick,
}

/// The record struct to make URLs from.
pub struct Report {
    /// A vector of taxon ID's/names.
    pub search: Vec<String>,
    /// The rank of the return type.
    pub rank: String,
    ///
    pub url: bool,
}

impl Report {
    /// Constructor function for [`Record`].
    pub fn new(matches: &clap::ArgMatches) -> Result<Self> {
        // simply return the populated struct
        // taxon, url, rank
        let search_op = matches.value_of("taxon");
        let url = matches.is_present("url");
        // safe to unwrap, as default is defined.
        let rank = matches.value_of("rank").unwrap().to_string();

        // turn tax_name_op into a vector of taxon names
        let search = match search_op {
            Some(s) => utils::parse_comma_separated(s),
            None => bail!("There was no taxon input."),
        };

        Ok(Self { search, rank, url })
    }

    /// Make the URL. Currently only [`RecordType::Newick`] supported.
    /// Also return the unique identifier so a progress bar can be made.
    pub fn make_url(&self, record_type: ReportType, unique_ids: Vec<String>) -> String {
        match record_type {
            ReportType::Newick => {
                let mut url = String::new();
                url += &GOAT_URL;
                // add report API, and result=taxon
                url += &"report?result=taxon";
                // it's a tree we're returning
                url += &"&report=tree";
                // get a string of comma separated queries
                let csqs = match self.search.len() {
                    // one entry
                    1 => self.search[0].clone(),
                    // or greater (zero entries handled by cli)
                    _ => self.search.join("%2C"),
                };
                // the x value source
                let x_value_source = format!(
                    "&x=tax_rank%28{}%29%20AND%20tax_tree%28{}%29",
                    self.rank, csqs
                );
                url += &x_value_source;
                // default stuff for now
                url += &"&includeEstimates=true";
                url += &format!("&taxonomy={}", &*TAXONOMY);
                // fix this for now, as only single requests can be submitted
                url += &format!("&queryId=goat_cli_{}", unique_ids[0]);
                url
            }
        }
    }
}
