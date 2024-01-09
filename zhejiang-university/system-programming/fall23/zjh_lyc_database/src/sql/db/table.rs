use crate::error::{Result, SQLRiteError};
use crate::sql::parser::create::CreateQuery;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::rc::Rc;

use prettytable::{Cell as PrintCell, Row as PrintRow, Table as PrintTable, row};

/// 枚举数据类型 DataType，对应sqlite的数据类型
/// Serialize 和 Deserialize 是用于序列化和反序列化的特性
/// PartialEq 用于比较枚举值是否相等
/// Debug 用于打印调试信息
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum DataType {
    Integer,
    Text,
    Real,
    Bool,
    None,
    Invalid,
}
 
impl DataType {
    // 通过对应的cmd反回一个对应的DataType类型
    pub fn new(cmd: String) -> DataType {
        match cmd.to_lowercase().as_ref() {
            "integer" => DataType::Integer,
            "text" => DataType::Text,
            "real" => DataType::Real,
            "bool" => DataType::Bool,
            "none" => DataType::None,
            _ => {
                eprintln!("Invalid data type given {}", cmd);
                return DataType::Invalid;
            }
        }
    }
}

// 为 DataType 枚举实现 Display 特性
impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // 将对应类型模式匹配到对应字符串
        match *self {
            DataType::Integer => f.write_str("Integer"),
            DataType::Text => f.write_str("Text"),
            DataType::Real => f.write_str("Real"),
            DataType::Bool => f.write_str("Boolean"),
            DataType::None => f.write_str("None"),
            DataType::Invalid => f.write_str("Invalid"),
        }
    }
}

///  每个 SQL 表的模式在内存中由以下结构表示
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Table {
    /// 表名
    pub tb_name: String,
    /// 包含各列信息的 HashMap
    pub columns: Vec<Column>,
    /// 包含每一行信息的 HashMap， Rc让一个数据可以拥有多个所有者，RefCell实现编译期可变、不可变引用共存
    pub rows: Rc<RefCell<HashMap<String, Row>>>,
    /// 该表 SQL 索引的 HashMap
    pub indexes: HashMap<String, String>,
    /// 最近插入的 ROWID
    pub last_rowid: i64,
    /// PRIMARY KEY 列名，如果表没有 PRIMARY KEY，则为 -1
    pub primary_key: String,
}

impl Table {
    // 通过CreateQuery的解析结果来创建 SQL 表的方法
    pub fn new(create_query: CreateQuery) -> Self {
        // 取出create_query的表名和columns
        let table_name = create_query.table_name;
        let mut primary_key: String = String::from("-1");
        let columns = create_query.columns;

        let mut table_cols: Vec<Column> = vec![];

        // Rc 是 Rust 的一个智能指针类型，代表"引用计数"（Reference Counted）
        //它使多个所有者能共享同一个数据实例，但仅在单线程环境中，当最后一个 Rc 被销毁时，其内部数据也会被销毁

        // RefCell 是 Rust 的一个智能指针类型，提供了运行时（而非编译时）的借用检查，常用于需要改变数据但又不能或不想使用可变引用的情况
        // 它允许在运行时对内部数据进行可变或不可变借用，即使在拥有不可变引用的情况下也可以改变内部数据
        let table_rows: Rc<RefCell<HashMap<String, Row>>> = Rc::new(RefCell::new(HashMap::new()));
        // 枚举每一列
        for col in &columns {
            let col_name = &col.name;
            // 如果当前是primary key，那么获取当前的列名作为primary_key值
            if col.is_pk {
                primary_key = col_name.to_string();
            }
            // 把当前枚举的列创建一个新的Column对象放入table_cols中
            table_cols.push(Column::new(
                col_name.to_string(),
                col.datatype.to_string(),
                col.is_pk,
                col.not_null,
                col.is_unique,
            ));
            // 将col.datatype创建成DataType类型进行模式匹配
            // 对于除了Invalid和None两种类型外，其它类型都往table_rows中插入一个 key为列名，value为对应类型的BTreeMap的键值对
            match DataType::new(col.datatype.to_string()) {
                DataType::Integer => table_rows
                    .clone()
                    .borrow_mut()
                    .insert(col.name.to_string(), Row::Integer(BTreeMap::new())),
                DataType::Real => table_rows
                    .clone()
                    .borrow_mut()
                    .insert(col.name.to_string(), Row::Real(BTreeMap::new())),
                DataType::Text => table_rows
                    .clone()
                    .borrow_mut()
                    .insert(col.name.to_string(), Row::Text(BTreeMap::new())),
                DataType::Bool => table_rows
                    .clone()
                    .borrow_mut()
                    .insert(col.name.to_string(), Row::Bool(BTreeMap::new())),
                DataType::Invalid => table_rows
                    .clone()
                    .borrow_mut()
                    .insert(col.name.to_string(), Row::None),
                DataType::None => table_rows
                    .clone()
                    .borrow_mut()
                    .insert(col.name.to_string(), Row::None),
            };
        }
        // 最后创建对应的Table对象作为反回结果
        Table {
            tb_name: table_name,
            columns: table_cols,
            rows: table_rows,
            indexes: HashMap::new(),
            last_rowid: 0,
            primary_key: primary_key,
        }
    }

