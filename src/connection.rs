use crate::frame::recv::Recv;
use crate::frame::send::Send;
use crate::Error;
use bytes::{Buf, BytesMut};
use std::io::{self, Cursor};
use tokio::io::BufWriter;
use tokio::net::TcpStream;
use tokio::prelude::*;

/// Send and receive frame.
#[derive(Debug)]
pub struct Connection {
    stream: BufWriter<TcpStream>,
    // When Tokio v0.3 change to tokio::BytesMut
    buffer: BytesMut,
}

impl Connection {
    /// Create new `Connection`, backed by `socket`.
    pub fn new(socket: TcpStream) -> Connection {
        Connection {
            stream: BufWriter::new(socket),
            // For now 4KB is the default, this may change based on the use cases.
            buffer: BytesMut::with_capacity(4 * 1024),
        }
    }

    /// Write a `Send` Frame into the `self.stream`.
    pub async fn write_frame(&mut self, frame: Send) -> io::Result<()> {
        self.write_string(frame.to_string()).await
    }

    /// Write a `String` into the `self.stream`.
    pub async fn write_string(&mut self, frame: String) -> io::Result<()> {
        self.stream.write_all(&frame.into_bytes()).await?;

        self.stream.flush().await
    }

    /// Read `self.buffer` into a `Recv` Frame.
    pub async fn read_frame(&mut self) -> Result<Recv, Error> {
        loop {
            let mut buf = Cursor::new(&self.buffer[..]);

            match Recv::check(&mut buf) {
                Ok(_) => {
                    // TODO Some kind of debug mode.

                    let len = buf.position() as usize;

                    // Set position to Zero before parsing.
                    buf.set_position(0);
                    let frame = Recv::parse(&mut buf)?;
                    self.buffer.advance(len);

                    return Ok(frame);
                }
                Err(crate::frame::Error::Incomplete) => {}
                Err(e) => return Err(e.into()),
            }

            if 0 == self
                .stream
                .read_buf(&mut self.buffer)
                .await
                .expect("Failed trying to read_buf")
            {
                // Mini-redis:
                // The remote closed the connection. For this to be a clean
                // shutdown, there should be no data in the read buffer. If
                // there is, this means that the peer closed the socket while
                // sending a frame.
                if self.buffer.is_empty() {
                    return Ok(Recv::Ended("Remote".to_string()));
                } else {
                    return Err("connection reset by peer".into());
                }
            }
        }
    }
}

mod test {
    use super::*;

    use crate::frame::send::{Push, Query};
    use crate::frame::Mode;

    #[tokio::test]
    async fn ingest_mode() {
        let socket = TcpStream::connect("[::1]:1491")
            .await
            .expect("Failed to create TcpStream connection.");

        let mut connection = Connection::new(socket);

        if let Ok(Recv::Connected(version)) = connection.read_frame().await {
            assert_eq!("1.2.3".to_string(), version);
        }

        connection
            .write_frame(Send::Start(
                crate::frame::Mode::Ingest,
                "SecretPassword".to_string(),
            ))
            .await
            .expect("Failed to send `START ingest`");

        if let Ok(Recv::Started(mode, _size)) = connection.read_frame().await {
            assert_eq!(Some(Mode::Ingest), mode);
        }

        connection
            .write_frame(Send::Push(Push::new(
                "messages".into(),
                "user:0dcde3a6".into(),
                "conversation:71f3d63c".into(),
                "Hello, how are you today?".into(),
            )))
            .await
            .expect("Failed to send `PUSH messages`");

        if let Ok(Recv::Ok) = connection.read_frame().await {}

        connection
            .write_frame(Send::Quit)
            .await
            .expect("Failed to send `QUIT messages`");

        if let Ok(Recv::Ended(_host)) = connection.read_frame().await {}
    }

    #[tokio::test]
    async fn search_mode() {
        let socket = TcpStream::connect("[::1]:1491")
            .await
            .expect("Failed to create TcpStream connection.");

        let mut connection = Connection::new(socket);

        if let Ok(Recv::Connected(version)) = connection.read_frame().await {
            assert_eq!("1.2.3".to_string(), version);
        }

        connection
            .write_frame(Send::Start(
                crate::frame::Mode::Search,
                "SecretPassword".to_string(),
            ))
            .await
            .expect("Failed to send `START ingest`");

        if let Ok(Recv::Started(mode, _size)) = connection.read_frame().await {
            assert_eq!(Some(Mode::Search), mode);
        }

        let query = Query::new(
            "messages".to_string(),
            "user:0dcde3a6".to_string(),
            "valerian saliou".to_string(),
        );
        connection
            .write_frame(Send::Query(query))
            .await
            .expect("Failed to send `QUERY messages`");

        if let Ok(Recv::Pending(_id)) = connection.read_frame().await {}

        if let Ok(Recv::EventQuery(_id, _keys)) = connection.read_frame().await {}

        connection
            .write_frame(Send::Quit)
            .await
            .expect("Failed to send `QUIT messages`");

        if let Ok(Recv::Ended(_host)) = connection.read_frame().await {}
    }
}
