use thiserror::Error;
use super::super::{
    datatype::token::*,
    models::error::StructError,
};

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Unexpected token: '{0}'")]
    UnexpectedToken(Token),

    #[error("Missing token: '{0}'")]
    MissingToken(Token),

    #[error("Missing table")]
    MissingTable,

    #[error("Missing comparator")]
    MissingComparator,

    #[error("Missing column")]
    MissingColumn,

    #[error("Missing sorting keyword")]
    MissingSort,

    #[error("Missing value")]
    MissingValue,

    #[error("Missing terminator")]
    MissingTerminator,

    #[error("Missing function")]
    MissingFunction,

    #[error("Syntax error: {0}")]
    SyntaxError(String),

    #[error("Incorrect number of values: expect {0}")]
    IncorrectValueCount(usize),

    #[error("Incorrect expression")]
    IncorrectExpression,

    #[error("Incorrect condition")]
    IncorrectCondition,

    #[error("Incorrect function")]
    IncorrectFunction,

    #[error("{0}")]
    StructError(#[from] StructError),

    #[error("Unknown error")]
    UnknownError,
}

pub type Result<T> = std::result::Result<T, ParseError>;