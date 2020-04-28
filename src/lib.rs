//! A minimal (and incomplete) implementation of an *async* Sonic client.
//!
//! This is an experimental project.

#[macro_use]
extern crate lazy_static;

pub mod connection;
pub mod frame;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub type Result<T> = std::result::Result<T, Error>;
