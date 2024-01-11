use thiserror::Error;
use super::parser::error::ParseError;

#[derive(Error, Debug)]
pub enum SQLError {
    #[error("{0}")]
    ParseError(#[from] ParseError),

    #[error("Unknown Statement")]
    UnknownStatement,
}

pub type Result<T> = std::result::Result<T, SQLError>;