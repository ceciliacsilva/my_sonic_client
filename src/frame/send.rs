use crate::frame::Mode;

#[derive(Debug, PartialEq)]
pub(crate) enum Send {
    Start(Mode, String),
    Query(Query),
    Push,
    Ping,
    Suggest,
    Count,
    Quit,
}

#[derive(Debug, PartialEq)]
pub struct Query {
    collection: String,
    bucket: String,
    terms: String,
    limit: Option<u64>,
    offset: Option<u64>,
}

impl Query {
    pub fn new(
        collection: String,
        bucket: String,
        terms: String,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> Self {
        Query {
            collection,
            bucket,
            terms,
            limit,
            offset,
        }
    }
}

impl ToString for Query {
    fn to_string(&self) -> String {
        let mut s = format!("{} {} \"{}\"", self.collection, self.bucket, self.terms);
        if let Some(limit) = self.limit {
            s.push_str(&format!(" {}", limit));
        };
        if let Some(offset) = self.offset {
            s.push_str(&format!(" {}", offset));
        };
        s
    }
}

impl ToString for Send {
    fn to_string(&self) -> String {
        match self {
            Send::Start(mode, passwd) => format!("START {} {}\r\n", mode.to_string(), passwd),
            Send::Quit => format!("QUIT\r\n"),
            Send::Query(query) => format!("QUERY {}\r\n", query.to_string()),
            _ => "".to_string(),
        }
    }
}
