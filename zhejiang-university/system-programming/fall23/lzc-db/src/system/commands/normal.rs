use crate::database;
use crate::database::table::{PrettyTable, Table};
use crate::parser::create::CreateQuery;
use crate::parser::delete::DeleteQuery;
use crate::parser::drop::DropQuery;
use crate::parser::insert::InsertQuery;
use crate::parser::join::FromType;
use crate::parser::select::SelectQuery;
use crate::parser::update::UpdateQuery;
use crate::parser::utils::parse_sql;
use crate::system::errors::Errors;
use std::collections::HashMap;

pub fn create_tb(query: String, db: &mut database::db::Database) {
    let state = match parse_sql(query.as_str()) {
        Ok(v) => v,
        Err(err) => {
            err.print();
            return;
        }
    };
    let query = match CreateQuery::format_stat(state) {
        Ok(v) => v,
        Err(err) => {
            err.print();
            return;
        }
    };
    let tb = Table::new(query);
    match db.create_table(tb) {
        Ok(_) => {}
        Err(err) => {
            err.print();
            return;
        }
    };
    db.save_disk().unwrap();
}

pub fn drop_tb(query: String, db: &mut database::db::Database) {
    let state = match parse_sql(query.as_str()) {
        Ok(v) => v,
        Err(err) => {
            err.print();
            return;
        }
    };
    let query_result = DropQuery::format_stat(state);
    let query = match query_result {
        Ok(v) => v,
        Err(err) => {
            err.print();
            return;
        }
    };
    db.drop_table(query.drop_tbs);
    db.save_disk().unwrap()
}

pub fn insert_data(query: String, db: &mut database::db::Database) {
    let state = match parse_sql(query.as_str()) {
        Ok(v) => v,
        Err(err) => {
            err.print();
            return;
        }
    };
    let query_result = InsertQuery::format_stat(state);
    let query = match query_result {
        Ok(v) => v,
        Err(err) => {
            err.print();
            return;
        }
    };
    db.insert_row(query.tb_name, query.cols, query.rows);
    db.save_disk().unwrap();
}

pub fn select_data(query: String, db: &mut database::db::Database) {
    let state = match parse_sql(query.as_str()) {
        Ok(v) => v,
        Err(err) => {
            err.print();
            return;
        }
    };
    let query = match SelectQuery::format_stat(state) {
        Ok(v) => v,
        Err(err) => {
            err.print();
            return;
        }
    };
    let froms = query.from.clone();
    if froms.len() != 1 {
        println!("{}", Errors::InvalidExpression);
        return;
    }
    let from = match froms.first() {
        None => {
            println!("{}", Errors::InvalidExpression);
            return;
        }
        Some(v) => v.to_owned(),
    };
    match from {
        FromType::Join { join_info, .. } => {
            let left_table = join_info.clone().left_table;
            let right_table = join_info.clone().right_table;
            let left_tb = match db.get_table(left_table) {
                Ok(v) => v,
                Err(err) => {
                    err.print();
                    return;
                }
            };
            let right_tb = match db.get_table(right_table) {
                Ok(v) => v,
                Err(err) => {
                    err.print();
                    return;
                }
            };
            let joint_tb = Table::join_tbs(left_tb, right_tb, join_info);
            joint_tb.select_data(query)
        }
        FromType::String { tb } => {
            let tb = match db.get_table(tb) {
                Ok(v) => v,
                Err(err) => {
                    err.print();
                    return;
                }
            };
            tb.select_data(query)
        }
    }
}

