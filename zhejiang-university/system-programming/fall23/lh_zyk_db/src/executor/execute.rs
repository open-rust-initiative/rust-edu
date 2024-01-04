use super::super::sql_analyzer::types::*;
use super::super::storage::StoreUtil;
use super::types::*;
use std::fmt::Display;
use tabled::settings::style::{HorizontalLine, VerticalLine};
use tabled::{builder::Builder, settings::Style};

fn compare_sqlvalue(sqlvalue1: &SqlValue, sqlvalue2: &SqlValue, cmp_opt: &CmpOpt) -> bool {
    //Determine whether the relationship between two values meets the input criteria
    match sqlvalue1 {
        SqlValue::Int(value1) => {
            match sqlvalue2 {
                //Compare int
                SqlValue::Int(value2) => match cmp_opt {
                    CmpOpt::Eq => value1 == value2,
                    CmpOpt::Ge => value1 >= value2,
                    CmpOpt::Gt => value1 > value2,
                    CmpOpt::Le => value1 <= value2,
                    CmpOpt::Lt => value1 < value2,
                    CmpOpt::Ne => value1 != value2,
                },
                _ => {
                    return false;
                }
            }
        }
        SqlValue::String(value1) => {
            match sqlvalue2 {
                //Compare String
                SqlValue::String(value2) => match cmp_opt {
                    CmpOpt::Eq => value1 == value2,
                    CmpOpt::Ge => value1 >= value2,
                    CmpOpt::Gt => value1 > value2,
                    CmpOpt::Le => value1 <= value2,
                    CmpOpt::Lt => value1 < value2,
                    CmpOpt::Ne => value1 != value2,
                },
                _ => {
                    return false;
                }
            }
        }
        _ => {
            return false;
        }
    }
}

fn compare_condition(wc: WhereConstraint, record: &RowValue, record_names: &Vec<String>) -> bool {
    //Based on the input conditions, judge whether the record meets the conditions
    match wc {
        WhereConstraint::Constrait(name, cmp_opt, sql_value) => {
            // Handle Constrait case
            if !record_names.contains(&name) {
                return false;
            } else {
                // Find the index of 'name' in 'record_name'
                let index = record_names
                    .iter()
                    .position(|r_name| r_name == &name)
                    .unwrap();
                // Retrieve the SqlValue from 'record' using the index
                let record_value = &record.values[index];

                // Compare 'record_value' with 'sql_value' based on 'cmp_opt'
                compare_sqlvalue(&record_value, &sql_value, &cmp_opt)
            }
        }
        WhereConstraint::Not(wc_box) => {
            // Handle Constrait case
            // let mut result: Vec<SelectCriteria> = Vec::new();
            let new_wc = *wc_box.clone();
            if !compare_condition(new_wc, &record, &record_names) {
                return true;
            } else {
                return false;
            }
        }
        WhereConstraint::And(left_wc, right_wc) => {
            let new_left_wc: WhereConstraint = *left_wc.clone();
            let new_right_wc: WhereConstraint = *right_wc.clone();
            if compare_condition(new_left_wc, &record, &record_names)
                && compare_condition(new_right_wc, &record, &record_names)
            {
                return true;
            } else {
                return false;
            }
        }
        WhereConstraint::Or(left_wc, right_wc) => {
            let new_left_wc: WhereConstraint = *left_wc.clone();
            let new_right_wc: WhereConstraint = *right_wc.clone();
            if compare_condition(new_left_wc, &record, &record_names)
                || compare_condition(new_right_wc, &record, &record_names)
            {
                return true;
            } else {
                return false;
            }
        }
    }
}
fn compare_name(names_insert: &Vec<String>, columns: &Vec<Column>) -> Option<String> {
    //Determine if the input column name exists in the table
    let mut names_columns: Vec<String> = Vec::new();
    for column in columns {
        names_columns.push(column.name.clone());
    }
    for name_insert in names_insert {
        if !names_columns.contains(name_insert) {
            return Some(name_insert.clone());
        }
    }
    None
}

