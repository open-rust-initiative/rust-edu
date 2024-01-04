use crate::parser::condition::Condition;
use crate::parser::join::FromType;
use crate::system::errors::Errors;
use sqlparser::ast::{SetExpr, Statement};
use std::option::Option;

#[derive(Debug, PartialEq)]
pub enum BinaryOpCus {
    Lt,
    Gt,
    Eq,
    And,
    Or,
    IsNull,
    Like,
}

#[derive(Debug)]
pub struct SelectQuery {
    pub from: Vec<FromType>,
    pub projection: Vec<String>,
    pub condition: Option<Condition>,
}

impl SelectQuery {
    pub fn format_stat(statement: Statement) -> Result<SelectQuery, Errors> {
        let mut select_from: Vec<FromType> = vec![];
        let mut select_projections: Vec<String> = vec![];
        let mut select_condition: Option<Condition> = None;
        match statement {
            Statement::Query(bd) => match &*bd.body {
                SetExpr::Select(select) => {
                    let projects = &select.projection;
                    let froms = &select.from;
                    let exprs = &select.selection;
                    if !exprs.is_none() {
                        let condition = Condition::from_expr(&exprs.clone().unwrap());
                        match condition {
                            Ok(v) => select_condition = Some(v),
                            Err(e) => {
                                return Err(e);
                            }
                        }
                    }
                    select_from = match FromType::new(froms.to_owned()) {
                        Ok(v) => v,
                        Err(err) => return Err(err),
                    };
                    for projection in projects {
                        let cname = projection.to_string();
                        select_projections.push(cname);
                    }
                }
                _ => {
                    return Err(Errors::InvalidExpression);
                }
            },
            _ => {}
        }
        Ok(SelectQuery {
            from: select_from,
            projection: select_projections,
            condition: select_condition,
        })
    }
}

#[test]
pub fn test_select() {
    let sql = "SELECT articles.id, articles.title, articles.userid, users.username FROM articles JOIN users ON articles.userid = users.id;";
    let stat = parse_sql(sql);
    let _query = SelectQuery::format_stat(stat.unwrap());

    let sql2 = "SELECT id,username from users;";
    let stat2 = parse_sql(sql2);
    let _query2 = SelectQuery::format_stat(stat2.unwrap());
}
