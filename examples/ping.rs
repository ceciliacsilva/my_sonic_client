use my_sonic_client::connection::Connection;
use my_sonic_client::frame::recv::Recv;
use my_sonic_client::frame::send::Send;
use my_sonic_client::frame::Mode;
use std::env;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed to read .env file.");

    let host = env::var("HOST").expect("Environment var `HOST` not found");
    let port = env::var("PORT").expect("Environment var `PORT` not found");
    let passwd = env::var("PASSWORD").expect("Environment var `PASSWORD` not found");

    let socket = TcpStream::connect(format!("[{}]:{}", host, port))
        .await
        .expect("Failed to create TcpStream connection.");

    let mut connection = Connection::new(socket);

    if let Ok(Recv::Connected(version)) = connection.read_frame().await {
        println!("Protocol version: {}", version);
    }

    connection
        .write_frame(Send::Start(Mode::Ingest, passwd))
        .await
        .expect("Failed to send `START ingest`");

    if let Ok(Recv::Started(Some(mode), size)) = connection.read_frame().await {
        println!("Mode: {:?}, buffer_size: {}", mode, size);
    }

    println!("Ping");

    connection
        .write_frame(Send::Ping)
        .await
        .expect("Failed to send `PING messages`");

    if let Ok(Recv::Pong) = connection.read_frame().await {
        println!("Pong");
    }

    connection
        .write_frame(Send::Quit)
        .await
        .expect("Failed to send `QUIT messages`");

    if let Ok(Recv::Ended(_host)) = connection.read_frame().await {
        println!("End connection");
    }
}
