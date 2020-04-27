use std::error::Error;
use tokio::net::TcpStream;
use tokio::prelude::*;

pub struct ControlChan {
    host: String,
    port: usize,
    password: String,
    debugging: bool,
}

impl ControlChan {
    fn new(host: &str, port: usize, password: &str) -> Self {
        ControlChan {
            host: host.into(),
            port: port,
            password: password.into(),
            debugging: false,
        }
    }

    fn connect(&self) -> String {
        format!("START control {}\n", &self.password)
    }

    fn trigger(&self, action: Option<&str>) -> String {
        format!("TRIGGER {}\r\n", action.unwrap_or(""))
    }

    fn ping(&self) -> String {
        "PING\r\n".to_string()
    }

    fn quit(&self) -> String {
        "QUIT\r\n".to_string()
    }

    fn help(&self, manual: Option<&str>) -> String {
        format!("HELP {}\r\n", manual.unwrap_or(""))
    }

    pub fn debug(&mut self) {
        self.debugging = true;
    }
}
