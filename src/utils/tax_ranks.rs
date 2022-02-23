// add tax ranks to URL queries

use anyhow::{bail, Result};
use std::fmt;

use crate::utils::utils;

// taken from the NCBI taxdump
// on date: 22.02.22
const TAX_RANKS: &[&str; 44] = &[
    "biotype",
    "clade",
    "class",
    "cohort",
    "family",
    "forma",
    "forma specialis",
    "genotype",
    "genus",
    "infraclass",
    "infraorder",
    "isolate",
    "kingdom",
    "morph",
    "no rank",
    "order",
    "parvorder",
    "pathogroup",
    "phylum",
    "section",
    "series",
    "serogroup",
    "serotype",
    "species",
    "species group",
    "species subgroup",
    "strain",
    "subclass",
    "subcohort",
    "subfamily",
    "subgenus",
    "subkingdom",
    "suborder",
    "subphylum",
    "subsection",
    "subspecies",
    "subtribe",
    "superclass",
    "superfamily",
    "superkingdom",
    "superorder",
    "superphylum",
    "tribe",
    "varietas",
];

// we only really need to do two things
// check if user has input a real tax rank
// and display the tax ranks
pub struct TaxRanks<'a> {
    pub ranks: &'a [&'a str; 44],
}

impl<'a> TaxRanks<'a> {
    // shove the tax ranks where they should be
    pub fn init() -> Self {
        Self { ranks: TAX_RANKS }
    }
    // take the string from the cli
    // which may contain commas
    // and convert it to a URL encoded string of tax ranks.
    pub fn parse(&self, cmp: &str) -> Result<String> {
        // split the string on commas
        let split = utils::parse_comma_separated(cmp);
        let mut tax_rank_string = String::new();
        // space and AND tax_rank open parentheses
        tax_rank_string += "%20AND%20tax_rank%28";
        let mut ranks_vec = Vec::new();

        // iterate over split
        for el in split {
            if self.ranks.contains(&&el[..]) {
                ranks_vec.push(el);
            } else {
                bail!(
                    "Taxonomic rank \"{}\" is not recognised.\n\nEnter one of: {}",
                    el,
                    Self::init()
                );
            }
        }

        tax_rank_string += &ranks_vec.join("%2C");
        // close parentheses
        tax_rank_string += "%29";

        Ok(tax_rank_string)
    }
}

// I think a comma separated list is okay?
impl<'a> fmt::Display for TaxRanks<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Use `self.number` to refer to each positional data point.
        let tax_ranks_csv = self.ranks.join(", ");
        write!(f, "{}", tax_ranks_csv)
    }
}
