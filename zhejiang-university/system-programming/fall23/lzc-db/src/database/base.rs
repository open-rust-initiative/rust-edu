use crate::system::errors::Errors;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum DataType {
    Float,
    Int,
    Bool,
    String,
    Invalid,
}

impl DataType {
    pub fn new(data_type: String) -> DataType {
        match data_type.to_lowercase().as_str() {
            "float" => DataType::Float,
            "int" => DataType::Int,
            "bool" => DataType::Bool,
            "string" => DataType::String,
            _ => DataType::Invalid,
        }
    }
    pub fn data_type(&self) -> String {
        match self {
            DataType::Float => "float".to_string(),
            DataType::Int => "int".to_string(),
            DataType::Bool => "bool".to_string(),
            DataType::String => "string".to_string(),
            DataType::Invalid => "null".to_string(),
        }
    }
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            DataType::Float => f.write_str("float"),
            DataType::Int => f.write_str("int"),
            DataType::Bool => f.write_str("bool"),
            DataType::String => f.write_str("string"),
            DataType::Invalid => f.write_str("null"),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct ColumnAttr {
    pub name: String,
    pub datatype: DataType,
    pub is_pk: bool,
    pub is_nullable: bool,
    pub default: Option<String>,
}

impl ColumnAttr {
    pub fn new(
        name: String,
        data_type: String,
        is_pk: bool,
        is_nullable: bool,
        default: Option<String>,
    ) -> ColumnAttr {
        let datatype = DataType::new(data_type);
        ColumnAttr {
            name,
            datatype,
            is_pk,
            is_nullable,
            default,
        }
    }

    pub fn attr(&self) -> HashMap<String, String> {
        let mut row: HashMap<String, String> = HashMap::new();
        row.insert("name".to_string(), self.name.to_string());
        row.insert("datatype".to_string(), self.datatype.data_type());
        row.insert("is_pk".to_string(), self.is_pk.to_string());
        row.insert("is_nullable".to_string(), self.is_nullable.to_string());
        row.insert(
            "default".to_string(),
            self.default.clone().unwrap_or("None".to_string()),
        );
        row
    }
}

impl std::fmt::Display for ColumnAttr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&*self.name)
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct ForeignKeyAttr {
    pub table: String,
    // current table's column
    pub col_a: String,
    // referred table's column
    pub col_b: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum ColumnData {
    Int(Vec<Option<i32>>),
    Float(Vec<Option<f32>>),
    Str(Vec<Option<String>>),
    Bool(Vec<Option<bool>>),
    None,
}

impl ColumnData {
    pub fn get_all_data(&self) -> Result<Vec<String>, Errors> {
        let result = match &self {
            ColumnData::Int(x) => x
                .iter()
                .map(|v| {
                    if let Some(value) = v {
                        value.to_string()
                    } else {
                        "".to_string()
                    }
                })
                .collect::<Vec<String>>(),
            ColumnData::Float(x) => x
                .iter()
                .map(|v| {
                    if let Some(value) = v {
                        value.to_string()
                    } else {
                        "".to_string()
                    }
                })
                .collect::<Vec<String>>(),
            ColumnData::Str(x) => x
                .iter()
                .map(|v| {
                    if let Some(value) = v {
                        value.to_string()
                    } else {
                        "".to_string()
                    }
                })
                .collect::<Vec<String>>(),
            ColumnData::Bool(x) => x
                .iter()
                .map(|v| {
                    if let Some(value) = v {
                        value.to_string()
                    } else {
                        "".to_string()
                    }
                })
                .collect::<Vec<String>>(),
            ColumnData::None => return Err(Errors::InvalidColumnType),
        };
        Ok(result)
    }

    pub fn get_data_by_ix(&self, ix: &Vec<usize>) -> Result<Vec<String>, Errors> {
        let mut data_list: Vec<String> = vec![];
        let all_data = match self.get_all_data() {
            Ok(v) => v,
            Err(err) => {
                return Err(err);
            }
        };
        for i in ix {
            data_list.push(String::from(&all_data[*i]));
        }
        Ok(data_list)
    }

    pub fn count(&self) -> Result<usize, Errors> {
        match self.get_all_data() {
            Ok(v) => Ok(v.len()),
            Err(err) => Err(err),
        }
    }

    pub fn update_val(&mut self, ix: usize, val: String) {
        match self {
            ColumnData::Int(v) => v[ix] = Option::from(val.parse::<i32>().unwrap()),
            ColumnData::Float(v) => v[ix] = Option::from(val.parse::<f32>().unwrap()),
            ColumnData::Str(v) => v[ix] = Option::from(val),
            ColumnData::Bool(v) => v[ix] = Option::from(val.parse::<bool>().unwrap()),
            ColumnData::None => {}
        }
    }

    pub fn delete_val(&mut self, ixs: Vec<usize>) {
        match self {
            ColumnData::Int(v) => {
                *v = v
                    .iter()
                    .enumerate()
                    .filter(|(ix, _)| !ixs.contains(ix))
                    .map(|(_, val)| val.to_owned())
                    .collect::<Vec<Option<i32>>>();
            }
            ColumnData::Float(v) => {
                *v = v
                    .iter()
                    .enumerate()
                    .filter(|(ix, _)| !ixs.contains(ix))
                    .map(|(_, val)| val.to_owned())
                    .collect::<Vec<Option<f32>>>();
            }
            ColumnData::Str(v) => {
                *v = v
                    .iter()
                    .enumerate()
                    .filter(|(ix, _)| !ixs.contains(ix))
                    .map(|(_, val)| val.to_owned())
                    .collect::<Vec<Option<String>>>();
            }
            ColumnData::Bool(v) => {
                *v = v
                    .iter()
                    .enumerate()
                    .filter(|(ix, _)| !ixs.contains(ix))
                    .map(|(_, val)| val.to_owned())
                    .collect::<Vec<Option<bool>>>();
            }
            ColumnData::None => {}
        }
    }
}
