use super::executor::types::*;
use super::sql_analyzer::types::Column;
use super::sql_analyzer::types::RowValue;
use super::sql_analyzer::types::SqlType;
use super::sql_analyzer::types::SqlValue;
use csv::Writer;
use csv::{ReaderBuilder, StringRecord};
use std::fs;
use std::fs::File;
use std::io;

pub enum StoreUtil {
    /// The persistent data table is in csv format
    Csv(String),
    /// The persistent data table is in json format
    Json(String),
}

//Consider constructing a table with the first row as the column name, the second row recording the data format(string, int),
//and the third row starting to record the data content

impl StoreUtil {
    // return file's path
    pub fn get_path(&self, name: &String) -> String {
        let path_root;
        match self {
            StoreUtil::Csv(path) => {
                path_root = path;
            }
            StoreUtil::Json(path) => {
                path_root = path;
            }
        };
        let path = path_root.to_owned() + "/" + &name;
        path
    }
    /// check whether the table exists
    pub fn exists(&self, name: &String) -> bool {
        let path = self.get_path(name);
        fs::metadata(path).is_ok()
    }

    pub fn delete(&self, name: &String) -> Result<(), io::Error> {
        if self.exists(&name) {
            let path = self.get_path(&name);
            fs::remove_file(path.clone())?;
            Ok(())
        } else {
            return Err(io::Error::new(io::ErrorKind::NotFound, "Table not found"));
        }
    }

    /// load table with table name
    pub fn load(&self, name: String) -> Result<SqlTable, io::Error> {
        if !self.exists(&name) {
            return Err(io::Error::new(io::ErrorKind::NotFound, "Table not found"));
        } else {
            let path = self.get_path(&name);
            let file = File::open(path)?;
            let mut csv_reader = ReaderBuilder::new().has_headers(true).from_reader(file);
            let head: &StringRecord = csv::Reader::headers(&mut csv_reader)?;
            let columns_name: Vec<String> = head.iter().map(|col| col.to_string()).collect();
            let records: Vec<StringRecord> =
                csv::Reader::records(&mut csv_reader).collect::<Result<_, _>>()?;
            let columns_type: Vec<String> = records[0].iter().map(|col| col.to_string()).collect();
            let columns: Vec<Column> = columns_name
                .iter()
                .zip(columns_type.iter())
                .map(|(name, type_name)| Column {
                    name: name.to_string(),
                    type_info: match type_name.as_str() {
                        "String" => SqlType::String,
                        "Int" => SqlType::Int,
                        _ => SqlType::Unknown,
                    },
                })
                .collect();
            let rows: Vec<RowValue> = records[1..]
                .iter()
                .map(|record| {
                    let values: Vec<SqlValue> = record
                        .iter()
                        .zip(columns.iter())
                        .map(|(field, column)| match column.type_info {
                            SqlType::String => SqlValue::String(field.to_string()),
                            SqlType::Int => field
                                .to_string()
                                .parse::<i32>()
                                .map_or(SqlValue::Unknown, SqlValue::Int),
                            _ => SqlValue::Unknown,
                        })
                        .collect();
                    RowValue { values }
                })
                .collect();

            Ok(SqlTable { columns, rows })
        }
    }
    /// save table persistently
    pub fn save(&self, name: String, table: &SqlTable) -> Result<(), io::Error> {
        let path = self.get_path(&name);
        if self.exists(&name) {
            fs::remove_file(path.clone())?
        }
        //get infos from table
        let columns = &table.columns;
        let columns_name: Vec<String> = columns.iter().map(|column| column.name.clone()).collect();
        let columns_type: Vec<String> = columns
            .iter()
            .map(|column| match column.type_info {
                SqlType::String => String::from("String"),
                SqlType::Int => String::from("Int"),
                _ => String::from("Unknown"),
            })
            .collect();

        //write
        //Temporarily not considering resource consumption for multiple complete storage
        let mut writer_csv = Writer::from_path(path)?;
        writer_csv.write_record(columns_name)?;
        writer_csv.write_record(columns_type)?;
        let original_matrix = &table.rows;
        for row in original_matrix {
            let row_values: Vec<String> = row
                .clone()
                .values
                .iter()
                .map(|sql_value| match sql_value {
                    SqlValue::String(s) => s.clone(),
                    SqlValue::Int(i) => i.to_string(),
                    SqlValue::Unknown => String::from("Unknown"),
                })
                .collect();
            writer_csv.write_record(row_values)?;
        }
        writer_csv.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_save_cvs() {
        // Create a test SqlTable
        let test_table = SqlTable {
            columns: vec![
                Column {
                    name: String::from("Name"),
                    type_info: SqlType::String,
                },
                Column {
                    name: String::from("Age"),
                    type_info: SqlType::Int,
                },
            ],
            rows: vec![
                RowValue {
                    values: vec![SqlValue::String(String::from("John")), SqlValue::Int(25)],
                },
                RowValue {
                    values: vec![SqlValue::String(String::from("Alice")), SqlValue::Int(30)],
                },
            ],
        };

        // Create a test Storage object
        let storage = StoreUtil::Csv(String::from(r"E:\git_commits\rust_db"));

        // Call the save function with the test data
        let result = storage.save("test_table.csv".into(), &test_table);

        // Assert that the save function returned Ok
        assert!(result.is_ok());
    }
}