fn get_newrol(
    names_insert: Vec<String>,
    columns: &Vec<Column>,
    value: RowValue,
) -> Result<RowValue, QueryExecutionError> {
    //Arrange and complete input values according to column names, ensuring that they are in the same order as the data in the table
    match compare_name(&names_insert, columns) {
        None => {
            // Create an iterator over the names_table
            let mut row_values: Vec<SqlValue> = Vec::new();
            // let name_columns: Vec<String> = columns.iter().map(|column: &Column| column.name.clone()).collect();
            for column in columns {
                if let Some(index) = names_insert
                    .iter()
                    .position(|name_insert| name_insert == &column.name)
                {
                    // row_value.push(value.clone().values[index]);
                    let values: &Vec<SqlValue> = &value.values;
                    match &values[index] {
                        SqlValue::String(row_value) => {
                            row_values.push(SqlValue::String(row_value.clone()));
                        }
                        SqlValue::Int(row_value) => {
                            row_values.push(SqlValue::Int(*row_value));
                        }
                        SqlValue::Unknown => {
                            row_values.push(SqlValue::Unknown);
                        }
                    }
                } else {
                    match column.type_info {
                        SqlType::String => row_values.push(SqlValue::String("NULL".to_string())),
                        SqlType::Int => row_values.push(SqlValue::Int(0)),
                        SqlType::Unknown => row_values.push(SqlValue::Unknown),
                    }
                }
            }
            let rowvalues: RowValue = RowValue { values: row_values };
            Ok(rowvalues)
        }
        Some(name_insert) => return Err(QueryExecutionError::ColumnDoesNotExist(name_insert)),
    }
}

impl SqlTable {
    /// used to create a new empty table
    pub fn new(columns: ColumnInfo) -> SqlTable {
        Self {
            columns,
            rows: Vec::new(),
        }
    }
}

impl Executable for CreateStatement {
    //Create a new table
    fn check_and_execute(
        self,
        storage_util: StoreUtil,
    ) -> Result<ExecuteResponse, QueryExecutionError> {
        let name = self.table.clone();
        let columns_infos = self.columns;
        let table = SqlTable::new(columns_infos);
        match storage_util.save(name.clone(), &table) {
            Ok(()) => Ok(ExecuteResponse::Message(format!(
                "save {} successful",
                name
            ))),
            Err(_) => Err(QueryExecutionError::TableSavefail(name)),
        }
    }
}

impl Executable for DropStatement {
    //delete a table
    fn check_and_execute(
        self,
        storage_util: StoreUtil,
    ) -> Result<ExecuteResponse, QueryExecutionError> {
        let name = self.table.clone();
        match storage_util.delete(&name) {
            Ok(()) => Ok(ExecuteResponse::Message(format!(
                "delete {} successful",
                name
            ))),
            Err(err) => match err.kind() {
                std::io::ErrorKind::NotFound => Err(QueryExecutionError::TableNotFound(name)),
                _ => Err(QueryExecutionError::TableDeletefail(name)),
            },
        }
    }
}

impl Executable for InsertStatement {
    // insert a record
    fn check_and_execute(
        self,
        storage_util: StoreUtil,
    ) -> Result<ExecuteResponse, QueryExecutionError> {
        let name = self.table.clone();
        match storage_util.load(name.clone()) {
            Ok(table) => {
                let columns: Vec<Column> = table.columns;
                let mut rows: Vec<RowValue> = table.rows;
                // let name_columns: Vec<String> = columns.iter().map(|column: &Column| column.name.clone()).collect();
                match self.columns {
                    Some(name_insert) => match get_newrol(name_insert, &columns, self.values) {
                        Ok(rowvalue) => {
                            rows.push(rowvalue);
                            let new_table = SqlTable { columns, rows };
                            match storage_util.save(name.clone(), &new_table) {
                                Ok(()) => Ok(ExecuteResponse::Message(format!(
                                    "save {} successful",
                                    name
                                ))),
                                Err(_) => Err(QueryExecutionError::TableSavefail(name)),
                            }
                        }
                        Err(err) => Err(err),
                    },
                    None => {
                        let rowvalue = self.values;
                        rows.push(rowvalue);
                        let new_table = SqlTable { columns, rows };
                        match storage_util.save(name.clone(), &new_table) {
                            Ok(()) => Ok(ExecuteResponse::Message(format!(
                                "save {} successful",
                                name
                            ))),
                            Err(_) => Err(QueryExecutionError::TableSavefail(name)),
                        }
                    }
                }
            }
            Err(err) => match err.kind() {
                std::io::ErrorKind::NotFound => Err(QueryExecutionError::TableNotFound(name)),
                _ => Err(QueryExecutionError::TableOpenfail(name)),
            },
        }
    }
}

