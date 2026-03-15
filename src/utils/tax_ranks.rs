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

#[cfg(test)]
mod tests {
    use super::*;

    // ── search mode (report = false) ─────────────────────────────────────────

    #[test]
    fn test_valid_rank_returns_url_segment() {
        let tr = TaxRanks::init();
        let result = tr.parse("species", false).unwrap();
        assert!(result.contains("tax_rank"));
        assert!(result.contains("species"));
        assert!(result.starts_with("%20AND%20tax_rank%28"));
        assert!(result.ends_with("%29"));
    }

    #[test]
    fn test_multiple_ranks_comma_separated() {
        let tr = TaxRanks::init();
        let result = tr.parse("species,genus", false).unwrap();
        assert!(result.contains("species"));
        assert!(result.contains("genus"));
        // both should be URL-encoded inside the same tax_rank() call
        assert!(result.contains("%2C"));
    }

    #[test]
    fn test_invalid_rank_search_mode_returns_err() {
        let tr = TaxRanks::init();
        let result = tr.parse("notarank", false);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_rank_in_list_returns_err() {
        let tr = TaxRanks::init();
        let result = tr.parse("species,notarank", false);
        assert!(result.is_err());
    }

    // ── report mode (report = true) ──────────────────────────────────────────

    #[test]
    fn test_report_mode_valid_rank_returns_plain_string() {
        let tr = TaxRanks::init();
        let result = tr.parse("genus", true).unwrap();
        assert_eq!(result, "genus");
    }

    #[test]
    fn test_report_mode_invalid_rank_returns_err() {
        let tr = TaxRanks::init();
        let result = tr.parse("notarank", true);
        assert!(result.is_err());
    }

    #[test]
    fn test_all_canonical_ranks_are_valid() {
        let tr = TaxRanks::init();
        for rank in tr.ranks.iter() {
            assert!(
                tr.parse(rank, false).is_ok(),
                "rank '{}' should be valid",
                rank
            );
        }
    }
}
