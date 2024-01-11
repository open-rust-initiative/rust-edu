use std::fmt;
use std::fmt::{Formatter};

#[derive(Debug)]
pub enum Errors {
    UnimplementedOperation,
    InvalidExpression,
    ElementNotFound,
    DatabaseNotExisted,
    DiskSaveError,
    FileSystemError,
    ParseSQLError,
    InvalidCommand,
    TableNotExisted(String),
    TableExisted(String),
    InvalidColumnType,
}

impl Errors {
    pub fn print(self) {
        println!("{}", self);
    }
}

impl fmt::Display for Errors {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Errors::UnimplementedOperation => { f.write_str("This operation is unimplemented.\n") }
            Errors::InvalidExpression => { f.write_str("Expression is invalid.\n") }
            Errors::ElementNotFound => { f.write_str("ElementNotFound.\n") }
            Errors::DatabaseNotExisted => { f.write_str("DatabaseNotExisted.\n") }
            Errors::DiskSaveError => { f.write_str("DiskSaveError.\n") }
            Errors::FileSystemError => { f.write_str("FileSystemError.\n") }
            Errors::ParseSQLError => { f.write_str("ParseSQLError.\n") }
            Errors::InvalidCommand => { f.write_str("InvalidCommand.\n") }
            Errors::TableNotExisted(s) => { f.write_str(format!("Table {} is not existed.\n", s).as_str()) }
            Errors::TableExisted(s) => { f.write_str(format!("Table {} is existed.\n", s).as_str()) }
            Errors::InvalidColumnType => { f.write_str("InvalidColumnType\n") }
        }
    }
}