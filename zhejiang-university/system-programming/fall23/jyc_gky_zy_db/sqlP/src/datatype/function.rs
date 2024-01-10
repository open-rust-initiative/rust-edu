use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum FunctionT {
    Sum,
    Avg,
    Count,
    Max,
    Min,
    Concat,
}

pub fn to_function(s: &str) -> Option<FunctionT> {
    match s.to_uppercase().as_str() {
        "SUM" => Some(FunctionT::Sum),
        "AVG" => Some(FunctionT::Avg),
        "COUNT" => Some(FunctionT::Count),
        "MAX" => Some(FunctionT::Max),
        "MIN" => Some(FunctionT::Min),
        "CONCAT" => Some(FunctionT::Concat),
        _ => None,
    }
}

impl FunctionT {
    pub fn arg_len(&self) -> u8 {
        match self {
            Self::Sum
            | Self::Avg
            | Self::Count
            | Self::Max
            | Self::Min => 1,
            Self::Concat => 0,
        }
    }
}

impl fmt::Display for FunctionT {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Sum => write!(f, "SUM"),
            Self::Avg => write!(f, "AVG"),
            Self::Count => write!(f, "COUNT"),
            Self::Max => write!(f, "MAX"),
            Self::Min => write!(f, "MIN"),
            Self::Concat => write!(f, "CONCAT"),
        }
    }
}