    /// 返回一个 bool 类型，表示是否存在特定名称的列
    pub fn contains_column(&self, column: String) -> bool {
        self.columns.iter().any(|col| col.column_name == column)
    }

    /// 如果表中包含以指定键作为列名的列，则返回 sql::db::table::Column 的不可变引用。
    pub fn get_column(&mut self, column_name: String) -> Result<&Column> {
        if let Some(column) = self
            .columns
            .iter()
            .filter(|c| c.column_name == column_name)
            .collect::<Vec<&Column>>()
            .first()
        {
            Ok(column)
        } else {
            Err(SQLRiteError::General(String::from("Column not found.")))
        }
    }

    /// 如果表中包含以指定键作为列名的列，则返回 sql::db::table::Column 的可变引用
    pub fn get_column_mut<'a>(&mut self, column_name: String) -> Result<&mut Column> {
        for elem in self.columns.iter_mut() {
            if elem.column_name == column_name {
                return Ok(elem);
            }
        }
        Err(SQLRiteError::General(String::from("Column not found.")))
    }

    /// 验证插入的列和值是否违反 UNIQUE 约束
    /// 值得注意的是，PRIMARY KEY 列会自动成为 UNIQUE 列
    pub fn validate_unique_constraint(
        &mut self,
        cols: &Vec<String>,
        values: &Vec<String>,
    ) -> Result<()> {
        // 枚举所有的列名
        for (idx, name) in cols.iter().enumerate() {
            // 获取表中对应列的可变引用
            let column = self.get_column_mut(name.to_string()).unwrap();
            // println!(
            //     "name: {} | is_pk: {} | is_unique: {}, not_null: {}",
            //     name, column.is_pk, column.is_unique, column.not_null
            // );

            // 如果对应的列是唯一的
            if column.is_unique {
                let col_idx = &column.index;
                // 检查name和获取的列名是否一致
                if *name == *column.column_name {
                    let val = &values[idx];
                    // 将Index进行模式匹配
                    match col_idx {
                        // 如果对应 Integer 的BTreeMap
                        Index::Integer(index) => {
                            // 检查BTreeMap是否已经存在相应的元素
                            if index.contains_key(&val.parse::<i32>().unwrap()) {
                                // 存在反回错误信息
                                return Err(SQLRiteError::General(format!(
                                    "Error: unique constraint violation for column {}.
                        Value {} already exists for column {}",
                                    *name, val, *name
                                )));
                            }
                        }
                        // 如果对应 Text 的BTreeMap
                        Index::Text(index) => {
                            // 检查BTreeMap是否已经存在相应的元素
                            if index.contains_key(val) {
                                // 存在反回错误信息
                                return Err(SQLRiteError::General(format!(
                                    "Error: unique constraint violation for column {}.
                        Value {} already exists for column {}",
                                    *name, val, *name
                                )));
                            }
                        }
                        // 如果对应空，说明列不存在
                        Index::None => {
                            return Err(SQLRiteError::General(format!(
                                "Error: cannot find index for column {}",
                                name
                            )));
                        }
                    };
                }
            }
        }
        // 不存在矛盾，反回成功
        return Ok(());
    }

    /// 将所有值插入到相应的列中，对所有 ROWS 使用编码的索引 ROWID
    /// 每个表都会持续跟踪 last_rowid 以便能知道下一个位置该在哪里
    /// 这种数据结构的一个限制是一次只能有一个写事务，否则可能会在 last_rowid.println! 上有竞争
    /// SQLite 同一时间也只允许一次写事务，所以此处宽松的处理方式无伤大雅
    pub fn insert_row(&mut self, cols: &Vec<String>, values: &Vec<String>) {
        let mut next_rowid = self.last_rowid + i64::from(1);

        // 检查表是否存在主键
        if self.primary_key != "-1" {
            // 检查主键是否在 INSERT 语句的列中
            // 如果不在，则将 next_rowid 分配给它
            if !cols.iter().any(|col| col == &self.primary_key) {
                let rows_clone = Rc::clone(&self.rows);
                let mut row_data = rows_clone.as_ref().borrow_mut();
                let mut table_col_data = row_data.get_mut(&self.primary_key).unwrap();

                // 基于主键获取列
                let column_headers = self.get_column_mut(self.primary_key.to_string()).unwrap();

                // 如果列的索引存在，则获取
                let col_index = column_headers.get_mut_index();

                // 如果行存在主键并且是整数类型，则自动分配
                match &mut table_col_data {
                    Row::Integer(tree) => {
                        let val = next_rowid as i32;
                        tree.insert(next_rowid.clone(), val);
                        if let Index::Integer(index) = col_index {
                            index.insert(val, next_rowid.clone());
                        }
                    }
                    _ => (),
                }
            } else {
                // 如果 INSERT 语句的列中存在主键列，则从值的部分获取它，并分配(赋值)给 next_rowid
                // 此外，下一个 rowid 应该从最后一个 ROWID 递增
                let rows_clone = Rc::clone(&self.rows);
                let mut row_data = rows_clone.as_ref().borrow_mut();
                let mut table_col_data = row_data.get_mut(&self.primary_key).unwrap();

                // 同样，这只对 INTEGER 类型的主键有效
                match &mut table_col_data {
                    Row::Integer(_) => {
                        for i in 0..cols.len() {
                            // 获取列名
                            let key = &cols[i];
                            if key == &self.primary_key {
                                let val = &values[i];
                                next_rowid = val.parse::<i64>().unwrap();
                            }
                        }
                    }
                    _ => (),
                }
            }
        }

        // 这一部分检查 INSERT 语句中是否缺少表中的某个或某些列
        // 如果有，将"Null"添加到该列中
        // 这样做是为了防止由于行的长度不同而引发错误
        let column_names = self
            .columns
            .iter()
            .map(|col| col.column_name.to_string())
            .collect::<Vec<String>>();
        let mut j: usize = 0;
        // 对 INSERT 语句中的每一列
        for i in 0..column_names.len() {
            let mut val = String::from("Null");
            let key = &column_names[i];

            if let Some(key) = &cols.get(j) {
                if &key.to_string() == &column_names[i] {
                    // 获取列名
                    val = values[j].to_string();
                    j += 1;
                } else {
                    if &self.primary_key == &column_names[i] {
                        continue;
                    }
                }
            } else {
                if &self.primary_key == &column_names[i] {
                    continue;
                }
            }

            let rows_clone = Rc::clone(&self.rows);
            let mut row_data = rows_clone.as_ref().borrow_mut();
            let mut table_col_data = row_data.get_mut(key).unwrap();

            let column_headers = self.get_column_mut(key.to_string()).unwrap();

            let col_index = column_headers.get_mut_index();

            match &mut table_col_data {
                Row::Integer(tree) => {
                    let val = val.parse::<i32>().unwrap();
                    tree.insert(next_rowid.clone(), val);
                    if let Index::Integer(index) = col_index {
                        index.insert(val, next_rowid.clone());
                    }
                }
                Row::Text(tree) => {
                    tree.insert(next_rowid.clone(), val.to_string());
                    if let Index::Text(index) = col_index {
                        index.insert(val.to_string(), next_rowid.clone());
                    }
                }
                Row::Real(tree) => {
                    let val = val.parse::<f32>().unwrap();
                    tree.insert(next_rowid.clone(), val);
                }
                Row::Bool(tree) => {
                    let val = val.parse::<bool>().unwrap();
                    tree.insert(next_rowid.clone(), val);
                }
                Row::None => panic!("None data Found"),
            }
        }
        self.last_rowid = next_rowid;
    }

    /// 以适合的格式将表格属性打印到标准输出中
    /// ```
    /// let table = Table::new(payload);
    /// table.print_table_schema();
    ///
    /// Prints to standard output:
    ///    +-------------+-----------+-------------+--------+----------+
    ///    | Column Name | Data Type | PRIMARY KEY | UNIQUE | NOT NULL |
    ///    +-------------+-----------+-------------+--------+----------+
    ///    | id          | Integer   | true        | true   | true     |
    ///    +-------------+-----------+-------------+--------+----------+
    ///    | name        | Text      | false       | true   | false    |
    ///    +-------------+-----------+-------------+--------+----------+
    ///    | email       | Text      | false       | false  | false    |
    ///    +-------------+-----------+-------------+--------+----------+
    /// ```
    pub fn print_table_schema(&self) -> Result<usize> {
        // 定义PrintTable对象
        let mut table = PrintTable::new();
        // 添加表头
        table.add_row(row![
            "Column Name",
            "Data Type",
            "PRIMARY KEY",
            "UNIQUE",
            "NOT NULL"
        ]);
        // 取出所有的列的属性
        for col in &self.columns {
            table.add_row(row![
                col.column_name,
                col.datatype,
                col.is_pk,
                col.is_unique,
                col.not_null
            ]);
        }
        // 打印结果
        let lines = table.printstd();
        Ok(lines)
    }

    /// 以适当的格式将表格数据打印到标准输出中
    /// ```
    /// let db_table = db.get_table_mut(table_name.to_string()).unwrap();
    /// db_table.print_table_data();
    ///
    /// Prints to standard output:
    ///     +----+---------+------------------------+
    ///     | id | name    | email                  |
    ///     +----+---------+------------------------+
    ///     | 1  | "Jack"  | "jack@mail.com"        |
    ///     +----+---------+------------------------+
    ///     | 10 | "Bob"   | "bob@main.com"         |
    ///     +----+---------+------------------------+
    ///     | 11 | "Bill"  | "bill@main.com"        |
    ///     +----+---------+------------------------+
    /// ```
    pub fn print_table_data(&self) {
        // 定义PrintTable对象
        let mut print_table = PrintTable::new();
        // 取出所有列名集合
        let column_names = self
            .columns
            .iter()
            .map(|col| col.column_name.to_string())
            .collect::<Vec<String>>();
        // 将列名集合创建成PrintRow对象
        let header_row = PrintRow::new(
            column_names
                .iter()
                .map(|col| PrintCell::new(&col))
                .collect::<Vec<PrintCell>>(),
        );
        // 用Rc::clone创建self.rows的智能指针
        let rows_clone = Rc::clone(&self.rows);
        // 从rows_clone的引用中获取对应的引用
        let row_data = rows_clone.as_ref().borrow();
        // 获取row_data中第一列数据的引用
        let first_col_data = row_data
            .get(&self.columns.first().unwrap().column_name)
            .unwrap();
        // 行数即为第一列数据的个数
        let num_rows = first_col_data.count();
        // 创建一个可变数组，用来存储PrintRow，里面有num_rows个空的PrintRow对象
        let mut print_table_rows: Vec<PrintRow> = vec![PrintRow::new(vec![]); num_rows];
        // 枚举列名
        for col_name in &column_names {
            // 从row_data中根据对应的列名获取相应的Row引用，表示列的值
            let col_val = row_data
                .get(col_name)
                .expect("Can't find any rows with the given column");
            // 获取col_val的序列化结果，Vec<String>
            let columns: Vec<String> = col_val.get_serialized_col_data();
            // 枚举行号
            for i in 0..num_rows {
                // 根据行号取出对应的字符串值引用，如果不为空，将其（否则将空）加入到print_table_rows的对应行中
                if let Some(cell) = &columns.get(i) {
                    print_table_rows[i].add_cell(PrintCell::new(cell));
                } else {
                    print_table_rows[i].add_cell(PrintCell::new(""));
                }
            }
        }
        // 首先将表头加入到 print_table
        print_table.add_row(header_row);
        // 然后将print_table_rows中所有的row加入到print_table
        for row in print_table_rows {
            print_table.add_row(row);
        }
        // 打印结果
        print_table.printstd();
    }
}

