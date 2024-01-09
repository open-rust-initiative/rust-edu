#[macro_use]
extern crate prettytable;
// 解析命令行参数
extern crate clap;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use repl::{get_command_type, get_config, CommandType, REPLHelper};
use clap::{crate_authors, crate_description, crate_name, crate_version, Command};
use meta_command::handle_meta_command;
use sql::db::database::Database;
use sql::process_command;

mod error;
mod meta_command;
mod repl;
mod sql;

fn main() -> rustyline::Result<()> {
    env_logger::init();

    let _matches = Command::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .get_matches();

    // 用默认配置启动 Rustyline
    let config = get_config();

    // 获取一个 Rustyline 助手
    let helper = REPLHelper::default();

    // 初始化 Rustyline Editor
    let mut repl = Editor::with_config(config);
    repl.set_helper(Some(helper));

    // 将历史文件加载到内存中
    // 如果不存在，则创建一个
    // 待做：如果历史文件过大，则清除
    if repl.load_history("history").is_err() {
        println!("No previous history.");
    }

    // 为用户提供友好的帮助信息
    println!(
        "{} - {}\n{}{}{}{}",
        crate_name!(),
        crate_version!(),
        "Enter .exit to quit.\n",
        "Enter .help for usage hints.\n",
        "Connected to a transient in-memory database.\n",
        "Use '.open FILENAME' to reopen on a persistent database."
    );

    let mut db = Database::new("tempdb".to_string());

    loop {
        let p = format!("Database-Rust> ");
        repl.helper_mut().expect("No helper found").colored_prompt =
            format!("\x1b[1;32m{}\x1b[0m", p);
        // ANSI Color: http://www.perpetualpc.net/6429_colors.html#color_list
        // http://bixense.com/clicolors/

        let readline = repl.readline(&p);
        match readline {
            Ok(command) => {
                repl.add_history_entry(command.as_str());
                // 解析用户输入的命令的类型 (repl::CommandType)
                match get_command_type(&command.trim().to_owned()) {
                    CommandType::SQLCommand(_cmd) => {
                        // process_command 负责将 SQL 语句分词、解析、执行，并返回一个 Result<String, SQLRiteError>
                        let _ = match process_command(&command, &mut db) {
                            Ok(response) => println!("{}", response),
                            Err(err) => eprintln!("An error occured: {}", err),
                        };
                    }
                    CommandType::MetaCommand(cmd) => {
                        // handle_meta_command 解析、执行元命令，并返回 Result<String, SQLRiteError>
                        let _ = match handle_meta_command(cmd, &mut repl) {
                            Ok(response) => println!("{}", response),
                            Err(err) => eprintln!("An error occured: {}", err),
                        };
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                break;
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                eprintln!("An error occured: {:?}", err);
                break;
            }
        }
    }
    repl.append_history("history").unwrap();

    Ok(())
}
