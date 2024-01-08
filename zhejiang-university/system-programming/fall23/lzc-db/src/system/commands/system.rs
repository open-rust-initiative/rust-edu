use crate::database::db::Database;
use crate::database::table::PrettyTable;
use crate::system::errors::Errors;
use ansi_term::Color;
use std::collections::HashMap;
use std::fs;
use crate::system::tips;

fn database_exists(db_name: &str) -> bool {
    let file_path = format!("sql_files/{}.bin", db_name);
    fs::metadata(file_path).is_ok()
}

pub fn create_db(command: &str) -> Result<(), Errors> {
    let vars: Vec<&str> = command.split_whitespace().collect();
    if vars.len() != 3 {
        return Err(Errors::InvalidExpression);
    }

    let db_name = vars[2];
    let mut db = Database::new();
    db.set_dbname(db_name.to_string());
    Ok(db.save_disk().map_err(|_| Errors::DiskSaveError)?)
}

pub fn use_db(command: String, db: &mut Database) -> Result<(), Errors> {
    let vars = command.split(" ").collect::<Vec<&str>>();
    assert_eq!(vars.len(), 3);
    let db_name = vars[2];
    let file_path = format!("sql_files/{}.bin", db_name);
    if !database_exists(db_name) {
        return Err(Errors::DatabaseNotExisted);
    }
    Ok(db
        .load_from_disk(file_path.as_str())
        .map_err(|_| Errors::FileSystemError)?)
}

pub fn drop_db(command: String) -> Result<(), Errors> {
    let vars = command.split(" ").collect::<Vec<&str>>();
    assert_eq!(vars.len(), 3);
    let db_name = vars[2];
    let file_path = format!("sql_files/{}.bin", db_name);
    Ok(fs::remove_file(file_path).map_err(|_| Errors::DatabaseNotExisted)?)
}

pub fn show_databases() -> std::io::Result<()> {
    let files = fs::read_dir("sql_files")?;

    let mut databases = Vec::new();

    for file in files {
        if let Ok(entry) = file {
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.ends_with(".bin") {
                    let db_name = file_name.trim_end_matches(".bin").to_string();
                    databases.push(db_name);
                }
            }
        }
    }
    let db_header = vec!["Database".to_string()];
    let mut col_values: HashMap<String, Vec<String>> = HashMap::new();
    col_values.insert("Database".to_string(), databases);
    let pt = PrettyTable::new("All Databases".to_string(), db_header, col_values);
    println!("{}", pt);
    Ok(())
}


pub fn help(command: String) {
    let vars = command.split(" ").collect::<Vec<&str>>();
    if vars.len() == 2 {
        println!("{}", Color::Yellow.bold().paint("Table Command"));
        println!("{}", tips::create_table(false));
        println!("{}", tips::drop_table(false));
        println!("{}", tips::select_data(false));
        println!("{}", tips::insert_data(false));
        println!("{}", tips::delete_data(false));
        println!("{}", tips::update_data(false));
        println!("{}", Color::Yellow.bold().paint("System Command"));
        println!("{}", tips::create_db());
        println!("{}", tips::use_db());
        println!("{}", tips::drop_db());
        println!("{}", tips::show_dbs());
        println!("{}", tips::change_pwd());
    } else if vars.len() == 3 {
        if vars[2].eq("create") { println!("{}", tips::create_table(true)) }
        if vars[2].eq("drop") { println!("{}", tips::drop_table(true)) }
        if vars[2].eq("select") { println!("{}", tips::select_data(true)) }
        if vars[2].eq("insert") { println!("{}", tips::insert_data(true)) }
        if vars[2].eq("delete") { println!("{}", tips::delete_data(true)) }
        if vars[2].eq("update") { println!("{}", tips::update_data(true)) }
    }
}