/// 每个表中每个 SQL 列都在内存中用以下结构表示
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Column {
    /// 列名
    pub column_name: String,
    /// 列的数据类型
    pub datatype: DataType,
    /// 是否为主键
    pub is_pk: bool,
    /// 是否有非空约束
    pub not_null: bool,
    /// 是否有唯一约束
    pub is_unique: bool,
    /// 是否已经被索引
    pub is_indexed: bool,
    /// Index 对应相应的 BTreeMap
    pub index: Index,
}

impl Column {
    // 根据参数创建对应的Column对象
    pub fn new(
        name: String,
        datatype: String,
        is_pk: bool,
        not_null: bool,
        is_unique: bool,
    ) -> Self {
        // 获取数据类型，创建相应的DataType
        let dt = DataType::new(datatype);
        let index = match dt {  // 模式匹配DataType，创建相应的Index
            DataType::Integer => Index::Integer(BTreeMap::new()),
            DataType::Bool => Index::None,
            DataType::Text => Index::Text(BTreeMap::new()),
            DataType::Real => Index::None,
            DataType::Invalid => Index::None,
            DataType::None => Index::None,
        };
        // 反回创建的对象
        Column {
            column_name: name,
            datatype: dt,
            is_pk,
            not_null,
            is_unique,
            is_indexed: if is_pk { true } else { false },
            index,
        }
    }
    // 获取Index的可变引用
    pub fn get_mut_index(&mut self) -> &mut Index {
        return &mut self.index;
    }
}

