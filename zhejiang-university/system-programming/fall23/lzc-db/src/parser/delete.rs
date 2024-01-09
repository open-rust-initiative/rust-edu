use crate::parser::condition::Condition;
use crate::parser::join::FromType;
use crate::system::errors::Errors;
use sqlparser::ast::Statement;

#[derive(Debug)]
pub struct DeleteQuery {
    pub tb_name: String,
    pub condition: Option<Condition>,
}

impl DeleteQuery {
    pub fn format_stat(state: Statement) -> Result<DeleteQuery, Errors> {
        let mut tb_name: String = "".to_string();
        let mut condition_data: Option<Condition> = None;
        if let Statement::Delete {
            from, selection, ..
        } = state
        {
            let from = match FromType::new(from) {
                Ok(v) => v.first().unwrap().to_owned(),
                Err(err) => {
                    return Err(err);
                }
            };
            if let FromType::String { tb } = from {
                tb_name = tb.to_string();
            }
            let condition = Condition::from_expr(&selection.unwrap());
            match condition {
                Ok(v) => condition_data = Option::from(v),
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Ok(DeleteQuery {
            tb_name,
            condition: condition_data,
        })
    }
}

#[test]
pub fn test_delete() {
    let sql = "DELETE FROM users where id=1";
    let state = parse_sql(sql);
    println!("{:?}", state);
}