impl Executable for DeleteStatement {
    // delete a record
    fn check_and_execute(
        self,
        storage_util: StoreUtil,
    ) -> Result<ExecuteResponse, QueryExecutionError> {
        let table_name = self.table.clone();
        let wc = match self.constraints {
            Some(wc_tmp) => wc_tmp,
            None => {
                return Err(QueryExecutionError::NoConditionsObtained());
            }
        };
        match storage_util.load(table_name.clone()) {
            //check the result of loading
            Ok(table) => {
                let rows_old: Vec<RowValue> = table.rows;
                let columns_old = table.columns;
                let mut name_old: Vec<String> = Vec::new();
                for column_old in &columns_old {
                    name_old.push(column_old.name.clone());
                }
                let mut rows_new: Vec<RowValue> = Vec::new();
                for row_old in rows_old {
                    if compare_condition(wc.clone(), &row_old, &name_old) {
                        continue;
                    } else {
                        rows_new.push(row_old.clone());
                    }
                }
                let table_new = SqlTable {
                    columns: columns_old,
                    rows: rows_new,
                };
                match storage_util.save(table_name.clone(), &table_new) {
                    Ok(_) => Ok(ExecuteResponse::Message("delete final".to_string())),
                    Err(_) => Err(QueryExecutionError::TableSavefail(table_name)),
                }
            }
            Err(err) => match err.kind() {
                std::io::ErrorKind::NotFound => Err(QueryExecutionError::TableNotFound(table_name)),
                _ => Err(QueryExecutionError::TableOpenfail(table_name)),
            },
        }
    }
}

impl Executable for SelectStatement {
    // select recodes in table
    fn check_and_execute(
        self,
        storage_util: StoreUtil,
    ) -> Result<ExecuteResponse, QueryExecutionError> {
        let table_name = self.table.clone();
        match storage_util.load(table_name.clone()) {
            //check the result of loading
            Ok(table) => {
                let columns: Vec<Column> = table.columns;
                let columns_clone = columns.clone();
                let names_columns = columns_clone
                    .iter()
                    .map(|column: &Column| column.name.clone())
                    .collect();
                let rows_mapping: Vec<RowValue> = {
                    match self.constraints {
                        None => table.rows.clone(),
                        Some(constraints) => {
                        let mut rows: Vec<RowValue> = Vec::new();
                        for row in table.rows {
                            if compare_condition(constraints.clone(), &row, &names_columns) {
                                rows.push(row.clone());
                            }
                        }
                        rows
                        }
                    }
                };
                let mut columns_return: Vec<Column> = Vec::new();
                let mut rows_return: Vec<RowValue> = Vec::new();
                let columns_get = self.columns;
                for column_get in columns_get {
                    if column_get == "*" {
                        for column in &columns {
                            columns_return.push(column.clone());
                        }
                    } else {
                        let columns_clone = columns.clone();
                        if let Some(matched_column) =
                            columns_clone.iter().find(|&col| col.name == column_get)
                        {
                            columns_return.push(matched_column.clone());
                        }
                    }
                }
                for row_mapping in rows_mapping {
                    let mut row_return: Vec<SqlValue> = Vec::new();
                    for column_return in &columns_return {
                        let row_mapping_values = row_mapping.values.clone();
                        let columns_clone = columns.clone();
                        let uindex = columns_clone
                            .iter()
                            .position(|r_name| r_name.name == column_return.name)
                            .unwrap();
                        row_return.push(row_mapping_values[uindex].clone());
                    }
                    let rowvalue_return = RowValue { values: row_return };
                    rows_return.push(rowvalue_return)
                }
                let sqltable_return = SqlTable {
                    columns: columns_return,
                    rows: rows_return,
                };
                let box_sqltable_return: Box<SqlTable> = Box::new(sqltable_return);
                return Ok(ExecuteResponse::View(box_sqltable_return));
            }
            Err(err) => match err.kind() {
                std::io::ErrorKind::NotFound => Err(QueryExecutionError::TableNotFound(table_name)),
                _ => Err(QueryExecutionError::TableOpenfail(table_name)),
            },
        }
    }
}

