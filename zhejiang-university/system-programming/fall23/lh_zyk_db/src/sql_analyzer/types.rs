use super::errors::MyParseError;
use nom::IResult;
use nom_locate::LocatedSpan;
use serde::{Deserialize, Serialize};
use tabled::Tabled;

// Use nom_locate's LocatedSpan as a wrapper around a string input
pub type Span<'a> = LocatedSpan<&'a str>;

// the result for all of our parsers, they will have our span type as input and can have any output
// this will use a default error type but we will change that latter
pub type ParseResult<'a, T> = IResult<Span<'a>, T, MyParseError<'a>>;

// many other imports omitted
/// A colum's type
#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum SqlType {
    // these are basic for now. Will add more + size max later on
    String,
    Int,
    Unknown,
}

/// A column's name + type
#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Column {
    pub name: String,
    pub type_info: SqlType,
}

/// Values appears in SQL statement, like insert, update..
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize, Tabled)]
pub enum SqlValue {
    String(String),
    Int(i32),
    Unknown,
}

/// Vector of SQL Value, used in insert
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct RowValue {
    pub values: Vec<SqlValue>,
}

/// Compare Operators in SQL statement, like <, <=, <>, = ...
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum CmpOpt {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum WhereConstraint {
    And(Box<WhereConstraint>, Box<WhereConstraint>),
    Or(Box<WhereConstraint>, Box<WhereConstraint>),
    Not(Box<WhereConstraint>),
    // column, cmp, value
    Constrait(String, CmpOpt, SqlValue),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct SetItem {
    pub column: String,
    pub value: SqlValue,
}

/// The table and its columns to create
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct CreateStatement {
    pub table: String,
    pub columns: Vec<Column>,
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct DropStatement {
    pub table: String,
}

/// The table and its columns to create
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct InsertStatement {
    pub table: String,
    pub columns: Option<Vec<String>>,
    pub values: RowValue,
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct SelectStatement {
    pub table: String,
    pub columns: Vec<String>,
    pub constraints: Option<WhereConstraint>,
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct DeleteStatement {
    pub table: String,
    pub constraints: Option<WhereConstraint>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct UpdateStatement {
    pub table: String,
    pub sets: Vec<SetItem>,
    pub constraints: Option<WhereConstraint>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum SqlQuery {
    Select(SelectStatement),
    Insert(InsertStatement),
    Create(CreateStatement),
    Delete(DeleteStatement),
    Drop(DropStatement),
    Update(UpdateStatement),
}
