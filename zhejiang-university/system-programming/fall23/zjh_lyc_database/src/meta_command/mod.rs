use crate::error::{Result, SQLRiteError};
use crate::repl::REPLHelper;
use rustyline::Editor;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum MetaCommand {
    Exit,
    Help,
    Open(String),
    Unknown,
}

/// 将类型翻译为具有可读性的格式化文本
impl fmt::Display for MetaCommand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MetaCommand::Exit => f.write_str(".exit"),
            MetaCommand::Help => f.write_str(".help"),
            MetaCommand::Open(_) => f.write_str(".open"),
            MetaCommand::Unknown => f.write_str("Unknown command"),
        }
    }
}

impl MetaCommand {
    pub fn new(command: String) -> MetaCommand {
        let args: Vec<&str> = command.split_whitespace().collect();
        let cmd = args[0].to_owned();
        match cmd.as_ref() {
            ".exit" => MetaCommand::Exit,
            ".help" => MetaCommand::Help,
            ".open" => MetaCommand::Open(command),
            _ => MetaCommand::Unknown,
        }
    }
}

pub fn handle_meta_command(command: MetaCommand, repl: &mut Editor<REPLHelper>) -> Result<String> {
    match command {
        MetaCommand::Exit => {
            repl.append_history("history").unwrap();
            std::process::exit(0)
        }
        MetaCommand::Help => Ok(format!(
            "{}{}{}{}{}{}{}{}",
            "Special commands:\n",
            ".help            - Display this message\n",
            ".open <FILENAME> - Close existing database and reopen FILENAME\n",
            ".save <FILENAME> - Write in-memory database into FILENAME\n",
            ".read <FILENAME> - Read input from FILENAME\n",
            ".tables          - List names of tables\n",
            ".ast <QUERY>     - Show the abstract syntax tree for QUERY.\n",
            ".exit            - Quits this application"
        )),
        MetaCommand::Open(args) => Ok(format!("To be implemented: {}", args)),
        MetaCommand::Unknown => Err(SQLRiteError::UnknownCommand(format!(
            "Unknown command or invalid arguments. Enter '.help'."
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::{get_config, REPLHelper};

    #[test]
    fn get_meta_command_exit_test() {
        // 使用默认配置启动 Rustyline
        let config = get_config();

        // 获得一个 Rustyline 助手
        let helper = REPLHelper::default();

        // 初始化 Rustyline Editor (设置配置和助手)
        let mut repl = Editor::with_config(config);
        repl.set_helper(Some(helper));

        let inputed_command = MetaCommand::Help;

        let result = handle_meta_command(inputed_command, &mut repl);
        assert_eq!(result.is_ok(), true);
    }

    #[test]
    fn get_meta_command_open_test() {
        let config = get_config();

        let helper = REPLHelper::default();

        let mut repl = Editor::with_config(config);
        repl.set_helper(Some(helper));

        let inputed_command = MetaCommand::Open(".open database.db".to_string());

        let result = handle_meta_command(inputed_command, &mut repl);
        assert_eq!(result.is_ok(), true);
    }

    #[test]
    fn get_meta_command_unknown_command_test() {
        let config = get_config();

        let helper = REPLHelper::default();

        let mut repl = Editor::with_config(config);
        repl.set_helper(Some(helper));

        let inputed_command = MetaCommand::Unknown;

        let result = handle_meta_command(inputed_command, &mut repl);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn meta_command_display_trait_test() {
        let exit = MetaCommand::Exit;
        let help = MetaCommand::Help;
        let open = MetaCommand::Open(".open database.db".to_string());
        let unknown = MetaCommand::Unknown;

        assert_eq!(format!("{}", exit), ".exit");
        assert_eq!(format!("{}", help), ".help");
        assert_eq!(format!("{}", open), ".open");
        assert_eq!(format!("{}", unknown), "Unknown command");
    }
}