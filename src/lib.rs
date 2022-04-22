//! `goat-cli` is a command line interface to query the
//! [Genomes on a Tree Open API](https://goat.genomehubs.org/api-docs/) using
//! an asynchronous [`tokio`](<https://docs.rs/tokio/latest/tokio/>) runtime.
//!
//! I'm documenting the code here for others, and for future me.

use lazy_static::lazy_static;

/// Query the GoaT count API.
pub mod count;
/// Collection of errors used throughout
/// the CLI.
pub mod error;
/// Query the GoaT lookup API.
pub mod lookup;
/// A module to produce a progress
/// bar.
pub mod progress;
/// Query the GoaT record API.
pub mod record;
/// Query the GoaT search API.
pub mod search;
/// Collection of utility functions
/// used elsewhere.
pub mod utils;

/// The base URL for GoaT.
const GOAT_URL_BASE: &str = "https://goat.genomehubs.org/api/";
/// The current version of the GoaT API.
const GOAT_API_VERSION: &str = "v0.0.1/";

lazy_static! {
    /// The current GoaT URL.
    pub static ref GOAT_URL: String = format!("{}{}", GOAT_URL_BASE, GOAT_API_VERSION);
    /// The taxonomy that `goat-cli` uses.
    pub static ref TAXONOMY: String = "ncbi".to_string();
}

// global size limits on pinging the API
lazy_static! {
    /// Upper limit for the CLI arg `--size`.
    pub static ref UPPER_CLI_SIZE_LIMIT: usize = 50000;
    /// Upper limit for the number of entries in the file for CLI arg `-f`.
    pub static ref UPPER_CLI_FILE_LIMIT: usize = 500;
}
