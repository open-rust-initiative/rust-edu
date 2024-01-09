use sqlparser::ast::{ColumnOption, DataType, Statement};
use crate::error::{Result, SQLRiteError};

/// ParsedColumn结构体用于解析SQL语句时获得的每个列的元数据属性
#[derive(PartialEq, Debug)]
pub struct ParsedColumn {
    /// 列的名称
    pub name: String,
    /// 列的数据类型（以字符串格式）
    pub datatype: String,
    /// 表示该列是否是主键
    pub is_pk: bool,
    /// 表示该列是否被声明为NOT NULL
    pub not_null: bool,
    /// 表示该列是否被声明为UNIQUE
    pub is_unique: bool,
}

/// CreateQuery结构体代表一个已经解析的CREATE TABLE查询
#[derive(Debug)]
pub struct CreateQuery {
    /// 表名
    pub table_name: String,
    /// 一个包含ParsedColumn类型的向量，代表列的元数据
    pub columns: Vec<ParsedColumn>,
}

impl CreateQuery {
    // new方法是CreateQuery结构体的构造器，用于从Statement对象创建一个CreateQuery实例。
    pub fn new(statement: &Statement) -> Result<CreateQuery> {
        match statement {
            // 确认语句为 sqlparser::ast:Statement::CreateTable
            Statement::CreateTable {
                name,
                columns,
                constraints: _constraints,
                with_options: _with_options,
                external: _external,
                file_format: _file_format,
                location: _location,
                ..
            } => {
                // 表名
                let table_name = name;
                // 用于储存解析的列
                let mut parsed_columns: Vec<ParsedColumn> = vec![];

                // 遍历解析器Parser::parse:sql 返回的列 
                for col in columns {
                    let name = col.name.to_string();

                    // 检查是否已将 columm 添加到已解析的列中，如果是，则返回错误信息
                    if parsed_columns.iter().any(|col| col.name == name) {
                        return Err(SQLRiteError::Internal(format!(
                            "Duplicate column name: {}",
                            &name
                        )));
                    }

                    //  解析每一列的数据类型，目前只能解析基本数据类型
                    let datatype = match &col.data_type {
                        DataType::SmallInt(_) => "Integer",
                        DataType::Int(_) => "Integer",
                        DataType::BigInt(_) => "Integer",
                        DataType::Boolean => "Bool",
                        DataType::Text => "Text",
                        DataType::Varchar(_bytes) => "Text",
                        DataType::Real => "Real",
                        DataType::Float(_precision) => "Real",
                        DataType::Double => "Real",
                        DataType::Decimal(_precision1, _precision2) => "Real",
                        _ => {
                            eprintln!("not matched on custom type");
                            "Invalid"
                        }
                    };

                    // 检查列是否是主键
                    let mut is_pk: bool = false;
                    // 检查列是否唯一
                    let mut is_unique: bool = false;
                    // 检查列是否 not_null
                    let mut not_null: bool = false;
                    for column_option in &col.options {
                        match column_option.option {
                            ColumnOption::Unique { is_primary } => {
                                // 仅支持整数和文本类型作为 PRIMERY KEY 和 Unique
                                // 只有唯一的情况下才可以是主键，因此主键的处理合并到了Unique处理内
                                if datatype != "Real" && datatype != "Bool" {
                                    is_pk = is_primary;
                                    if is_primary {
                                        // 检查创建的表是否已经有 PRIMARY KEY，如果是，则返回错误信息
                                        if parsed_columns.iter().any(|col| col.is_pk == true) {
                                            return Err(SQLRiteError::Internal(format!(
                                                "Table '{}' has more than one primary key",
                                                &table_name
                                            )));
                                        }
                                        not_null = true;
                                    }
                                    is_unique = true;
                                }
                            }
                            ColumnOption::NotNull => {
                                not_null = true;
                            }
                            _ => (),
                        };
                    }
                    
                    // 将解析的结果放入 parsed_columns 中
                    parsed_columns.push(ParsedColumn {
                        name,
                        datatype: datatype.to_string(),
                        is_pk,
                        not_null,
                        is_unique,
                    });
                }
                // TODO: 处理约束条件（暂未实现）
                for constraint in _constraints {
                    println!("{:?}", constraint);
                }
                // 反回对应的CreateQuery对象
                return Ok(CreateQuery {
                    table_name: table_name.to_string(),
                    columns: parsed_columns,
                });
            }
            // 如果语句不是 sqlparser::ast:Statement::CreateTable，反回错误
            _ => return Err(SQLRiteError::Internal("Error parsing query".to_string())),
        }
    }
}

// 测试create模块正确性
#[cfg(test)]
mod tests {
    use super::*;
    use crate::sql::*;
    use sqlparser::parser::Parser;
    use sqlparser::dialect::SQLiteDialect;

    #[test]
    fn create_table_validate_tablename_test() {
        // sql语句输入
        let sql_input = String::from(
            "CREATE TABLE contacts (
            id INTEGER PRIMARY KEY,
            first_name TEXT NOT NULL,
            last_name TEXT NOT NULl,
            email TEXT NOT NULL UNIQUE
        );",
        );
        // 将sql_input转化成对应的Statement对象
        let expected_table_name = String::from("contacts");
        let dialect = SQLiteDialect {};
        let mut ast = Parser::parse_sql(&dialect, &sql_input).unwrap();
        // Statement应当只有一个
        assert!(ast.len() == 1, "ast has more then one Statement");
        // 取出对应Statement对象
        let query = ast.pop().unwrap();

        match query {
            Statement::CreateTable { .. } => {
                // 用CreateQuery::new方法解析query
                let result = CreateQuery::new(&query);
                match result {
                    // 检查解析的结果是否正确
                    Ok(payload) => {
                        assert_eq!(payload.table_name, expected_table_name);
                    }
                    // 解析出错反回错误信息
                    Err(_) => assert!(
                        false,
                        "an error occured during parsing CREATE TABLE Statement"
                    ),
                }
            }
            _ => (),
        };
    }
}
