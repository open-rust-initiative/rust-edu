use crate::parser::condition::Condition;
use crate::parser::join::FromType;
use crate::system::errors::Errors;
use crate::system::utils::custom_strip;
use sqlparser::ast::Statement;
use std::collections::HashMap;

#[derive(Debug)]
pub struct UpdateQuery {
    pub tb_name: String,
    pub assignments: HashMap<String, String>,
    pub condition: Option<Condition>,
}

impl UpdateQuery {
    pub fn format_stat(statement: Statement) -> Result<UpdateQuery, Errors> {
        let mut tb_name: String = "".to_string();
        let mut assignments_data: HashMap<String, String> = HashMap::new();
        let mut condition_data: Option<Condition> = None;
        if let Statement::Update {
            table,
            assignments,
            selection,
            ..
        } = statement
        {
            let from = match FromType::new(vec![table]) {
                Ok(v) => v.first().unwrap().to_owned(),
                Err(err) => {
                    return Err(err);
                }
            };
            if let FromType::String { tb } = from {
                tb_name = tb;
            }
            assignments_data = assignments
                .iter()
                .map(|assign| {
                    (
                        assign.id.first().unwrap().to_owned().value.to_string(),
                        custom_strip(custom_strip(assign.value.to_string().as_str(), "\'"), "\"")
                            .to_string(),
                    )
                })
                .collect::<HashMap<String, String>>();
            let condition = Condition::from_expr(&selection.unwrap());
            match condition {
                Ok(v) => condition_data = Option::from(v),
                Err(e) => return Err(e),
            }
        }
        Ok(UpdateQuery {
            tb_name,
            assignments: assignments_data,
            condition: condition_data,
        })
    }
}

#[test]
pub fn test_update_query() {
    let sql = "UPDATE users SET password = 'new_password', email = \"new_email@example.com\" WHERE id = 1;";
    let state = parse_sql(sql);
    println!("{:?}", state);
}
