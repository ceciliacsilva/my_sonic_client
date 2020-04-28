use my_sonic_client::connection::Connection;
use my_sonic_client::frame::recv::Recv;
use my_sonic_client::frame::send::Push;
use my_sonic_client::frame::send::Send;
use my_sonic_client::frame::Mode;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() {
    let socket = TcpStream::connect("[::1]:1491")
        .await
        .expect("Failed to create TcpStream connection.");

    let mut connection = Connection::new(socket);

    if let Ok(Recv::Connected(version)) = connection.read_frame().await {
        println!("Protocol version: {}", version);
    }

    connection
        .write_frame(Send::Start(Mode::Ingest, "SecretPassword".to_string()))
        .await
        .expect("Failed to send `START ingest`");

    if let Ok(Recv::Started(Some(mode), size)) = connection.read_frame().await {
        println!("Mode: {:?}, buffer_size: {}", mode, size);
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

    if let Ok(Recv::Ok) = connection.read_frame().await {
        println!("Push Ok");
    }

    connection
        .write_frame(Send::Quit)
        .await
        .expect("Failed to send `QUIT messages`");

    if let Ok(Recv::Ended(_host)) = connection.read_frame().await {
        println!("End connection");
    }
}
