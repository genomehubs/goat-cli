use clap::Error as ClapError;
use indicatif::style::TemplateError;
use owo_colors::{OwoColorize, Stream::Stderr, Style};
use reqwest::Error as ReqError;
use serde_json::Error as SerdeJSONError;
use std::io::Error as IOError;
use std::{error::Error as StdError, fmt, result};

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub struct Error(Box<ErrorKind>);

impl Error {
    /// A crate private constructor for `Error`.
    pub(crate) fn new(kind: ErrorKind) -> Error {
        Error(Box::new(kind))
    }

    /// Return the specific type of this error.
    pub fn kind(&self) -> &ErrorKind {
        &self.0
    }

    /// Unwrap this error into its underlying type.
    pub fn into_kind(self) -> ErrorKind {
        *self.0
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    // any generic command line interface error
    GenericCli(String),
    // specifically erroring through clap
    Cli(ClapError),
    // any i/o errors
    IO(IOError),
    // errors in formatting the TSV
    FormatTSV(String),
    // errors originating from reqwest
    Reqwest(ReqError),
    // serde JSON error
    SerdeJSON(SerdeJSONError),
    // errors in expression strings
    Expression(String),
    // errors in variable strings
    Variable(String),
    // error in tax ranks
    TaxRank(String),
    // progress bar error
    Progress(TemplateError),
    // error in report
    Report(String),
}

impl From<ClapError> for Error {
    fn from(err: ClapError) -> Self {
        Error::new(ErrorKind::Cli(err))
    }
}

impl From<IOError> for Error {
    fn from(err: IOError) -> Self {
        Error::new(ErrorKind::IO(err))
    }
}

impl From<ReqError> for Error {
    fn from(err: ReqError) -> Self {
        Error::new(ErrorKind::Reqwest(err))
    }
}

impl From<SerdeJSONError> for Error {
    fn from(err: SerdeJSONError) -> Self {
        Error::new(ErrorKind::SerdeJSON(err))
    }
}

impl From<TemplateError> for Error {
    fn from(err: TemplateError) -> Self {
        Error::new(ErrorKind::Progress(err))
    }
}

impl StdError for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} in goat-cli:\n\t",
            "error".if_supports_color(Stderr, |text| {
                let style = Style::new().red().bold();
                text.style(style)
            })
        )?;

        match &*self.0 {
            ErrorKind::Cli(err) => write!(f, "command line error (clap) - {}", err),
            ErrorKind::GenericCli(err) => write!(f, "command line error - {}", err),
            ErrorKind::IO(err) => write!(f, "I/O error - {}", err),
            ErrorKind::FormatTSV(err) => write!(f, "format TSV error - {}", err),
            ErrorKind::Reqwest(err) => write!(f, "request error - {}", err),
            ErrorKind::Expression(err) => write!(f, "expression error - {}", err),
            ErrorKind::Variable(err) => write!(f, "variable error - {}", err),
            ErrorKind::TaxRank(err) => write!(f, "tax rank error - {}", err),
            ErrorKind::SerdeJSON(err) => write!(f, "serialising JSON error - {}", err),
            ErrorKind::Progress(err) => write!(f, "progress bar error - {}", err),
            ErrorKind::Report(err) => write!(f, "report error - {}", err),
        }
    }
}
