use crate::error::{Result, SQLRiteError};
use crate::sql::db::table::Table;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 数据库结构体
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Database {
    /// 数据库的名称(模式名称，而非文件名）
    pub db_name: String,
    /// 数据库中表格的 HashMap
    pub tables: HashMap<String, Table>,
}

impl Database {
    /// 创建一个空的数据库示例如下
    /// ```
    /// let mut db = sql::db::database::Database::new("my_db".to_string());
    /// ```
    pub fn new(db_name: String) -> Self {
        // 通过数据库名称创建新的空数据库
        Database {
            db_name,
            tables: HashMap::new(),
        }
    }

    /// 查询数据库中是否包含以指定表名的表
    pub fn contains_table(&self, table_name: String) -> bool {
        self.tables.contains_key(&table_name)
    }

    /// 如果数据库中包含以指定名称作为表名的表，则返回 sql::db::table::Table 的引用
    pub fn get_table(&self, table_name: String) -> Result<&Table> {
        if let Some(table) = self.tables.get(&table_name) {
            Ok(table)
        } else {
            Err(SQLRiteError::General(String::from("Table not found.")))
        }
    }

    /// 如果数据库中包含以指定名称作为表名的表，则返回 sql::db::table::Table 的可变引用
    pub fn get_table_mut(&mut self, table_name: String) -> Result<&mut Table> {
        if let Some(table) = self.tables.get_mut(&table_name) {
            Ok(table)
        } else {
            Err(SQLRiteError::General(String::from("Table not found.")))
        }
    }
}

// 代码测试模块
#[cfg(test)]
mod tests {
    use super::*;
    use crate::sql::parser::create::CreateQuery;
    use sqlparser::dialect::SQLiteDialect;
    use sqlparser::parser::Parser;

    // 测试创建空数据库功能是否正常
    #[test]
    fn new_database_create_test() {
        let db_name = String::from("my_db");
        let db = Database::new(db_name.to_string());
        assert_eq!(db.db_name, db_name);
    }

    // 测试contains_table函数是否正常
    #[test]
    fn contains_table_test() {
        let db_name = String::from("my_db");
        let mut db = Database::new(db_name.to_string());

        let query_statement = "CREATE TABLE contacts (
            id INTEGER PRIMARY KEY,
            first_name TEXT NOT NULL,
            last_name TEXT NOT NULl,
            email TEXT NOT NULL UNIQUE
        );";
        let dialect = SQLiteDialect {};
        let mut ast = Parser::parse_sql(&dialect, &query_statement).unwrap();
        if ast.len() > 1 {
            panic!("Expected a single query statement, but there are more then 1.")
        }
        let query = ast.pop().unwrap();

        let create_query = CreateQuery::new(&query).unwrap();
        let table_name = &create_query.table_name;
        db.tables
            .insert(table_name.to_string(), Table::new(create_query));

        assert!(db.contains_table("contacts".to_string()));
    }


    // 测试 get_table 相关函数是否正常
    #[test]
    fn get_table_test() {
        let db_name = String::from("my_db");
        let mut db = Database::new(db_name.to_string());

        let query_statement = "CREATE TABLE contacts (
            id INTEGER PRIMARY KEY,
            first_name TEXT NOT NULL,
            last_name TEXT NOT NULl,
            email TEXT NOT NULL UNIQUE
        );";
        let dialect = SQLiteDialect {};
        let mut ast = Parser::parse_sql(&dialect, &query_statement).unwrap();
        if ast.len() > 1 {
            panic!("Expected a single query statement, but there are more then 1.")
        }
        let query = ast.pop().unwrap();

        let create_query = CreateQuery::new(&query).unwrap();
        let table_name = &create_query.table_name;
        db.tables
            .insert(table_name.to_string(), Table::new(create_query));

        // 拿到的table应当和插入的相同，是4列
        let table = db.get_table(String::from("contacts")).unwrap();
        assert_eq!(table.columns.len(), 4);

        let mut table = db.get_table_mut(String::from("contacts")).unwrap();
        table.last_rowid += 1;
        assert_eq!(table.columns.len(), 4);
        // 拿到的table的last_rowid应当是1
        assert_eq!(table.last_rowid, 1);
    }
}
