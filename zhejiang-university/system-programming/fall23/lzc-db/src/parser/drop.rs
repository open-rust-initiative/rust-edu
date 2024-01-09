use crate::system::errors::Errors;
use sqlparser::ast::Statement;

pub struct DropQuery {
    pub drop_tbs: Vec<String>,
}

impl DropQuery {
    pub fn format_stat(state: Statement) -> Result<DropQuery, Errors> {
        if let Statement::Drop { names, .. } = state {
            let drop_tbs = names.iter().map(|x| x.to_string()).collect::<Vec<String>>();
            return Ok(DropQuery { drop_tbs });
        } else {
            Err(Errors::InvalidExpression)
        }
    }
}

#[test]
fn test_drop_query() {
    let sql = "DROP TABLE articles;";
    let state = parse_sql(sql);
    println!("{:?}", state);
}