pub fn update_data(query: String, db: &mut database::db::Database) {
    let state = match parse_sql(query.as_str()) {
        Ok(v) => v,
        Err(err) => {
            err.print();
            return;
        }
    };
    let query = match UpdateQuery::format_stat(state) {
        Ok(v) => v,
        Err(err) => {
            err.print();
            return;
        }
    };
    let tb: &mut Table = match db.get_table_mut(query.tb_name) {
        Ok(v) => v,
        Err(err) => {
            err.print();
            return;
        }
    };
    let rows = match tb.get_rows() {
        Ok(v) => v,
        Err(err) => {
            err.print();
            return;
        }
    };
    let filtered_rows = tb.filter_rows(&query.condition, rows.clone(), None);
    let pk = tb
        .columns
        .iter()
        .filter(|col| col.is_pk)
        .map(|col| col.name.clone())
        .collect::<Vec<String>>()
        .first()
        .unwrap()
        .to_string();
    let row_ids = filtered_rows
        .iter()
        .map(|row| row.get(pk.as_str()).unwrap().to_string())
        .collect::<Vec<String>>();
    let row_ixs: Vec<usize> = rows
        .iter()
        .enumerate()
        .filter_map(|(ix, row)| {
            if row_ids.contains(row.get(pk.as_str()).unwrap()) {
                Some(ix)
            } else {
                None
            }
        })
        .collect();
    for (col, val) in &query.assignments {
        for &row_ix in &row_ixs {
            if let Some(column_data) = tb.col_map.get_mut(col.as_str()) {
                column_data.update_val(row_ix, val.clone());
            }
        }
    }
    db.save_disk().unwrap()
}

pub fn delete_data(query: String, db: &mut database::db::Database) {
    let state = match parse_sql(query.as_str()) {
        Ok(v) => v,
        Err(err) => {
            err.print();
            return;
        }
    };
    let query_result = DeleteQuery::format_stat(state);
    let query;
    match query_result {
        Ok(v) => query = v,
        Err(err) => {
            err.print();
            return;
        }
    }
    let tb: &mut Table = match db.get_table_mut(query.tb_name) {
        Ok(v) => v,
        Err(err) => {
            err.print();
            return;
        }
    };
    let rows = match tb.get_rows() {
        Ok(v) => v,
        Err(err) => {
            err.print();
            return;
        }
    };
    let filtered_rows = tb.filter_rows(&query.condition, rows.clone(), None);
    let pk = tb
        .columns
        .iter()
        .filter(|col| col.is_pk)
        .map(|col| col.name.clone())
        .collect::<Vec<String>>()
        .first()
        .unwrap()
        .to_string();
    let row_ids = filtered_rows
        .iter()
        .map(|row| row.get(pk.as_str()).unwrap().to_string())
        .collect::<Vec<String>>();
    let row_ixs: Vec<usize> = rows
        .iter()
        .enumerate()
        .filter_map(|(ix, row)| {
            if row_ids.contains(row.get(pk.as_str()).unwrap()) {
                Some(ix)
            } else {
                None
            }
        })
        .collect();
    for (_, val) in &mut tb.col_map {
        val.delete_val(row_ixs.clone());
    }
    println!("Number of affected rows: {}", row_ixs.len());
    db.save_disk().unwrap()
}

pub fn show_tb_data(query: String, db: &mut database::db::Database) {
    let vars = query.split(" ").collect::<Vec<&str>>();
    assert_eq!(vars.len(), 2);
    let tb_name = vars[1].to_string();
    let tb = match db.get_table(tb_name) {
        Ok(v) => v,
        Err(err) => {
            err.print();
            return;
        }
    };
    tb.print_table_data();
}

pub fn show_tb_info(query: String, db: &mut database::db::Database) {
    let vars = query.split(" ").collect::<Vec<&str>>();
    assert_eq!(vars.len(), 2);
    let tb_name = vars[1].to_string();
    let tb = match db.get_table(tb_name) {
        Ok(v) => v,
        Err(err) => {
            err.print();
            return;
        }
    };
    tb.show_info();
}

pub fn show_all_tbs(db: &mut database::db::Database) {
    let tb_names = db
        .tables
        .iter()
        .map(|x| {
            let mut table_map = HashMap::new();
            table_map.insert("Table Name".to_string(), x.name.clone());
            table_map
        })
        .collect::<Vec<HashMap<String, String>>>();
    let pt = PrettyTable::create(
        db.db_name.to_string(),
        vec!["Table Name".to_string()],
        tb_names,
    );
    println!("{pt}")
}