impl Executable for UpdateStatement {
    // Replace Record
    fn check_and_execute(
        self,
        storage_util: StoreUtil,
    ) -> Result<ExecuteResponse, QueryExecutionError> {
        let table_name = self.table.clone();
        let wc = match self.constraints {
            Some(wc_tmp) => wc_tmp,
            None => {
                return Err(QueryExecutionError::NoConditionsObtained());
            }
        };
        let sets_new = self.sets;
        match storage_util.load(table_name.clone()) {
            //check the result of loading
            Ok(table) => {
                let rows_old: Vec<RowValue> = table.rows;
                let columns_old = table.columns;
                let mut names_old: Vec<String> = Vec::new();
                for column_old in &columns_old {
                    names_old.push(column_old.name.clone());
                }
                let mut rows_new: Vec<RowValue> = Vec::new();
                for row_old in rows_old {
                    if compare_condition(wc.clone(), &row_old, &names_old) {
                        let mut row_new: Vec<SqlValue> = Vec::new();
                        let mut row_old_value = row_old.values.clone();
                        for name_old in &names_old {
                            let mut flag = 1;
                            for set_new in &sets_new {
                                if name_old == &set_new.column {
                                    flag = 0;
                                    row_new.push(set_new.value.clone());
                                    row_old_value.drain(0..1);
                                    break;
                                }
                            }
                            if flag == 1 {
                                row_new.push(row_old_value[0].clone());
                                row_old_value.drain(0..1);
                            }
                        }
                        let row_add = RowValue { values: row_new };
                        rows_new.push(row_add.clone());
                    } else {
                        rows_new.push(row_old.clone());
                    }
                }
                let table_new = SqlTable {
                    columns: columns_old,
                    rows: rows_new,
                };
                match storage_util.save(table_name.clone(), &table_new) {
                    Ok(_) => Ok(ExecuteResponse::Message("update final".to_string())),
                    Err(_) => Err(QueryExecutionError::TableSavefail(table_name)),
                }
            }
            Err(err) => match err.kind() {
                std::io::ErrorKind::NotFound => Err(QueryExecutionError::TableNotFound(table_name)),
                _ => Err(QueryExecutionError::TableOpenfail(table_name)),
            },
        }
    }
}

impl Executable for SqlQuery {
    fn check_and_execute(
        self,
        storage_util: StoreUtil,
    ) -> Result<ExecuteResponse, QueryExecutionError> {
        match self {
            SqlQuery::Create(stmt) => stmt.check_and_execute(storage_util),
            SqlQuery::Drop(stmt) => stmt.check_and_execute(storage_util),
            SqlQuery::Insert(stmt) => stmt.check_and_execute(storage_util),
            SqlQuery::Delete(stmt) => stmt.check_and_execute(storage_util),
            SqlQuery::Update(stmt) => stmt.check_and_execute(storage_util),
            SqlQuery::Select(stmt) => stmt.check_and_execute(storage_util),
        }
    }
}

impl Into<String> for SqlValue {
    fn into(self) -> String {
        match self {
            SqlValue::String(s) => s,
            SqlValue::Int(i) => i.to_string(),
            _ => String::from("Unknow"),
        }
    }
}

