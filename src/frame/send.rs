use crate::frame::Mode;

#[derive(Debug, PartialEq)]
pub(crate) enum Send {
    Start(Mode, String),
    Query(Query),
    Push(Push),
    Ping,
    Suggest(Suggest),
    Count(Count),
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
    pub fn new(collection: String, bucket: String, terms: String) -> Self {
        Query {
            collection,
            bucket,
            terms,
            limit: None,
            offset: None,
        }
    }

    pub fn limit(mut self, limit: u64) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn offset(mut self, offset: u64) -> Self {
        self.offset = Some(offset);
        self
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

#[derive(Debug, PartialEq)]
pub struct Push {
    collection: String,
    bucket: String,
    object: String,
    text: String,
    lang: Option<String>,
}

impl Push {
    pub fn new(collection: String, bucket: String, object: String, text: String) -> Self {
        Push {
            collection,
            bucket,
            object,
            text,
            lang: None,
        }
    }

    pub fn lang(&mut self, lang: String) {
        self.lang = Some(lang);
    }
}

impl ToString for Push {
    fn to_string(&self) -> String {
        let mut s = format!(
            "{} {} {} \"{}\"",
            self.collection, self.bucket, self.object, self.text
        );
        if let Some(lang) = &self.lang {
            s.push_str(&format!(" {}", lang));
        };
        s
    }
}

#[derive(Debug, PartialEq)]
pub struct Count {
    collection: String,
    bucket: Option<String>,
    object: Option<String>,
}

impl Count {
    pub fn new(collection: String) -> Self {
        Count {
            collection,
            bucket: None,
            object: None,
        }
    }

    pub fn bucket(mut self, bucket: String) -> Self {
        self.bucket = Some(bucket);
        self
    }

    pub fn object(mut self, object: String) -> Self {
        if let Some(_) = self.bucket {
            self.object = Some(object);
        }
        self
    }
}

impl ToString for Count {
    fn to_string(&self) -> String {
        let mut s = format!("{}", self.collection);
        if let Some(bucket) = &self.bucket {
            s.push_str(&format!(" {}", bucket));
        };
        if let Some(object) = &self.object {
            s.push_str(&format!(" {}", object));
        };
        s
    }
}

#[derive(Debug, PartialEq)]
pub struct Suggest {
    collection: String,
    bucket: String,
    word: String,
    limit: Option<u64>,
}

impl Suggest {
    pub fn new(collection: String, bucket: String, word: String) -> Self {
        Suggest {
            collection,
            bucket,
            word,
            limit: None,
        }
    }

    pub fn limit(&mut self, limit: u64) {
        self.limit = Some(limit);
    }
}

impl ToString for Suggest {
    fn to_string(&self) -> String {
        let mut s = format!("{} {} \"{}\"", self.collection, self.bucket, self.word);
        if let Some(limit) = &self.limit {
            s.push_str(&format!(" {}", limit));
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
            Send::Push(push) => format!("PUSH {}\r\n", push.to_string()),
            Send::Count(count) => format!("COUNT {}\r\n", count.to_string()),
            Send::Suggest(suggest) => format!("SUGGEST {}\r\n", suggest.to_string()),
            Send::Ping => format!("PING\r\n"),
        }
    }
}

mod test {
    use super::*;

    #[test]
    fn frame_send_to_string() {
        assert_eq!(
            "START ingest passwd\r\n".to_string(),
            Send::Start(Mode::Ingest, "passwd".to_string()).to_string()
        );

        assert_eq!("QUIT\r\n".to_string(), (Send::Quit).to_string());

        assert_eq!(
            "QUERY messages user:0dcde3a6 \"valerian saliou\"\r\n".to_string(),
            Send::Query(Query::new(
                "messages".into(),
                "user:0dcde3a6".into(),
                "valerian saliou".into()
            ))
            .to_string()
        );

        assert_eq!(
            "PUSH messages user:0dcde3a6 conversation:71f3d63b \"Hello Valerian Saliou, how are you today?\"\r\n".to_string(),
            Send::Push(Push::new(
                "messages".into(),
                "user:0dcde3a6".into(),
                "conversation:71f3d63b".into(),
                "Hello Valerian Saliou, how are you today?".into()
            ))
            .to_string()
        );

        assert_eq!(
            "COUNT messages user:0dcde3a6 conversation:71f3d63b\r\n".to_string(),
            Send::Count(
                Count::new("messages".into())
                    .bucket("user:0dcde3a6".into())
                    .object("conversation:71f3d63b".into())
            )
            .to_string()
        );

        assert_eq!("PING\r\n".to_string(), Send::Ping.to_string());

        assert_eq!(
            "SUGGEST messages user:0dcde3a6 \"val\"\r\n".to_string(),
            Send::Suggest(Suggest::new(
                "messages".into(),
                "user:0dcde3a6".into(),
                "val".into()
            ))
            .to_string()
        )
    }
}
