use std::{
    vec::IntoIter,
    iter::Peekable
};
use super::{
    datatype::{
        keyword::Keyword,
        token::Token
    },
    models::structs::Statement,
    parser::{
        statement_parser::parse_select
    },
    error::Result,
    lexer::lex,
};

pub struct Parser {
    iter: Peekable<IntoIter<Token>>
}

impl Parser {
    pub fn new() -> Self {
        Self {
            iter: Vec::new().into_iter().peekable()
        }
    }
    pub fn parse(&mut self, s: &str) -> Result<Statement> {
        let tokens = lex(s);
        self.iter = tokens.clone().into_iter().peekable();
        match tokens.get(0) {
            Some(Token::Keyword(Keyword::Select)) => return Ok(parse_select(&mut self.iter)?),
            _ => (),
        };
        todo!()
    }
}