use std::fmt;
use std::num::ParseIntError;
use std::string::FromUtf8Error;

pub mod recv;
pub mod send;

#[derive(Debug, PartialEq)]
pub enum Mode {
    Search,
    Ingest,
}

impl ToString for Mode {
    fn to_string(&self) -> String {
        match self {
            Mode::Ingest => "ingest".to_string(),
            Mode::Search => "search".to_string(),
        }
    }
}

#[derive(Debug)]
pub(crate) enum Error {
    Incomplete,
    Other(crate::Error),
}

impl From<FromUtf8Error> for Error {
    fn from(_src: FromUtf8Error) -> Error {
        "protocol error; invalid frame format".into()
    }
}

impl From<String> for Error {
    fn from(src: String) -> Error {
        Error::Other(src.into())
    }
}

impl From<&str> for Error {
    fn from(src: &str) -> Error {
        src.to_string().into()
    }
}

impl From<ParseIntError> for Error {
    fn from(_src: ParseIntError) -> Error {
        "protocol error; invalid frame format".into()
    }
}

impl std::error::Error for Error {}
impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Incomplete => "stream ended early".fmt(fmt),
            Error::Other(err) => err.fmt(fmt),
        }
    }
}
