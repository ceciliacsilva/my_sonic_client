#[macro_use]
extern crate lazy_static;

mod connection;
mod frame;
mod frame_send;
// pub mod control;
// mod task;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub type Result<T> = std::result::Result<T, Error>;
