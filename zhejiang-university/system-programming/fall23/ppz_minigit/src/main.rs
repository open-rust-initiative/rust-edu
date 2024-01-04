use std::env;
use minigit::Config;
use std::process;

fn main() {
    let config = Config::build(env::args()).unwrap_or_else(|err| {
        eprintln!("Error at input argument: {err}");
        process::exit(1);
    });
    if let Err(err) = minigit::run(&config){
        eprintln!("Error at make operator: {err}");
        process::exit(1);
    }
}