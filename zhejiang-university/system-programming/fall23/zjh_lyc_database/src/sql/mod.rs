pub mod parser;
pub mod db;

use parser::create::CreateQuery;
use parser::insert::InsertQuery;

use sqlparser::ast::Statement;
use sqlparser::dialect::SQLiteDialect;
use sqlparser::parser::{Parser, ParserError};

use crate::error::{Result, SQLRiteError};
use crate::sql::db::database::Database;
use crate::sql::db::table::Table;

#[derive(Debug, PartialEq)]
pub enum SQLCommand {
    Insert(String),
    Delete(String),
    Update(String),
    CreateTable(String),
    Select(String),
    Unknown(String),
}

impl SQLCommand {
    pub fn new(command: String) -> SQLCommand {
        let v = command.split(" ").collect::<Vec<&str>>();
        match v[0] {
            "insert" => SQLCommand::Insert(command),
            "update" => SQLCommand::Update(command),
            "delete" => SQLCommand::Delete(command),
            "create" => SQLCommand::CreateTable(command),
            "select" => SQLCommand::Select(command),
            _ => SQLCommand::Unknown(command),
        }
    }
}

/// 使用 sqlparser-rs 对 SQL 语句执行初始解析
pub fn process_command(query: &str, db: &mut Database) -> Result<String> {
    let dialect = SQLiteDialect {};
    let message: String;
    let mut ast = Parser::parse_sql(&dialect, &query).map_err(SQLRiteError::from)?;

    if ast.len() > 1 {
        return Err(SQLRiteError::SqlError(ParserError::ParserError(format!(
            "Expected a single query statement, but there are {}",
            ast.len()
        ))));
    }

    let query = ast.pop().unwrap();

    // 目前只实现一些基本的 SQL 语句
    match query {
        Statement::CreateTable { .. } => {
            let create_query = CreateQuery::new(&query);
            match create_query {
                // 获取有用信息
                Ok(payload) => {
                    let table_name = payload.table_name.clone();
                    // 解析 CREATE TABLE 后查询表是否已经存在
                    match db.contains_table(table_name.to_string()) {
                        true => {
                            return Err(SQLRiteError::Internal(
                                "Cannot create, table already exists.".to_string(),
                            ));
                        }
                        false => {
                            let table = Table::new(payload);
                            let _ = table.print_table_schema();
                            db.tables.insert(table_name.to_string(), table);
                            message = String::from("CREATE TABLE Statement executed.");
                        }
                    }
                }
                Err(err) => return Err(err),
            }
        }
        Statement::Insert { .. } => {
            let insert_query = InsertQuery::new(&query);
            match insert_query {
                Ok(payload) => {
                    let table_name = payload.table_name;
                    let columns = payload.columns;
                    let values = payload.rows;

                    // println!("table_name = {:?}\n cols = {:?}\n vals = {:?}", table_name, columns, values);
                    // 检查表是否存在
                    match db.contains_table(table_name.to_string()) {
                        true => {
                            let db_table = db.get_table_mut(table_name.to_string()).unwrap();
                            // 检查表中是否已经有要 INSERT 的所有内容 (columns)
                            match columns
                                .iter()
                                .all(|column| db_table.contains_column(column.to_string()))
                            {
                                true => {
                                    for value in &values {
                                        // 检查查询 (SQL语句) 中的列数是否与值的数量相同
                                        if columns.len() != value.len() {
                                            return Err(SQLRiteError::Internal(format!(
                                                "{} values for {} columns",
                                                value.len(),
                                                columns.len()
                                            )));
                                        }
                                        match db_table.validate_unique_constraint(&columns, value) {
                                            Ok(()) => {
                                                // 没有违反唯一性约束，继续插入行
                                                db_table.insert_row(&columns, &value);
                                            }
                                            Err(err) => {
                                                return Err(SQLRiteError::Internal(format!(
                                                    "Unique key constaint violation: {}",
                                                    err
                                                )))
                                            }
                                        }
                                    }
                                }
                                false => {
                                    return Err(SQLRiteError::Internal(
                                        "Cannot insert, some of the columns do not exist"
                                            .to_string(),
                                    ));
                                }
                            }
                            db_table.print_table_data();
                        }
                        false => {
                            return Err(SQLRiteError::Internal("Table doesn't exist".to_string()))
                        }
                    }
                }
                Err(err) => return Err(err),
            }

            message = String::from("INSERT Statement executed.")
        }
        Statement::Query(_query) => message = String::from("SELECT Statement executed."),
        Statement::Delete { .. } => message = String::from("DELETE Statement executed."),
        _ => {
            return Err(SQLRiteError::NotImplemented(
                "SQL Statement not supported yet.".to_string(),
            ))
        }
    };

    Ok(message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_command_select_test() {
        let inputed_query = String::from("SELECT * from users;");
        let mut db = Database::new("tempdb".to_string());

        let _ = match process_command(&inputed_query, &mut db) {
            Ok(response) => assert_eq!(response, "SELECT Statement executed."),
            Err(err) => {
                eprintln!("Error: {}", err);
                assert!(false)
            }
        };
    }

    #[test]
    fn process_command_insert_test() {
        // 创建临时数据库
        let mut db = Database::new("tempdb".to_string());

        // 创建临时 table
        let query_statement = "CREATE TABLE users (
            id INTEGER PRIMARY KEY,
            name TEXT
        );";
        let dialect = SQLiteDialect {};
        let mut ast = Parser::parse_sql(&dialect, &query_statement).unwrap();
        if ast.len() > 1 {
            panic!("Expected a single query statement, but there are more then 1.")
        }
        let query = ast.pop().unwrap();
        let create_query = CreateQuery::new(&query).unwrap();

        // 将 table 插入数据库
        db.tables.insert(
            create_query.table_name.to_string(),
            Table::new(create_query),
        );

        // 向 table 中插入数据
        let insert_query = String::from("INSERT INTO users (name) Values ('josh');");
        let _ = match process_command(&insert_query, &mut db) {
            Ok(response) => assert_eq!(response, "INSERT Statement executed."),
            Err(err) => {
                eprintln!("Error: {}", err);
                assert!(false)
            }
        };
    }

    #[test]
    fn process_command_insert_no_pk_test() {
        let mut db = Database::new("tempdb".to_string());

        let query_statement = "CREATE TABLE users (
            name TEXT
        );";
        let dialect = SQLiteDialect {};
        let mut ast = Parser::parse_sql(&dialect, &query_statement).unwrap();
        if ast.len() > 1 {
            panic!("Expected a single query statement, but there are more then 1.")
        }
        let query = ast.pop().unwrap();
        let create_query = CreateQuery::new(&query).unwrap();

        db.tables.insert(
            create_query.table_name.to_string(),
            Table::new(create_query),
        );

        let insert_query = String::from("INSERT INTO users (name) Values ('josh');");
        let _ = match process_command(&insert_query, &mut db) {
            Ok(response) => assert_eq!(response, "INSERT Statement executed."),
            Err(err) => {
                eprintln!("Error: {}", err);
                assert!(false)
            }
        };
    }

    #[test]
    fn process_command_delete_test() {
        let inputed_query = String::from("DELETE FROM users WHERE id=1;");
        let mut db = Database::new("tempdb".to_string());

        let _ = match process_command(&inputed_query, &mut db) {
            Ok(response) => assert_eq!(response, "DELETE Statement executed."),
            Err(err) => {
                eprintln!("Error: {}", err);
                assert!(false)
            }
        };
    }

    #[test]
    fn process_command_not_implemented_test() {
        let inputed_query = String::from("UPDATE users SET name='josh' where id=1;");
        let mut db = Database::new("tempdb".to_string());
        let expected = Err(SQLRiteError::NotImplemented(
            "SQL Statement not supported yet.".to_string(),
        ));

        let result = process_command(&inputed_query, &mut db).map_err(|e| e);
        assert_eq!(result, expected);
    }
}
