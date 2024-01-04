use crate::database::table::Table;
use crate::system::errors::Errors;
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::File;
use std::io::{self, Write};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Database {
    pub db_name: String,
    pub tables: Vec<Table>,
}

impl Database {
    pub fn new() -> Database {
        return Database {
            db_name: "".to_string(),
            tables: vec![],
        };
    }

    pub fn set_dbname(&mut self, db_name: String) {
        self.db_name = db_name;
    }

    pub fn create_table(&mut self, tb: Table) -> Result<(), Errors> {
        if self.check_table(tb.name.clone()) {
            return Err(Errors::TableExisted(tb.name));
        }
        self.tables.push(tb);
        Ok(())
    }

    pub fn drop_table(&mut self, drop_tbs: Vec<String>) {
        self.tables.retain(|table| !drop_tbs.contains(&table.name));
    }

    pub fn check_table(&self, tb_name: String) -> bool {
        self.tables.iter().any(|v| v.name == tb_name)
    }

    pub fn get_table(&self, tb_name: String) -> Result<&Table, Errors> {
        for tb in &self.tables {
            if tb.name == tb_name {
                return Ok(tb);
            }
        }
        Err(Errors::TableNotExisted(tb_name))
    }
    pub fn get_table_mut(&mut self, tb_name: String) -> Result<&mut Table, Errors> {
        for tb in &mut self.tables {
            if tb.name == tb_name {
                return Ok(tb);
            }
        }
        Err(Errors::TableNotExisted(tb_name))
    }

    pub fn insert_row(&mut self, tb_name: String, cols: Vec<String>, rows: Vec<Vec<String>>) {
        let tb: &mut Table = match self.get_table_mut(tb_name.clone()) {
            Ok(v) => v,
            Err(err) => {
                err.print();
                return;
            }
        };
        tb.insert_row(cols, rows);
    }
    pub fn save_disk(&self) -> io::Result<()> {
        let serialized_data = serde_json::to_string(&self)?;
        let mut file =
            File::create("sql_files/".to_owned() + self.db_name.to_string().as_str() + ".bin")?;
        file.write_all(serialized_data.as_bytes())?;
        Ok(())
    }
    pub fn load_from_disk(&mut self, filename: &str) -> io::Result<()> {
        let file = File::open(filename)?;
        let database: Database = serde_json::from_reader(file)?;
        *self = database;
        Ok(())
    }
}
