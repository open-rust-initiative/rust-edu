use super::super::sql_analyzer::types::*;
use super::super::storage::*;
use miette::Diagnostic;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// List of column info
pub type ColumnInfo = Vec<Column>;

/// The struct of data table as well as execute result
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct SqlTable {
    /// Column info for all columns in the table
    pub columns: ColumnInfo,
    /// row id to row
    pub rows: Vec<RowValue>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ExecuteResponse {
    Message(String),
    Count(usize),
    View(Box<SqlTable>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SelectCriteria {
    pub columns_name: String,
    pub cmpopt: CmpOpt,
    pub value: SqlValue,
}

/// Errors during query execution
#[derive(Error, Debug, Diagnostic)]
#[error("Query Execution Error")]
pub enum QueryExecutionError {
    #[error("Table {0} was not found")]
    TableNotFound(String),
    #[error("Table {0} already exists")]
    TableAlreadyExists(String),
    #[error("Column {0} does not exist")]
    ColumnDoesNotExist(String),
    #[error("Type {0} does not match the column definition")]
    TypeDoesNotMatch(String),
    #[error("Table {0} delete fail")]
    TableDeletefail(String),
    #[error("Table {0} save fail")]
    TableSavefail(String),
    #[error("Table {0} open fail")]
    TableOpenfail(String),
    #[error("The value which want to delete is null")]
    ValueIsNull(String),
    #[error("No conditions obtained")]
    NoConditionsObtained(),
}

pub trait Executable {
    /// The error should be check error with error message
    fn check_and_execute(
        self,
        storage_util: StoreUtil,
    ) -> Result<ExecuteResponse, QueryExecutionError>;
}
