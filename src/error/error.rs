use thiserror::Error;

#[derive(Error, Debug)]
pub enum RetrieveRecordError {
    #[error("[-]\tCould not cast record to &str.")]
    CastToStr,
    #[error("[-]\tCould not cast record to u64.")]
    CastToU64,
    #[error("[-]\tCould not cast record to f64.")]
    CastToF64,
    #[error("[-]\tFound &str other than assembly_level, or assembly_span.")]
    MatchingRawValueError,
}

#[derive(Error, Debug)]
pub enum NotYetImplemented {
    #[error("[-]\tThis feature is not yet implemented :(")]
    NotYetImplemented,
    #[error("[-]\tPlease use `goat search`. Other subcommands are not implemented yet.")]
    CLIError,
}

#[derive(Error, Debug)]
pub enum FileError {
    #[error("[-]\tCould not parse the line.")]
    ReadLineError,
    #[error("[-]\tCould not open the file.")]
    FileOpenError,
}
