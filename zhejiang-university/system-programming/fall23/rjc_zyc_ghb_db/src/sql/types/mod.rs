pub mod expression;

use crate::error::{Error, Result};

use serde_derive::{Deserialize, Serialize};
use std::borrow::Cow;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

/// waterdb 的支持的数据类型
#[derive(Clone, Debug, Hash, PartialEq, Serialize, Deserialize)]
pub enum DataType {
    Boolean,
    Integer,
    Float,
    String,
}

impl std::fmt::Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(match self {
            Self::Boolean => "BOOLEAN",
            Self::Integer => "INTEGER",
            Self::Float => "FLOAT",
            Self::String => "STRING",
        })
    }
}

/// waterdb 支持的 Value
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
}

impl std::cmp::Eq for Value {}

#[allow(clippy::derived_hash_with_manual_eq)]
impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.datatype().hash(state);
        match self {
            Value::Null => self.hash(state),
            Value::Boolean(v) => v.hash(state),
            Value::Integer(v) => v.hash(state),
            Value::Float(v) => v.to_be_bytes().hash(state),
            Value::String(v) => v.hash(state),
        }
    }
}

impl<'a> From<Value> for Cow<'a, Value> {
    fn from(v: Value) -> Self {
        Cow::Owned(v)
    }
}

impl<'a> From<&'a Value> for Cow<'a, Value> {
    fn from(v: &'a Value) -> Self {
        Cow::Borrowed(v)
    }
}

impl Value {
    /// 返回 DataType，对于 Null 类型返回 None
    pub fn datatype(&self) -> Option<DataType> {
        match self {
            Self::Null => None,
            Self::Boolean(_) => Some(DataType::Boolean),
            Self::Integer(_) => Some(DataType::Integer),
            Self::Float(_) => Some(DataType::Float),
            Self::String(_) => Some(DataType::String),
        }
    }

    /// 返回 boolean 类型的值，其他报错
    pub fn boolean(self) -> Result<bool> {
        match self {
            Self::Boolean(b) => Ok(b),
            v => Err(Error::Value(format!("Not a boolean: {:?}", v))),
        }
    }

    /// 返回 float 类型的值，其他报错
    pub fn float(self) -> Result<f64> {
        match self {
            Self::Float(f) => Ok(f),
            v => Err(Error::Value(format!("Not a float: {:?}", v))),
        }
    }

    /// 返回 integer 类型的值，其他报错
    pub fn integer(self) -> Result<i64> {
        match self {
            Self::Integer(i) => Ok(i),
            v => Err(Error::Value(format!("Not an integer: {:?}", v))),
        }
    }

    /// 返回 string 类型的值，其他报错
    pub fn string(self) -> Result<String> {
        match self {
            Self::String(s) => Ok(s),
            v => Err(Error::Value(format!("Not a string: {:?}", v))),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(
            match self {
                Self::Null => "NULL".to_string(),
                Self::Boolean(b) if *b => "TRUE".to_string(),
                Self::Boolean(_) => "FALSE".to_string(),
                Self::Integer(i) => i.to_string(),
                Self::Float(f) => f.to_string(),
                Self::String(s) => s.clone(),
            }
            .as_ref(),
        )
    }
}

/// 实现元素之前的比较，实现 broadcast
impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Self::Null, Self::Null) => Some(Ordering::Equal),
            (Self::Null, _) => Some(Ordering::Less),
            (_, Self::Null) => Some(Ordering::Greater),
            (Self::Boolean(a), Self::Boolean(b)) => a.partial_cmp(b),
            (Self::Float(a), Self::Float(b)) => a.partial_cmp(b),
            (Self::Float(a), Self::Integer(b)) => a.partial_cmp(&(*b as f64)),
            (Self::Integer(a), Self::Float(b)) => (*a as f64).partial_cmp(b),
            (Self::Integer(a), Self::Integer(b)) => a.partial_cmp(b),
            (Self::String(a), Self::String(b)) => a.partial_cmp(b),
            (_, _) => None,
        }
    }
}

impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Value::Boolean(v)
    }
}

impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Value::Float(v)
    }
}

impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Value::Integer(v)
    }
}

impl From<String> for Value {
    fn from(v: String) -> Self {
        Value::String(v)
    }
}

impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Value::String(v.to_owned())
    }
}

/// 定义行数据的表示
pub type Row = Vec<Value>;

/// 行数据的迭代器
pub type Rows = Box<dyn Iterator<Item = Result<Row>> + Send>;

/// table 属性名字
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Column {
    pub name: Option<String>,
}

/// table 属性
pub type Columns = Vec<Column>;
