#[macro_use]
extern crate lazy_static;

mod connection;
mod frame;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub type Result<T> = std::result::Result<T, Error>;
