use thiserror::Error;
use std::result;
use sqlparser::parser::ParserError;

/// 声明类型别名，便于使用
/// 创建特定的Result类型，成功时返回"T"类型的值，失败时返回"SQLRiteError"类型的错误
pub type Result<T> = result::Result<T, SQLRiteError>;

#[derive(Error, Debug, PartialEq)]
pub enum SQLRiteError {
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("General error: {0}")]
    General(String),
    #[error("Unknown command error: {0}")]
    UnknownCommand(String),
    #[error("Not Implemented error: {0}")]
    NotImplemented(String),
    #[error("SQL error: {0:?}")]
    SqlError(#[from] ParserError),
}

/// 反回 SQLRiteError::General 的字符串错误信息
pub fn sqlrite_error(message: &str) -> SQLRiteError {
    SQLRiteError::General(message.to_owned())
}


#[cfg(test)]
mod tests {
    use super::*;

    // 测试sqlrite_error_test函数
    #[test]
    fn sqlrite_error_test() {
        let input = String::from("test error");
        let expected = SQLRiteError::General("test error".to_string());

        let result = sqlrite_error(&input);
        assert_eq!(result, expected);
    }

    // 测试NotImplemented
    #[test]
    fn sqlrite_display_not_implemented_test() {
        let error_string = String::from("Feature not implemented.");
        let input = SQLRiteError::NotImplemented(error_string.clone());

        let expected = format!("Not Implemented error: {}", error_string);
        let result = format!("{}", input);
        assert_eq!(result, expected);
    }

    // 测试General
    #[test]
    fn sqlrite_display_general_test() {
        let error_string = String::from("General error.");
        let input = SQLRiteError::General(error_string.clone());

        let expected = format!("General error: {}", error_string);
        let result = format!("{}", input);
        assert_eq!(result, expected);
    }


    // 测试Internal
    #[test]
    fn sqlrite_display_internal_test() {
        let error_string = String::from("Internet error.");
        let input = SQLRiteError::Internal(error_string.clone());

        let expected = format!("Internal error: {}", error_string);
        let result = format!("{}", input);
        assert_eq!(result, expected);
    }


    // 测试SqlError
    #[test]
    fn sqlrite_display_sqlrite_test() {
        let error_string = String::from("SQL error.");
        let input = SQLRiteError::SqlError(ParserError::ParserError(error_string.clone()));

        let expected = format!("SQL error: ParserError(\"{}\")", error_string);
        let result = format!("{}", input);
        assert_eq!(result, expected);
    }

    // 测试UnknownCommand
    #[test]
    fn sqlrite_unknown_test() {
        let error_string = String::from("Unknown error.");
        let input = SQLRiteError::UnknownCommand(error_string.clone());

        let expected = format!("Unknown command error: {}", error_string);
        let result = format!("{}", input);
        assert_eq!(result, expected);
    }
}
