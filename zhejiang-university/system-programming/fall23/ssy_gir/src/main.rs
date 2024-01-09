mod bits;
mod builtin;
mod cli;
mod index;
mod object;
mod refs;
mod sha1;
mod work_dir;
mod zlib;

use std::env;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        print_help();
        return;
    }

    let (args, flags) = cli::split_args_from_flags(args);
    let cmd = &args[1];
    let args = &args[2..];
    if cmd != "init" && cmd != "clone" && !Path::new(".git").exists() {
        println!("Not a top-level git repository");
        return;
    }

    match cmd.as_str() {
        "init" => builtin::init::cmd_init(&args),
        "hash-object" => builtin::hash_object::cmd_hash_object(&args, &flags),
        "cat-file" => builtin::cat_file::cmd_cat_file(&args, &flags),
        "ls-files" => builtin::ls_files::cmd_ls_files(&flags),
        "status" => builtin::status::cmd_status(),
        "diff" => builtin::diff::cmd_diff(&args),
        "add" => builtin::add::cmd_add(&args),
        "rm" => builtin::rm::cmd_rm(&args),
        "write-tree" => builtin::write_tree::cmd_write_tree(),
        "read-tree" => builtin::read_tree::cmd_read_tree(&args),
        "commit" => builtin::commit::cmd_commit(&args, &flags),
        "config" => builtin::config::cmd_config(&args, &flags),
        "log" => builtin::log::cmd_log(),
        "branch" => builtin::branch::cmd_branch(&args, &flags),
        "checkout" => builtin::checkout::cmd_checkout(&args),
        "merge" => builtin::merge::cmd_merge(&args),
        "remote" => builtin::remote::cmd_remote(&args),
        "push" => builtin::push::cmd_push(&args),
        "fetch" => builtin::fetch::cmd_fetch(&args),
        "pull" => builtin::pull::cmd_pull(&args),
        "clone" => builtin::clone::cmd_clone(&args),
        "help" | _ => print_help(),
    }
}

fn print_help() {
    println!("Help | list of commands:");
    println!("* basic commands:");
    println!("\tinit: create empty git repository");
    println!("\tconfig: get and set repo options");
    println!("\tadd: add content to the index");
    println!("\trm: remove content from the files");
    println!("\tcommit: record changes to the repo");
    println!("\tstatus: show the working dir status");
    println!("* branches:");
    println!("\tbranch: list or create branches");
    println!("\tcheckout: switch branches");
    println!("\tmerge: merge two branches together\t");

}
