//!
//! Errors used in `/utils`

use thiserror::Error;

/// Errors pertaining to implementations not yet there.
#[derive(Error, Debug)]
pub enum NotYetImplemented {
    #[error("This feature is not yet implemented :(")]
    NotYetImplemented,
    #[error("GoaT requires a subcommand. Run `goat help` to see more.")]
    CLIError,
}

/// Errors in parsing files.
#[derive(Error, Debug)]
pub enum FileError {
    #[error("Could not parse the line.")]
    ReadLineError,
    #[error("Could not open the file.")]
    FileOpenError,
}

/// Errors in parsing expressions.
#[derive(Error, Debug)]
pub enum ExpressionParseError {
    #[error("This expression query is greater than 100 chars.")]
    QueryTooLong,
    #[error("Use AND keyword, not && for expression queries.")]
    KeywordAndError,
    #[error("Using the \"contains\" keyword is not yet supported.")]
    KeywordContainsError,
    #[error("OR (or ||) keyword is not supported.")]
    KeywordOrError,
    #[error("Set tax_name through -t <taxon_name> and tax_tree by -d flag.")]
    KeywordTaxError,
    #[error("No operators were found in the expression.")]
    NoOperatorError,
    #[error("The input variable is not recognised.")]
    InputVariableError,
    #[error("Input keyword enum does not match database.")]
    KeywordEnumError,
    #[error("Error in expression format. Expressions must be in the format:\n\t<variable> <operator> <value> AND ...")]
    FormatExpressionError,
}
