pub const DEFAULT_PORT: &str = "3306";
pub const DEFAULT_IP: &str = "127.0.0.1";
pub const  DEFAULT_PROMPT: &str = "waterdb";


pub mod client;

pub mod server;

mod connection;

pub use connection::Connection;

mod shutdown;
// use shutdown::Shutdown;

pub mod frame;
pub use frame::Frame;

pub mod storage;

pub mod error;

pub mod sql;

pub mod db;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub type Result<T> = std::result::Result<T, Error>;

pub mod config;