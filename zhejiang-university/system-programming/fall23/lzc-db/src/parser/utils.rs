use crate::system::errors::Errors;
use sqlparser::ast::Statement;
use sqlparser::dialect::AnsiDialect;
use sqlparser::parser::{Parser};

pub fn parse_sql(sql: &str) -> Result<Statement, Errors> {
    let dialect = AnsiDialect {};
    let binding = match Parser::parse_sql(&dialect, &sql) {
        Ok(v) => v,
        Err(_) => return Err(Errors::ParseSQLError),
    };
    let statement: &Statement = binding.first().unwrap();
    Ok(statement.to_owned())
}
