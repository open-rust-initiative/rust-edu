use std::io::{stdin, stdout, Write};
use system::commands::parser::process_command;

mod database;
mod parser;
mod system;

use crate::system::dbs::DbSystem;

fn main() {
    DbSystem::init_cfg();
    let sys: DbSystem = DbSystem::new();
    let mut command = String::new();
    loop {
        print!("login: ");
        stdout().flush().unwrap();
        stdin()
            .read_line(&mut command)
            .expect("Error while trying to read from stdin");
        let vars = command.trim().split(" ").collect::<Vec<&str>>();
        let username = vars[0].to_string();
        let password = vars[1].to_string();
        let is_login = sys.login(username.clone(), password.clone());
        if !is_login {
            println!("Invalid username or password.")
        } else {
            println!("Welcome back {}.", username);
            break;
        }
        command.clear()
    }
    command.clear();
    let mut db = database::db::Database::new();
    loop {
        if db.db_name.is_empty() {
            print!("simple-db> ");
        } else {
            print!("simple-db[{}]> ", db.db_name);
        }
        stdout().flush().unwrap();
        stdin()
            .read_line(&mut command)
            .expect("Error while trying to read from stdin");
        process_command(command.trim().to_string(), &mut db);
        command.clear();
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::utils::parse_sql;

    #[test]
    fn test_create_table_new() {
        let sql = "CREATE TABLE employees (
        id INT PRIMARY KEY,
        name VARCHAR(100) NOT NULL DEFAULT Tom,
        role VARCHAR(100),
        department_id INT DEFAULT 0,
        abcd_id INT DEFAULT 0,
        abcd_x INT DEFAULT 0,
        email VARCHAR(100) UNIQUE,
        FOREIGN KEY (department_id) REFERENCES departments(id),
        FOREIGN KEY (abcd_id) REFERENCES abcds(id),
        FOREIGN KEY (abcd_x) REFERENCES abcds(x)
    );";
        let state = parse_sql(sql);
    }
}
