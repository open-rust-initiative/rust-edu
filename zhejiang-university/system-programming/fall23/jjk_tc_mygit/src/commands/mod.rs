use clap::{App, Arg, ArgMatches, SubCommand};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::PathBuf;

mod init;
use init::init_command;
mod add;
use add::add_command;
mod commit;
use commit::commit_command;
mod remove;
use remove::rm_command;

// 当前目录、环境变量、命令行参数


pub struct CommandContext<'a, I, O, E>
where
    I: Read,
    O: Write,
    E: Write,
{
    pub dir: PathBuf,
    pub env: &'a HashMap<String, String>,
    pub options: Option<ArgMatches<'a>>,
    pub stdin: I,
    pub stdout: O,
    pub stderr: E,
}

pub fn get_app() -> App<'static, 'static> {
    App::new("mygit")
        .subcommand(
            SubCommand::with_name("init")
            .about("Create an empty Git repository or reinitialize an existing one")
            .arg(Arg::with_name("args").multiple(true)),
        )
        .subcommand(
            SubCommand::with_name("add")
                .about("Add file contents to the index")
                .arg(Arg::with_name("args").multiple(true)),
        )
        .subcommand(
            SubCommand::with_name("commit")
                .about("Record changes to the repository")
                .arg(Arg::with_name("args").multiple(true)),
        )
        .subcommand(
            SubCommand::with_name("rm")
                .about("Romove file contents to the index")
                .arg(Arg::with_name("args").multiple(true)),
        )
        .subcommand(
            SubCommand::with_name("branch")
                .about("List, create branches")
                .arg(Arg::with_name("args").multiple(true)),
        )
        .subcommand(
            SubCommand::with_name("checkout")
                .about("Switch branches or restore working tree files")
                .arg(Arg::with_name("args").multiple(true)),
        )
}

pub fn execute<'a, I, O, E>(
    matches: ArgMatches<'a>,
    mut ctx: CommandContext<'a, I, O, E>,
) -> Result<(), String>
where
    I: Read,
    O: Write,
    E: Write, 
{
    match matches.subcommand() {
        ("init", sub_matches) => {
            ctx.options = sub_matches.cloned();
            init_command(ctx)
        }
        ("add", sub_matches) => {
            ctx.options = sub_matches.cloned();
            add_command(ctx)
        }
        ("commit", sub_matches) => {
            ctx.options = sub_matches.cloned();
            commit_command(ctx)
        }
        ("rm", sub_matches) => {
            ctx.options = sub_matches.cloned();
            rm_command(ctx)
        }
        _ => Ok(()),
    }
}