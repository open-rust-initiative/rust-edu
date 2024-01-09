use std::io::{self, Read, Write};

use crate::commands::CommandContext;
use crate::database::blob::Blob;
use crate::database::object::Object;
use crate::repository::Repository;

static INDEX_LOAD_OR_CREATE_FAILED: &'static str = "fatal: could not create/load .git/index\n";

fn locked_index_message(e: &std::io::Error) -> String {
    format!("fatal: {}

Another mygit process seems to be running in this repository. Please make sure all processes are terminated then try again.

If it still fails, a mygit process may have crashed in this repository earlier: remove the .git/index.lock file manually to continue.\n",
            e)
}

fn add_failed_message(e: &std::io::Error) -> String {
    format!(
        "{}

fatal: adding files failed\n",
        e
    )
}
fn remove_failed_message(e: &std::io::Error) -> String {
    format!(
        "{}

fatal: removing files failed\n",
        e
    )
}

fn add_to_index(repo: &mut Repository, pathname: &str) -> Result<(), String> {
    let data = match repo.workspace.read_file(&pathname) {
        Ok(data) => data,
        Err(ref err) if err.kind() == io::ErrorKind::PermissionDenied => {
            repo.index.release_lock().unwrap();
            return Err(add_failed_message(&err));
        }
        _ => {
            panic!("fatal: adding files failed");
        }
    };

    let stat = repo
        .workspace
        .stat_file(&pathname)
        .expect("could not stat file");
    let blob = Blob::new(data.as_bytes());
    repo.database.store(&blob).expect("storing blob failed");

    repo.index.add(&pathname, &blob.get_oid(), &stat);

    Ok(())
}

fn remove_from_index(repo: &mut Repository, pathname: &str) -> Result<(), String> {
    repo.index.remove(&pathname);

    Ok(())
}


pub fn rm_command<I, O, E>(ctx: CommandContext<I, O, E>) -> Result<(), String>
where
    I: Read,
    O: Write,
    E: Write,
{
    let working_dir = ctx.dir;
    let root_path = working_dir.as_path();
    let mut repo = Repository::new(&root_path);
    let options = ctx.options.as_ref().unwrap();
    let args: Vec<_> = if let Some(args) = options.values_of("args") {
        args.collect()
    } else {
        vec![]
    };

    match repo.index.load_for_update() {
        Ok(_) => (),
        Err(ref e) if e.kind() == io::ErrorKind::AlreadyExists => {
            return Err(locked_index_message(e));
        }
        Err(_) => {
            return Err(INDEX_LOAD_OR_CREATE_FAILED.to_string());
        }
    }

    let mut paths = vec![];
    for arg in args {
        let path = working_dir.join(arg);

        for pathname in repo.workspace.list_files(&path).unwrap() {
            paths.push(pathname);
        }
    }

    for pathname in paths {
        remove_from_index(&mut repo, &pathname)?;
    }

    repo.index
        .write_updates()
        .expect("writing updates to index failed");

    Ok(())
}
