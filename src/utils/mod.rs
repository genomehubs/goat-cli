/// Parse the command line arguments
/// for `goat-cli search` and `goat-cli count`.
pub mod cli_matches;
/// Parse an expression from the `-e` flag from
/// the CLI.
pub mod expression;
/// Parse taxon ranks from the `--tax-ranks`
/// from the CLI.
pub mod tax_ranks;
/// Generate the URLs from the CLI.
pub mod url;
/// Utility functions used across `goat-cli`.
pub mod utils;
/// Stored data for each of the variables used
/// in `goat-cli`. Useful for comparing and debugging
/// CLI arguments.
pub mod variable_data;
/// Parse variables on the CLI from the `-v` flag.
pub mod variables;