impl Display for ExecuteResponse {
    // Pretty print select result
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecuteResponse::Message(s) => s.fmt(f),
            ExecuteResponse::Count(cnt) => cnt.fmt(f),
            ExecuteResponse::View(table) => {
                let mut builder = Builder::default();
                for row in table.rows.iter() {
                    builder.push_record(row.values.clone());
                }
                let header = table
                    .columns
                    .iter()
                    .map(|col| col.name.clone())
                    .collect::<Vec<String>>();
                builder.set_header(header);
                let mut table = builder.build();
                let style = Style::modern()
                    .remove_horizontals()
                    .remove_verticals()
                    .horizontals([HorizontalLine::new(1, Style::modern().get_horizontal())
                        .main(Some('═'))
                        .intersection(None)])
                    .verticals([VerticalLine::new(1, Style::modern().get_vertical())]);
                table.with(style);
                table.fmt(f)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_table() {
        let res = ExecuteResponse::View(Box::new(SqlTable {
            columns: vec![
                Column {
                    name: "id".into(),
                    type_info: SqlType::Int,
                },
                Column {
                    name: "des".into(),
                    type_info: SqlType::String,
                },
            ],
            rows: vec![
                RowValue {
                    values: vec![SqlValue::Int(1), SqlValue::String("aabbccdd".into())],
                },
                RowValue {
                    values: vec![SqlValue::Int(123), SqlValue::String("aabbcc".into())],
                },
                RowValue {
                    values: vec![
                        SqlValue::Int(11),
                        SqlValue::String("aabbccddaabbccdd".into()),
                    ],
                },
                RowValue {
                    values: vec![SqlValue::Int(2141), SqlValue::String("aabbccdd".into())],
                },
            ],
        }));
        println!("{}", res);
    }
}

#[cfg(test)]
mod tests_create {
    use super::*;
    #[test]
    fn test_check_and_execute_success() {
        let col1 = Column {
            name: "col1".to_string(),
            type_info: SqlType::Int,
        };
        let col2 = Column {
            name: "col2".to_string(),
            type_info: SqlType::String,
        };
        let col3 = Column {
            name: "col3".to_string(),
            type_info: SqlType::Unknown,
        };

        let create_statement = CreateStatement {
            table: "test_table_new".to_string(),
            columns: vec![col1, col2, col3],
        };

        let store_util = StoreUtil::Csv(r"E:\git_commits\rust_db".to_string());

        match create_statement.check_and_execute(store_util) {
            Ok(response) => {
                assert_eq!(
                    response,
                    ExecuteResponse::Message("save test_table_new successful".to_string())
                );
            }
            Err(_) => {
                panic!("Expected Ok but got Err");
            }
        }
    }
}

#[cfg(test)]
mod tests_drop {
    use super::*;
    #[test]
    fn test_check_and_execute_success() {
        let drop_statement = DropStatement {
            table: "test_table_drop".to_string(),
        };

        let store_util = StoreUtil::Csv(r"E:\git_commits\rust_db".to_string());

        match drop_statement.check_and_execute(store_util) {
            Ok(response) => {
                assert_eq!(
                    response,
                    ExecuteResponse::Message("delete test_table_drop successful".to_string())
                );
            }
            Err(_) => {
                panic!("Expected Ok but got Err");
            }
        }
    }
}

#[cfg(test)]
mod tests_insert {
    use super::*;
    #[test]
    fn test_check_and_execute_success() {
        let expected = InsertStatement {
            table: String::from("test_table"),
            columns: Some(vec![
                String::from("col1"),
                String::from("col2"),
                String::from("col3"),
            ]),
            values: RowValue {
                values: vec![
                    SqlValue::Int(123),
                    SqlValue::String(String::from("abc")),
                    SqlValue::Unknown,
                ],
            },
        };

        let store_util = StoreUtil::Csv(r"E:\git_commits\rust_db".to_string());

        match expected.check_and_execute(store_util) {
            Ok(response) => {
                assert_eq!(
                    response,
                    ExecuteResponse::Message("save test_table successful".to_string())
                );
            }
            Err(_) => {
                panic!("Expected Ok but got Err");
            }
        }
    }
}
