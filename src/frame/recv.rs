use regex::Regex;
use std::io::Cursor;

use crate::frame::{Error, Mode};

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub enum Recv {
    Connected(String),
    // TODO
    //      (mode,        buffer_size)
    // Use buffer_size - admission control.
    Started(Option<Mode>, u64),
    Pending(String),
    Ok,
    Pong,
    EventQuery(String, Vec<String>),
    EventSuggest(String, Vec<String>),
    Ended(String),
    Err(String),
}

impl Recv {
    pub(crate) fn parse(src: &mut Cursor<&[u8]>) -> Result<Self, Error> {
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
                                return Ok(Recv::Connected(version[1].to_string()));
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

                                return Ok(Recv::Started(mode, size));
                            }

                            return Err(format!(
                                "invalid frame; `STARTED` not is_match {}",
                                b_size
                            )
                            .into());
                        }

                        "PENDING" => {
                            let id = words.next().ok_or("invalid frame; `PENDING`")?;
                            return Ok(Recv::Pending(id.to_string()));
                        }
                        "EVENT" => {
                            let event_type = words.next().ok_or("invalid frame; `EVENT` type")?;
                            if event_type == "QUERY" {
                                let id = words.next().ok_or("invalid frame; `EVENT` id")?;

                                let keys = words.map(|word| word.to_string()).collect();

                                return Ok(Recv::EventQuery(id.to_string(), keys));
                            }

                            if event_type == "SUGGEST" {
                                let id = words.next().ok_or("invalid frame; `EVENT` id")?;

                                let suggestions = words.map(|word| word.to_string()).collect();

                                return Ok(Recv::EventSuggest(id.to_string(), suggestions));
                            }

                            return Err("invalid frame; `EVENT` final".into());
                        }
                        "OK" => return Ok(Recv::Ok),
                        "PONG" => return Ok(Recv::Pong),

                        "ENDED" => {
                            let quit = words.next().ok_or("invalid frame; `ENDED`")?;
                            return Ok(Recv::Ended(quit.to_string()));
                        }
                        "ERR" => {
                            let err_vec: Vec<&str> = words.collect();
                            let mut err_str = String::from("");
                            for word in err_vec {
                                err_str.push_str(&format!(" {}", word));
                            }
                            return Ok(Recv::Err(err_str.to_string()));
                        }
                        _ => return Ok(Recv::Pending(word.to_string())), // _ => return Err("error protocol; invalid command".into()),
                    }
                } else {
                    return Err("error protocol; invalid frame".into());
                }
            }
            Err(e) => Err(e),
        }
    }

    /// Check if it is possible to read a line.
    pub(crate) fn check(src: &mut Cursor<&[u8]>) -> Result<(), Error> {
        match get_line(src) {
            Ok(_) => Ok(()),
            Err(Error::Incomplete) => Err(Error::Incomplete),
            _ => Err("invalid protocol".into()),
        }
    }
}

/// Try to get a line ('\r\n') from the Cursor.
/// If it isn't possible, `return` frame `Incomplete`.
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

mod test {
    use super::*;

    #[test]
    fn frame_parse() {
        let mut line: Cursor<&[u8]> = Cursor::new(b"CONNECTED <sonic-server v1.0.0>\r\n");

        assert_eq!(
            Recv::Connected("1.0.0".to_string()),
            Recv::parse(&mut line).expect("Failed to parse; `CONNECTED`")
        );

        let mut line: Cursor<&[u8]> = Cursor::new(b"STARTED search protocol(1) buffer(20000)\r\n");

        assert_eq!(
            Recv::Started(Some(Mode::Search), 20000),
            Recv::parse(&mut line).expect("Failed to parse; `STARTED`")
        );

        let mut line: Cursor<&[u8]> = Cursor::new(b"PENDING Bt2m2gYa\r\n");

        assert_eq!(
            Recv::Pending("Bt2m2gYa".into()),
            Recv::parse(&mut line).expect("Failed to parse; `PENDING`")
        );

        let mut line: Cursor<&[u8]> = Cursor::new(b"ENDED quit\r\n");

        assert_eq!(
            Recv::Ended("quit".into()),
            Recv::parse(&mut line).expect("Failed to parse; `ENDED`")
        );

        let mut line: Cursor<&[u8]> =
            Cursor::new(b"EVENT QUERY Bt2m2gYa conversation:71f3d63b conversation:6501e83a\r\n");

        assert_eq!(
            Recv::EventQuery(
                "Bt2m2gYa".into(),
                vec![
                    "conversation:71f3d63b".into(),
                    "conversation:6501e83a".into()
                ]
            ),
            Recv::parse(&mut line).expect("Failed to parse; `EVENT QUEUE`")
        );

        let mut line: Cursor<&[u8]> = Cursor::new(b"EVENT SUGGEST z98uDE0f valerian valala\r\n");

        assert_eq!(
            Recv::EventSuggest("z98uDE0f".into(), vec!["valerian".into(), "valala".into()]),
            Recv::parse(&mut line).expect("Failed to parse; `EVENT SUGGEST`")
        );

        let mut line: Cursor<&[u8]> =
            Cursor::new(b"ERR invalid_format(PUSH <collection> <bucket> <object> \"<text>\")\r\n");

        assert_eq!(
            Recv::Err(
                " invalid_format(PUSH <collection> <bucket> <object> \"<text>\")".to_string()
            ),
            Recv::parse(&mut line).expect("Failed to parse; `ERR`")
        );
    }
}
