use crate::error::Result;
use crate::cli::LookupArgs;
use crate::utils::url::percent_encode_query_value;
use crate::utils::utils::{some_kind_of_uppercase_first_letter, taxa_from_input};
use crate::{IndexType, GOAT_URL, TAXONOMY};
use url::Url;

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
        let base = format!("{}lookup", *GOAT_URL);
        let mut url = Url::parse(&base).expect("GOAT_URL is a valid base");
        let qp = format!(
            "searchTerm={}&size={}&result={}&taxonomy={}",
            percent_encode_query_value(&self.search),
            self.size,
            self.index_type,
            *TAXONOMY,
        );
        url.set_query(Some(&qp));
        url.to_string()
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
    pub fn new(args: &LookupArgs, index_type: IndexType) -> Result<Self> {
        let no_hits = args.size;
        let tax_name_vector = taxa_from_input(args.taxon.as_deref(), args.file.as_deref(), false)?;

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

/// Tell the user (on stderr) that a search had no hits, with any
/// suggestions, so the rest of a batch can carry on.
fn print_no_results(search: &str, suggestions: &[Option<String>]) {
    let suggestions = suggestions
        .iter()
        .flatten()
        .map(|s| some_kind_of_uppercase_first_letter(s))
        .collect::<Vec<_>>();
    if suggestions.is_empty() {
        eprintln!("No results for \"{}\".", search);
    } else {
        eprintln!(
            "No results for \"{}\" - did you mean: {}?",
            search,
            suggestions.join(", ")
        );
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
    /// The TSV header for [`TaxonCollector::print_result`].
    pub const HEADER: &'static str = "taxon\trank\tsearch_query\tname\ttype";

    /// Print the rows for this result. A search without hits is reported
    /// on stderr, rather than as an error, so other searches still print.
    pub fn print_result(&self) -> Result<()> {
        let search = self.search.as_deref().unwrap_or_default();
        // GoaT only returns suggestions when there are no hits
        if let Some(suggestions) = &self.suggestions {
            print_no_results(search, suggestions);
            return Ok(());
        }

        let mut rows = String::new();
        for ((taxon_id, taxon_rank), taxon_names) in self
            .taxon_id
            .iter()
            .zip(self.taxon_rank.iter())
            .zip(self.taxon_names.iter())
        {
            let (Some(taxon_id), Some(taxon_rank), Some(taxon_names)) =
                (taxon_id, taxon_rank, taxon_names)
            else {
                continue;
            };
            for (name, class) in taxon_names {
                rows += &format!("{}\t{}\t{}\t{}\t{}\n", taxon_id, taxon_rank, search, name, class);
            }
        }
        if rows.is_empty() {
            print_no_results(search, &[]);
            return Ok(());
        }
        crate::outln!("{}", rows.trim_end_matches('\n'))?;
        Ok(())
    }
}

/// Collect the results from concurrent `goat-cli assembly lookup`
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
    /// The TSV header for [`AssemblyCollector::print_result`].
    pub const HEADER: &'static str = "taxon\tsearch_query\tidentifier\ttype";

    /// Print the rows for this result. A search without hits is reported
    /// on stderr, rather than as an error, so other searches still print.
    pub fn print_result(&self) -> Result<()> {
        let search = self.search.as_deref().unwrap_or_default();
        // GoaT only returns suggestions when there are no hits
        if let Some(suggestions) = &self.suggestions {
            print_no_results(search, suggestions);
            return Ok(());
        }

        let mut rows = String::new();
        for (taxon_id, identifiers) in self.taxon_id.iter().zip(self.identifiers.iter()) {
            let (Some(taxon_id), Some(identifiers)) = (taxon_id, identifiers) else {
                continue;
            };
            for (identifier, class) in identifiers {
                rows += &format!("{}\t{}\t{}\t{}\n", taxon_id, search, identifier, class);
            }
        }
        if rows.is_empty() {
            print_no_results(search, &[]);
            return Ok(());
        }
        crate::outln!("{}", rows.trim_end_matches('\n'))?;
        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::IndexType;

    fn taxon_lookup(search: &str, size: u64) -> Lookup {
        Lookup {
            search: search.into(),
            size,
            index_type: IndexType::Taxon,
        }
    }

    fn assembly_lookup(search: &str, size: u64) -> Lookup {
        Lookup {
            search: search.into(),
            size,
            index_type: IndexType::Assembly,
        }
    }

    // ── Lookup::make_url ─────────────────────────────────────────────────────

    #[test]
    fn test_make_url_contains_search_term() {
        let url = taxon_lookup("Homo sapiens", 10).make_url();
        // query_pairs_mut encodes spaces as '+' (form encoding)
        assert!(url.contains("searchTerm=Homo%20sapiens"));
    }

    #[test]
    fn test_make_url_contains_size() {
        let url = taxon_lookup("Mammalia", 25).make_url();
        assert!(url.contains("size=25"));
    }

    #[test]
    fn test_make_url_taxon_result_field() {
        let url = taxon_lookup("Mammalia", 10).make_url();
        assert!(url.contains("result=taxon"));
    }

    #[test]
    fn test_make_url_assembly_result_field() {
        let url = assembly_lookup("GCA_000001405", 5).make_url();
        assert!(url.contains("result=assembly"));
    }

    #[test]
    fn test_make_url_contains_taxonomy() {
        let url = taxon_lookup("Mammalia", 10).make_url();
        assert!(url.contains("taxonomy="));
    }

    #[test]
    fn test_make_url_contains_lookup_endpoint() {
        let url = taxon_lookup("Mammalia", 10).make_url();
        assert!(url.contains("lookup?"));
    }

    // ── Lookups::make_urls ───────────────────────────────────────────────────

    #[test]
    fn test_make_urls_returns_one_per_entry() {
        let lookups = Lookups {
            entries: vec![
                taxon_lookup("Mammalia", 10),
                taxon_lookup("Aves", 10),
                taxon_lookup("Reptilia", 10),
            ],
        };
        let urls = lookups.make_urls();
        assert_eq!(urls.len(), 3);
    }

    #[test]
    fn test_make_urls_search_query_is_second_element() {
        let lookups = Lookups {
            entries: vec![
                taxon_lookup("Mammalia", 10),
                taxon_lookup("Aves", 10),
            ],
        };
        let urls = lookups.make_urls();
        assert_eq!(urls[0].1, "Mammalia");
        assert_eq!(urls[1].1, "Aves");
    }

    #[test]
    fn test_make_urls_preserves_order() {
        let taxa = vec!["Zeta", "Alpha", "Gamma"];
        let lookups = Lookups {
            entries: taxa
                .iter()
                .map(|t| taxon_lookup(t, 10))
                .collect(),
        };
        let urls = lookups.make_urls();
        for (i, taxon) in taxa.iter().enumerate() {
            assert_eq!(&urls[i].1, taxon);
        }
    }
}
