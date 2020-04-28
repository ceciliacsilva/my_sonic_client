use crate::frame::RecvFrame;
use bytes::{Buf, BytesMut};
use std::io::{self, Cursor};
use tokio::io::BufWriter;
use tokio::net::TcpStream;
use tokio::prelude::*;

/// Send and receive frame.
#[derive(Debug)]
pub(crate) struct Connection {
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

    pub(crate) async fn write_string(&mut self, frame: String) -> io::Result<()> {
        self.stream.write_all(&frame.into_bytes()).await?;

        self.stream.flush().await
    }

    pub(crate) async fn read_frame(&mut self) -> Option<RecvFrame> {
        loop {
            let mut buf = Cursor::new(&self.buffer[..]);

            match RecvFrame::check(&mut buf) {
                Ok(_) => {
                    println!("READ: ");

                    let len = buf.position() as usize;

                    // before parsing
                    buf.set_position(0);

                    let frame = RecvFrame::parse(&mut buf).expect("mal");

                    self.buffer.advance(len);

                    return Some(frame);
                }
                Err(Incomplete) => {}
                Err(e) => return None,
            }

            if 0 == self
                .stream
                .read_buf(&mut self.buffer)
                .await
                .expect("Failed trying to read_buf")
            {
                // The remote closed the connection. For this to be a clean
                // shutdown, there should be no data in the read buffer. If
                // there is, this means that the peer closed the socket while
                // sending a frame.
                if self.buffer.is_empty() {
                    return Some(RecvFrame::Ended);
                } else {
                    return None;
                }
            }
        }
    }
}

mod test {
    use super::*;
    #[tokio::test]
    async fn test_connection() {
        let socket = TcpStream::connect("[::1]:1491")
            .await
            .expect("Failed to create TcpStream connection.");

        let mut connection = Connection::new(socket);

        connection
            .write_string("START search SecretPassword".to_string())
            .await;

        if let Some(res) = connection.read_frame().await {
            assert_eq!(RecvFrame::Connected("1.2.3".to_string()), res);
        }
        if let Some(res) = connection.read_frame().await {
            assert_eq!(
                RecvFrame::Started(Some(crate::frame::Mode::Search), 20000),
                res
            );
        }
    }
}