/// 每个表中每个 SQL 列索引都在内存中用以下结构表示
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum Index {
    Integer(BTreeMap<i32, i64>),
    Text(BTreeMap<String, i64>),
    None,
}

/// 每个 SQL 行在内存中用如下结构体表示
/// 一个枚举类型，代表 BTreeMap 中的每种可用类型，使用 ROWID 作为键，每种相应类型作为值
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum Row {
    Integer(BTreeMap<i64, i32>),
    Text(BTreeMap<i64, String>),
    Real(BTreeMap<i64, f32>),
    Bool(BTreeMap<i64, bool>),
    None,
}

impl Row {
    // 获取序列化的列数据，即将一行数据转化成一个Vec<String>
    fn get_serialized_col_data(&self) -> Vec<String> {
        match self {
            Row::Integer(cd) => cd.iter().map(|(_i, v)| v.to_string()).collect(),
            Row::Real(cd) => cd.iter().map(|(_i, v)| v.to_string()).collect(),
            Row::Text(cd) => cd.iter().map(|(_i, v)| v.to_string()).collect(),
            Row::Bool(cd) => cd.iter().map(|(_i, v)| v.to_string()).collect(),
            Row::None => panic!("Found None in columns"),
        }
    }
    // 获取当前行数据的个数
    fn count(&self) -> usize {
        match self {
            Row::Integer(cd) => cd.len(),
            Row::Real(cd) => cd.len(),
            Row::Text(cd) => cd.len(),
            Row::Bool(cd) => cd.len(),
            Row::None => panic!("Found None in columns"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlparser::dialect::SQLiteDialect;
    use sqlparser::parser::Parser;

    #[test]
    fn datatype_display_trait_test() {
        let integer = DataType::Integer;
        let text = DataType::Text;
        let real = DataType::Real;
        let boolean = DataType::Bool;
        let none = DataType::None;
        let invalid = DataType::Invalid;

        assert_eq!(format!("{}", integer), "Integer");
        assert_eq!(format!("{}", text), "Text");
        assert_eq!(format!("{}", real), "Real");
        assert_eq!(format!("{}", boolean), "Boolean");
        assert_eq!(format!("{}", none), "None");
        assert_eq!(format!("{}", invalid), "Invalid");
    }

    #[test]
    fn create_new_table_test() {
        let query_statement = "CREATE TABLE contacts (
            id INTEGER PRIMARY KEY,
            first_name TEXT NOT NULL,
            last_name TEXT NOT NULl,
            email TEXT NOT NULL UNIQUE,
            active BOOL,
            score REAL
        );";
        let dialect = SQLiteDialect {};
        let mut ast = Parser::parse_sql(&dialect, &query_statement).unwrap();
        if ast.len() > 1 {
            panic!("Expected a single query statement, but there are more then 1.")
        }
        let query = ast.pop().unwrap();

        let create_query = CreateQuery::new(&query).unwrap();

        let table = Table::new(create_query);

        assert_eq!(table.columns.len(), 6);
        assert_eq!(table.last_rowid, 0);

        let id_column = "id".to_string();
        if let Some(column) = table
            .columns
            .iter()
            .filter(|c| c.column_name == id_column)
            .collect::<Vec<&Column>>()
            .first()
        {
            assert_eq!(column.is_pk, true);
            assert_eq!(column.datatype, DataType::Integer);
        } else {
            panic!("column not found");
        }
    }

    #[test]
    fn print_table_schema_test() {
        let query_statement = "CREATE TABLE contacts (
            id INTEGER PRIMARY KEY,
            first_name TEXT NOT NULL,
            last_name TEXT NOT NULl
        );";
        let dialect = SQLiteDialect {};
        let mut ast = Parser::parse_sql(&dialect, &query_statement).unwrap();
        if ast.len() > 1 {
            panic!("Expected a single query statement, but there are more then 1.")
        }
        let query = ast.pop().unwrap();

        let create_query = CreateQuery::new(&query).unwrap();

        let table = Table::new(create_query);
        let lines_printed = table.print_table_schema();
        assert_eq!(lines_printed, Ok(9));
    }
}
