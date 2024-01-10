use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum Keyword {
    Select,
    Insert,
    Update,
    Delete,
    From,
    Where,
    GroupBy,
    OrderBy,
    Join,
    Into,
    InnerJoin,
    LeftJoin,
    RightJoin,
    FullJoin,
    Values,
    On,
    As,
    Distinct,
    All,
    Exists,
    Having,
    Union,
    Not,
    And,
    Or,
    Asc,
    Desc,
}

pub fn to_keyword(s: &str) -> Option<Keyword> {
    let string = s.to_uppercase();
    let mut iter = string.split_whitespace();
    let first = iter.next()?;

    match first {
        "SELECT" => Some(Keyword::Select),
        "INSERT" => Some(Keyword::Insert),
        "UPDATE" => Some(Keyword::Update),
        "DELETE" => Some(Keyword::Delete),
        "FROM" => Some(Keyword::From),
        "WHERE" => Some(Keyword::Where),
        "GROUP" => {
            if let Some(next) = iter.next() {
                if next == "BY" {
                    return Some(Keyword::GroupBy);
                }
            }
            None
        }
        "ORDER" => {
            if let Some(next) = iter.next() {
                if next == "BY" {
                    return Some(Keyword::OrderBy);
                }
            }
            None
        }
        "JOIN" => Some(Keyword::Join),
        "INTO" => Some(Keyword::Into),
        "INNER" => {
            if iter.next() == Some("JOIN") {
                return Some(Keyword::InnerJoin);
            }
            None
        }
        "LEFT" => {
            if iter.next() == Some("JOIN") {
                return Some(Keyword::LeftJoin);
            } else if iter.next() == Some("OUTER") && iter.next() == Some("JOIN") {
                return Some(Keyword::LeftJoin);
            }
            None
        }
        "RIGHT" => {
            if iter.next() == Some("JOIN") {
                return  Some(Keyword::RightJoin);
            }
            None
        }
        "FULL" => {
            if iter.next() == Some("JOIN") {
                return Some(Keyword::FullJoin);
            }
            None
        }
        "VALUES" => Some(Keyword::Values),
        "ON" => Some(Keyword::On),
        "AS" => Some(Keyword::As),
        "DISTINCT" => Some(Keyword::Distinct),
        "ALL" => Some(Keyword::All),
        "EXISTS" => Some(Keyword::Exists),
        "HAVING" => Some(Keyword::Having),
        "UNION" => Some(Keyword::Union),
        "NOT" => Some(Keyword::Not),
        "AND" => Some(Keyword::And),
        "OR" => Some(Keyword::Or),
        "ASC" => Some(Keyword::Asc),
        "DESC" => Some(Keyword::Desc),
        _ => None,
    }
}

impl Keyword {
    pub fn is_clause(&self) -> bool {
        match self {
            Self::From
            | Self::Where
            | Self::GroupBy
            | Self::Having
            | Self::OrderBy => true,
            _ => false,
        }
    }
}

impl fmt::Display for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Select => write!(f, "SELECT"),
            Self::Insert => write!(f, "INSERT"),
            Self::Update => write!(f, "UPDATE"),
            Self::Delete => write!(f, "DELETE"),
            Self::From => write!(f, "FROM"),
            Self::Where => write!(f, "WHERE"),
            Self::GroupBy => write!(f, "GROUP BY"),
            Self::OrderBy => write!(f, "ORDER BY"),
            Self::Join => write!(f, "JOIN"),
            Self::Into => write!(f, "INTO"),
            Self::InnerJoin => write!(f, "INNER JOIN"),
            Self::LeftJoin => write!(f, "LEFT JOIN"),
            Self::RightJoin => write!(f, "RIGHT JOIN"),
            Self::FullJoin => write!(f, "FULL JOIN"),
            Self::Values => write!(f, "VALUES"),
            Self::On => write!(f, "ON"),
            Self::As => write!(f, "AS"),
            Self::Distinct => write!(f, "DISTINCT"),
            Self::All => write!(f, "ALL"),
            Self::Exists => write!(f, "EXISTS"),
            Self::Having => write!(f, "HAVING"),
            Self::Union => write!(f, "UNION"),
            Self::Not => write!(f, "NOT"),
            Self::And => write!(f, "AND"),
            Self::Or => write!(f, "OR"),
            Self::Asc => write!(f, "ASC"),
            Self::Desc => write!(f, "DESC")
        }
    }
}

pub trait KeywordExt {
    fn has_suffix(&self) -> bool;
}

impl KeywordExt for String {
    fn has_suffix(&self) -> bool {
        match self.to_uppercase().as_str() {
            "GROUP"
            | "ORDER"
            | "INNER"
            | "LEFT"
            | "OUTER"
            | "RIGHT"
            | "FULL" => true,
            _ => false,
        }
    }
}