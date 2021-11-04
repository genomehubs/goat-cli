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
