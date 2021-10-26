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
}
