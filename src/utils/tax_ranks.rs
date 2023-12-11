use crate::error::{Error, ErrorKind, Result};
use std::fmt;

use crate::utils::utils;

/// Taken from the NCBI taxdump
/// on the date: 22.02.22.
///
/// These are all possible ranks that
/// a user can return results as.
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

/// The [`TaxRanks`] struct holds the NCBI taxon
/// ranks defined in the [`TAX_RANKS`] const.
pub struct TaxRanks<'a> {
    pub ranks: &'a [&'a str; 44],
}

impl<'a> TaxRanks<'a> {
    /// Constructor for [`TaxRanks`].
    pub fn init() -> Self {
        Self { ranks: TAX_RANKS }
    }

    /// Convert a `--tax-rank` CLI comma separated string
    /// into a URL encoded string of taxon ranks.
    ///
    /// If report == true, we are comparing a taxon rank for `report`.
    /// Was unsure whether to keep this functionality here or to split
    /// it out. Probably confusing to keep it here, but will change later.
    pub fn parse(&self, cmp: &str, report: bool) -> Result<String> {
        // if we are in the report API
        if report {
            let needle = cmp;
            if self.ranks.contains(&needle) {
                return Ok(needle.to_string());
            } else {
                return Err(Error::new(ErrorKind::TaxRank(format!(
                    "Taxonomic rank \"{}\" is not recognised.\n\nEnter one of: {}",
                    needle,
                    Self::init()
                ))));
            }
        }
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
                return Err(Error::new(ErrorKind::TaxRank(format!(
                    "taxonomic rank \"{}\" is not recognised.\n\nEnter one of: {}",
                    el,
                    Self::init()
                ))));
            }
        }

        tax_rank_string += &ranks_vec.join("%2C");
        // close parentheses
        tax_rank_string += "%29";

        Ok(tax_rank_string)
    }
}

impl<'a> fmt::Display for TaxRanks<'a> {
    /// Format [`TaxRanks`] into a comma separated list.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Use `self.number` to refer to each positional data point.
        let tax_ranks_csv = self.ranks.join(", ");
        write!(f, "{}", tax_ranks_csv)
    }
}
