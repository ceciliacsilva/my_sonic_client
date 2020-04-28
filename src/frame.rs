// use failure::Error;
use regex::Regex;
use std::fmt;
use std::io::Cursor;
use std::num::ParseIntError;
use std::string::FromUtf8Error;

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub(crate) enum RecvFrame {
    Connected(String),
    // read buffer(_) parameter
    Started(Option<Mode>, u64),
    Pending(String),
    Pong,
    EventQuery(String, Vec<String>),
    EventSuggest(String, Vec<String>),
    Ended,
}

#[derive(Debug, PartialEq)]
pub(crate) enum Mode {
    Search,
    Ingest,
}

#[derive(Debug)]
pub(crate) enum Error {
    Incomplete,
    Other(crate::Error),
}

impl RecvFrame {
    pub(crate) fn parse(src: &mut Cursor<&[u8]>) -> Result<RecvFrame, Error> {
        match get_line(src) {
            Ok(line) => {
                let line = String::from_utf8(line.to_vec())?;
                let mut words = line.split_whitespace();

                if let Some(word) = words.next() {
                    match word {
                        "CONNECTED" => {
                            let sonic_version = words.next().ok_or("invalid frame; `CONNECTED`")?;

                            if sonic_version != "<sonic-server" {
                                return Err("invalid frame; `CONNECTED`".into());
                            }

                            let version = words.next().ok_or("invalid frame; `CONNECTED`")?;

                            lazy_static! {
                                static ref RE_CONNECTED: Regex =
                                    Regex::new(r"v\d\.\d\.\d>").expect("Failed to create Regex");
                            }

                            if RE_CONNECTED.is_match(version) {
                                let version: Vec<&str> =
                                    version.split(|c| c == 'v' || c == '>').collect();
                                return Ok(RecvFrame::Connected(version[1].to_string()));
                            }

                            return Err("invalid frame; `CONNECTED`".into());
                        }
                        "STARTED" => {
                            let mode = words.next().ok_or("invalid frame; `STARTED` mode")?;
                            let mode = if mode == "search" {
                                Some(Mode::Search)
                            } else if mode == "ingest" {
                                Some(Mode::Ingest)
                            } else {
                                None
                            };

                            let _prot_version =
                                words.next().ok_or("invalid frame; `STARTED` proto")?;

                            let b_size = words.next().ok_or("invalid frame; `STARTED` b_size")?;

                            lazy_static! {
                                static ref RE_STARTED: Regex =
                                    Regex::new(r"buffer(\d*)").expect("Failed to create Regex");
                            };

                            if RE_STARTED.is_match(b_size) {
                                let b_size: Vec<&str> =
                                    b_size.split(|c| c == '(' || c == ')').collect();
                                let size = b_size.get(1).ok_or("invalid frame; `STARTED` size")?;
                                let size = size.parse::<u64>()?;

                                return Ok(RecvFrame::Started(mode, size));
                            }

                            return Err(format!(
                                "invalid frame; `STARTED` not is_match {}",
                                b_size
                            )
                            .into());
                        }

                        "PENDING" => {
                            let id = words.next().ok_or("invalid frame; `PENDING`")?;
                            return Ok(RecvFrame::Pending(id.to_string()));
                        }
                        "EVENT" => {
                            let event_type = words.next().ok_or("invalid frame; `EVENT` type")?;
                            if event_type == "QUERY" {
                                let id = words.next().ok_or("invalid frame; `EVENT` id")?;

                                let keys = words.map(|word| word.to_string()).collect();

                                return Ok(RecvFrame::EventQuery(id.to_string(), keys));
                            }

                            if event_type == "SUGGEST" {
                                let id = words.next().ok_or("invalid frame; `EVENT` id")?;

                                let suggestions = words.map(|word| word.to_string()).collect();

                                return Ok(RecvFrame::EventSuggest(id.to_string(), suggestions));
                            }

                            return Err("invalid frame; `EVENT` final".into());
                        }
                        _ => return Err("error protocol; invalid command".into()),
                    }
                } else {
                    return Err("error protocol; invalid frame".into());
                }
            }
            Err(e) => Err(e),
        }
    }

    pub(crate) fn check(src: &mut Cursor<&[u8]>) -> Result<(), Error> {
        match get_line(src) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
}

fn get_line<'a>(src: &mut Cursor<&'a [u8]>) -> Result<&'a [u8], Error> {
    let start = src.position() as usize;
    let end = src.get_ref().len() as usize;

    for i in start..end {
        if src.get_ref()[i] == b'\r' && src.get_ref()[i + 1] == b'\n' {
            src.set_position((i + 2) as u64);

            return Ok(&src.get_ref()[start..i]);
        }
    }

    Err(Error::Incomplete)
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
