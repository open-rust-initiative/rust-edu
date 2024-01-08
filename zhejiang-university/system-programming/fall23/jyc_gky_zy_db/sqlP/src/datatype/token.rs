use std::fmt;
use super::{
    keyword::*,
    symbol::*,
    function::*
};

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Keyword(Keyword),
    Symbol(Symbol),
    Function(FunctionT),
    Identifier(String),
    Variable(String),
    Number(String),
    Bool(bool),
    Null,
}

pub trait SqlCharExt {
    fn is_symbol(&self) -> bool;
    fn as_symbol(&self) -> Option<Symbol>;
}

impl SqlCharExt for char {
    fn is_symbol(&self) -> bool {
        if to_symbol(&self.to_string().as_str()).is_some() {
            return true
        }
        false
    }
    fn as_symbol(&self) -> Option<Symbol> {
        if let Some(symbol) = to_symbol(&self.to_string().as_str()) {
            return Some(symbol)
        }
        None
    }
}

pub trait SqlStringExt {
    fn is_keyword(&self) -> bool;
    fn is_function(&self) -> bool;
    fn as_keyword(&self) -> Option<Keyword>;
    fn as_symbol(&self) -> Option<Symbol>;
    fn as_function(&self) -> Option<FunctionT>;
    fn as_bool(&self) -> Option<bool>;
}

impl SqlStringExt for String {
    fn is_keyword(&self) -> bool {
        if to_keyword(&self.as_str()).is_some() {
            return true
        }
        false
    }
    fn is_function(&self) -> bool {
        if to_function(&self.as_str()).is_some() {
            return true
        }
        false
    }
    fn as_keyword(&self) -> Option<Keyword> {
        if let Some(keyword) = to_keyword(&self) {
            return Some(keyword)
        }
        None
    }
    fn as_symbol(&self) -> Option<Symbol> {
        if let Some(symbol) = to_symbol(&self.as_str()) {
            return Some(symbol)
        }
        None
    }
    fn as_function(&self) -> Option<FunctionT> {
        if let Some(function) = to_function(&self.as_str()) {
            return Some(function)
        }
        None
    }
    fn as_bool(&self) -> Option<bool> {
        if self.to_uppercase() == "TRUE" {
            return Some(true)
        } else if self.to_uppercase() == "FALSE" {
            return Some(false)
        }
        None
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Keyword(keyword) => write!(f, "{}", keyword),
            Token::Symbol(symbol) => write!(f, "{}", symbol),
            Token::Function(function) => write!(f, "{}", function),
            Token::Identifier(identifier) => write!(f, "{}", identifier),
            Token::Variable(variable) => write!(f, "{}", variable),
            Token::Number(num) => write!(f, "{}", num),
            Token::Bool(bool) => {
                match bool {
                    true => write!(f, "TRUE"),
                    false => write!(f, "FALSE"),
                }
            }
            Token::Null => write!(f, "Null"),
        }
    }
}

impl Token {
    pub fn is_operator(&self) -> bool {
        match self {
            Token::Symbol(Symbol::Comma)
            | Token::Symbol(Symbol::Dot)
            | Token::Symbol(Symbol::Asterisk)
            | Token::Symbol(Symbol::Plus)
            | Token::Symbol(Symbol::Minus)
            | Token::Symbol(Symbol::Slash)
            | Token::Symbol(Symbol::Percent)
            | Token::Symbol(Symbol::LeftParen)
            | Token::Symbol(Symbol::RightParen) => true,
            _ => false,
        }
    }

    pub fn as_symbol(&self) -> Option<Symbol> {
        match self {
            Token::Symbol(s) => Some(s.clone()),
            _ => None,
        }
    }

    pub fn as_keyword(&self) -> Option<Keyword> {
        match self {
            Token::Keyword(k) => Some(k.clone()),
            _ => None,
        }
    }

    pub fn is_terminator(&self) -> bool {
        match self {
            Token::Symbol(Symbol::Semicolon)
            | Token::Symbol(Symbol::Slash) => true,
            _ => false
        }
    }
}