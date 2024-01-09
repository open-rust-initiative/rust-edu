use crate::meta_command::*;
use crate::sql::*;

use std::borrow::Cow::{self, Borrowed, Owned};

use rustyline::config::OutputStreamType;
use rustyline::error::ReadlineError;
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::validate::Validator;
use rustyline::validate::{ValidationContext, ValidationResult};
use rustyline::{CompletionType, Config, Context, EditMode};
use rustyline_derive::{Completer, Helper};

/// 命令分为 MetaCommand 和 SQLCommand 两种类型
#[derive(Debug, PartialEq)]
pub enum CommandType {
    MetaCommand(MetaCommand),
    SQLCommand(SQLCommand),
}

/// 返回在 REPL 中输入的命令类型
pub fn get_command_type(command: &String) -> CommandType {
    match command.starts_with(".") {
        true => CommandType::MetaCommand(MetaCommand::new(command.to_owned())),
        false => CommandType::SQLCommand(SQLCommand::new(command.to_owned())),
    }
}

#[derive(Helper, Completer)]
pub struct REPLHelper {
    // pub validator: MatchingBracketValidator,
    pub colored_prompt: String,
    pub hinter: HistoryHinter,
    pub highlighter: MatchingBracketHighlighter,
}

/// 实现 Default 特性以给 REPLHelper 默认值
impl Default for REPLHelper {
    fn default() -> Self {
        Self {
            highlighter: MatchingBracketHighlighter::new(),
            hinter: HistoryHinter {},
            colored_prompt: "".to_owned(),
        }
    }
}

/// 提供提示
impl Hinter for REPLHelper {
    type Hint = String;

    /// 获取当前光标所在的正在编辑的行，并返回应显示的字符串；如果用户当前键入的文本没有提示，则返回 None
    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<String> {
        self.hinter.hint(line, pos, ctx)
    }
}

/// 实现负责确定当前输入缓冲区是否有效的特性
/// Rustyline 使用此特性提供的方法来决定按下 Enter 键是否会结束正在编辑的会话，
/// 并将当前行缓冲区返回给 Editor::readline 或变种的调用方法
impl Validator for REPLHelper {
    /// 接收当前编辑的输入，并返回一个 ValidationResult 以验证它是否有效，或是不完整
    fn validate(&self, ctx: &mut ValidationContext) -> Result<ValidationResult, ReadlineError> {
        use ValidationResult::{Incomplete, /*Invalid,*/ Valid};
        let input = ctx.input();
        let result = if input.starts_with(".") {
            Valid(None)
        } else if !input.ends_with(';') {
            Incomplete
        } else {
            Valid(None)
        };
        Ok(result)
    }
}

/// 使用 ANSI 颜色实现语法高亮
impl Highlighter for REPLHelper {
    /// 接收 prompt 并返回高亮显示的版本
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        if default {
            Borrowed(&self.colored_prompt)
        } else {
            Borrowed(prompt)
        }
    }

    /// 接收 hint 并返回高亮显示的版本
    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned("\x1b[1m".to_owned() + hint + "\x1b[m")
    }

    /// 将当前光标所在的正在编辑的行转为高亮显示的版本
    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    /// 判断字符是否是某种特定类型，或光标移到某个特定字符，是则相应部分需要高亮
    /// 用于在插入字符或移动光标时优化刷新
    fn highlight_char(&self, line: &str, pos: usize) -> bool {
        self.highlighter.highlight_char(line, pos)
    }
}

/// 返回具有基本 Editor 配置的 Config::builder
pub fn get_config() -> Config {
    Config::builder()
        .history_ignore_space(true)
        .completion_type(CompletionType::List)
        .edit_mode(EditMode::Emacs)
        .output_stream(OutputStreamType::Stdout)
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_command_type_meta_command_test() {
        let input = String::from(".help");
        let expected = CommandType::MetaCommand(MetaCommand::Help);
        let result = get_command_type(&input);
        assert_eq!(result, expected);
    }

    #[test]
    fn get_command_type_sql_command_test() {
        let input = String::from("SELECT * from users;");
        let expected = CommandType::SQLCommand(SQLCommand::Unknown(input.clone()));
        let result = get_command_type(&input);
        assert_eq!(result, expected);
    }
}
