

use std::env;
use std::io::{self, Write};
use std::collections::HashMap;

mod database;
mod repository;
mod refs;
mod index;
mod workspace;
mod lockfile;
mod util;
mod commands;
use commands::{execute, get_app, CommandContext};



fn main() {
    let mut oenv = HashMap::new();
    oenv.insert("GIT_AUTHOR_NAME".to_string(), "jkji".to_string());
    oenv.insert("GIT_AUTHOR_EMAIL".to_string(), "jkji@exp.com".to_string());

    let ctx = CommandContext {
        dir: env::current_dir().unwrap(),
        env: &oenv,
        options: None,
        stdin: io::stdin(),
        stdout: io::stdout(),
        stderr: io::stderr(),
    };

    let matches = get_app().get_matches();

    match execute(matches, ctx) {
        Ok(_) => (),
        Err(msg) => {
            io::stderr().write_all(msg.as_bytes()).unwrap();
            std::process::exit(128);
        }
    }
    
    println!("Yes!");
}
