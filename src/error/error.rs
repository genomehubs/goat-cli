use thiserror::Error;
#[derive(Error, Debug)]
pub enum NotYetImplemented {
    #[error("[-]\tThis feature is not yet implemented :(")]
    NotYetImplemented,
    #[error(
        "[-]\tPlease use `goat search` or `goat count`. Other subcommands are not implemented yet."
    )]
    CLIError,
}

#[derive(Error, Debug)]
pub enum FileError {
    #[error("[-]\tCould not parse the line.")]
    ReadLineError,
    #[error("[-]\tCould not open the file.")]
    FileOpenError,
}

// will need some parsing errors.
#[derive(Error, Debug)]
pub enum ExpressionParseError {
    #[error("[-]\tThis expression query is greater than 100 chars.")]
    QueryTooLong,
    #[error("[-]\tUse AND keyword, not && for expression queries.")]
    KeywordAndError,
    #[error("[-]\tUsing the \"contains\" keyword is not yet supported.")]
    KeywordContainsError,
    #[error("[-]\tOR (or ||) keyword is not supported.")]
    KeywordOrError,
    #[error("[-]\tSet tax_name through -t <taxon_name> and tax_tree by -d flag.")]
    KeywordTaxError,
    #[error("[-]\tNo operators were found in the expression.")]
    NoOperatorError,
    #[error("[-]\tSplit vector on single expression is invalid. Are the input variables or operands correct?")]
    SplitVectorError,
    #[error("[-]\tThe input variable is not recognised.")]
    InputVariableError,
    #[error("[-]\tInput keyword enum does not match database.")]
    KeywordEnumError,
    #[error("[-]\tError in expression format. Expressions must be in the format:\n\t<variable> <operator> <value> AND ...")]
    FormatExpressionError,
}
