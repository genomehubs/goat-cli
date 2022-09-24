//! `goat-cli` is a command line interface to query the
//! [Genomes on a Tree Open API](https://goat.genomehubs.org/api-docs/) using
//! an asynchronous [`tokio`](<https://docs.rs/tokio/latest/tokio/>) runtime.
//!
//! I'm documenting the code here for others, and for future me.

use lazy_static::lazy_static;
use std::fmt;

/// Query the GoaT count API.
pub mod count;
/// Query the GoaT lookup API.
pub mod lookup;
/// A module to produce a progress
/// bar.
pub mod progress;
/// Query the GoaT record API.
pub mod report;
/// Query the GoaT search API.
pub mod search;
/// Collection of utility functions
/// used elsewhere.
pub mod utils;

/// The base URL for GoaT.
const GOAT_URL_BASE: &str = "https://goat.genomehubs.org/api/";
/// The current version of the GoaT API.
const GOAT_API_VERSION: &str = "v2/";

lazy_static! {
    /// The current GoaT URL.
    pub static ref GOAT_URL: String = format!("{}{}", GOAT_URL_BASE, GOAT_API_VERSION);
    /// The taxonomy that `goat-cli` uses.
    pub static ref TAXONOMY: String = "ncbi".into();
}

// global size limits on pinging the API
lazy_static! {
    /// Upper limit for the CLI arg `--size`.
    pub static ref UPPER_CLI_SIZE_LIMIT: usize = 50000;
    /// Upper limit for the number of entries in the file for CLI arg `-f`.
    pub static ref UPPER_CLI_FILE_LIMIT: usize = 500;
}

/// The indexes we make searches over in GoaT.
///
/// Currently implemented (to some extent) is taxon
/// and assembly. Others exist, e.g. feature/sample.
///
/// Each tuple variant can store their respective
/// [`BTreeMap`] databases.

#[derive(Clone, Copy, Debug)]
pub enum IndexType {
    /// Taxon search index. The historical main
    /// functionality of goat-cli went through taxon.
    Taxon,
    /// Assembly search index.
    Assembly,
}

impl fmt::Display for IndexType {
    /// Implement [`Display`] for [`IndexType`] so we can
    /// use `.to_string()` method.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IndexType::Taxon => write!(f, "taxon"),
            IndexType::Assembly => write!(f, "assembly"),
        }
    }
}

/// The type of result to return.
///
/// This is true for both `taxon` and
/// `assembly` indexes?
#[derive(Default, Clone, Copy)]
pub enum TaxType {
    /// tax_tree() returns a node and all
    /// of its descendants.
    #[default]
    Tree,
    /// tax_name() returns only the taxon of
    /// interest.
    Name,
    /// tax_lineage() returns all of the nodes
    /// from a given taxon back to the root of the
    /// tree.
    Lineage,
}

impl fmt::Display for TaxType {
    /// Implement [`Display`] for [`TaxType`] so we can
    /// use `.to_string()` method.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaxType::Tree => write!(f, "tax_tree"),
            TaxType::Name => write!(f, "tax_name"),
            TaxType::Lineage => write!(f, "tax_lineage"),
        }
    }
}
