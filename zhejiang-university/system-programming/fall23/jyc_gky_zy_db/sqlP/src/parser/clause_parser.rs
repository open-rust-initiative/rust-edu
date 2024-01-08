use std::{
    vec::IntoIter,
    iter::Peekable,
};
use super::{
    error::{ParseError, Result},
    expression_parser::*,
    super::{
        models::structs::*,
        datatype::{
            token::*,
            keyword::Keyword,
            symbol::Symbol,
        },
    }
};

pub fn parse_where(iter: &mut Peekable<IntoIter<Token>>) -> Result<Option<Condition>> {
    match iter.peek() {
        Some(Token::Keyword(Keyword::Where)) => iter.next(),
        _  => return Ok(None),
    };
    
    let condition = parse_condition(iter)?;
    return Ok(Some(condition))
}

pub fn parse_having(iter: &mut Peekable<IntoIter<Token>>) -> Result<Option<Condition>> {
    match iter.peek() {
        Some(Token::Keyword(Keyword::Having)) => iter.next(),
        _  => return Ok(None),
    };

    let condition = parse_condition(iter)?;
    return Ok(Some(condition))
}

pub fn parse_projection(iter: &mut Peekable<IntoIter<Token>>) -> Result<Column> {
    if let Some(Token::Symbol(Symbol::Asterisk)) = iter.peek() {
        iter.next();
        return Ok(Column::AllColumns);
    }

    parse_columns(iter)
}

pub fn parse_groupby(iter: &mut Peekable<IntoIter<Token>>) -> Result<Column> {
    match iter.peek() {
        Some(Token::Keyword(Keyword::GroupBy)) => iter.next(),
        _  => return Ok(Column::AllColumns),
    };

    parse_columns(iter)
}

pub fn parse_orderby(
    iter: &mut Peekable<IntoIter<Token>>
) -> Result<Option<Vec<(String, Sort)>>> {
    match iter.peek() {
        Some(Token::Keyword(Keyword::OrderBy)) => iter.next(),
        _  => return Ok(None),
    };

    let mut order_by: Vec<(String, Sort)> = Vec::new();

    loop {
        match iter.peek() {
            Some(Token::Identifier(name)) => {
                let current_name = name.clone();
                iter.next();

                match iter.next() {
                    Some(t) => {
                        let sort = match t {
                            Token::Keyword(Keyword::Asc) => Sort::ASC,
                            | Token::Keyword(Keyword::Desc) => Sort::DESC,
                            _ => return Err(ParseError::UnexpectedToken(t.clone())),
                        };
                        let tuple = (current_name, sort);
                        order_by.push(tuple);
                    },
                    None => return Err(ParseError::MissingSort)
                }
            },
            Some(Token::Symbol(Symbol::Comma)) => {
                iter.next();
                continue;
            },
            Some(token) if token.is_terminator() => break,
            Some(token) => return Err(ParseError::UnexpectedToken(token.clone())),
            None => return Err(ParseError::MissingColumn)
        }
    }
    return Ok(Some(order_by));
}

pub fn parse_tables(
    iter: &mut Peekable<IntoIter<Token>>
) -> Result<Vec<(Expression, Option<Expression>)>> {
    match iter.peek() {
        Some(Token::Keyword(Keyword::From)) => (),
        _  => return Err(ParseError::MissingToken(Token::Keyword(Keyword::From))),
    }
    iter.next();

    let tables = parse_items_with_alias(iter)?;

    if tables.len() == 0 {
        return Err(ParseError::MissingTable);
    }
    Ok(tables)
}

fn parse_columns(iter: &mut Peekable<IntoIter<Token>>) -> Result<Column> {
    Ok(Column::Columns(parse_items_with_alias(iter)?))
}

fn parse_items_with_alias(
    iter: &mut Peekable<IntoIter<Token>>
) -> Result<Vec<(Expression, Option<Expression>)>> 
{
    let mut columns = Vec::new();

    loop {
        match iter.peek() {
            Some(Token::Keyword(k)) if k.is_clause() => break,
            Some(s) if s.is_terminator() => break,
            _ => ()
        }
        match parse_expression(iter) {
            Ok(e) => {
                let mut alias = None;
                if let Some(Token::Keyword(Keyword::As)) = iter.peek() {
                    iter.next();
                    alias = Some(parse_expression(iter)?);
                }
                columns.push((e, alias));
                if let Some(Token::Symbol(Symbol::Comma)) = iter.peek() {
                    iter.next();
                }
            },
            Err(e) => return Err(e),
        }
    }

    return Ok(columns);
}

fn parse_condition(iter: &mut Peekable<IntoIter<Token>>) -> Result<Condition> {
    let mut left: Option<Condition> = None;

    while let Some(token) = iter.peek() {
        match token {
            Token::Keyword(Keyword::And)
            | Token::Keyword(Keyword::Or)
            | Token::Keyword(Keyword::Not) => {
                let current_token = token.clone();
                iter.next();
                let next_condition = parse_condition(iter)?;
                left = match current_token {
                    Token::Keyword(Keyword::And) => {
                        Some(Condition::And {
                            left: Box::new(left.take().unwrap().clone()),
                            right: Box::new(next_condition)
                        })
                    },
                    Token::Keyword(Keyword::Or) => {
                        Some(Condition::Or {
                            left: Box::new(left.take().unwrap().clone()),
                            right: Box::new(next_condition)
                        })
                    },
                    Token::Keyword(Keyword::Not) => Some(Condition::Not(Box::new(next_condition))),
                    _ => return Err(ParseError::UnknownError),
                };
            },
            Token::Symbol(Symbol::LeftParen) => {
                iter.next();
                let next_condition = parse_condition(iter)?;
                if let Some(Token::Symbol(Symbol::RightParen)) = iter.next() {
                    left = Some(next_condition);
                } else {
                    return Err(ParseError::MissingToken(Token::Symbol(Symbol::RightParen)));
                }
            },
            token if token.is_terminator() => break,
            Token::Symbol(Symbol::RightParen) | Token::Keyword(_) => break,
            Token::Symbol(_) | Token::Number(_) => {
                return Err(ParseError::UnexpectedToken(token.clone()));
            }
            Token::Identifier(_) | Token::Variable(_) | Token::Function(_) | Token::Bool(_) => {
                left = Some(parse_comparison(iter)?);
            }
            t => return Err(ParseError::UnexpectedToken(t.clone())),
        }
    }

    if let Some(r) = left {
        return Ok(r);
    }
    return Err(ParseError::IncorrectCondition);
}

fn parse_comparison(iter: &mut Peekable<IntoIter<Token>>) -> Result<Condition> {
    let left = match iter.peek() {
        Some(Token::Identifier(_))
        | Some(Token::Symbol(Symbol::LeftParen))
        | Some(Token::Variable(_)) 
        | Some(Token::Function(_)) => parse_expression(iter)?,
        Some(t) => return Err(ParseError::UnexpectedToken(t.clone())),
        None => return Err(ParseError::MissingComparator),
    };

    let operator = match iter.peek() {
        Some(Token::Symbol(t)) if t.is_comparator() => t.clone(),
        Some(t) => return Err(ParseError::UnexpectedToken(t.clone())),
        None => return Err(ParseError::MissingComparator),
    };
    iter.next();

    let right = parse_expression(iter)?;

    Ok(
        Condition::Comparison {
            left,
            operator,
            right,
        }
    )
}
