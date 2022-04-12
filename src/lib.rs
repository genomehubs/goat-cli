//! `goat-cli` is a command line interface to query the
//! [Genomes on a Tree Open API](https://goat.genomehubs.org/api-docs/) using
//! an asynchronous [`tokio`](<https://docs.rs/tokio/latest/tokio/>) runtime.
//!
//! I'm documenting the code here for others, and for future me.

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
