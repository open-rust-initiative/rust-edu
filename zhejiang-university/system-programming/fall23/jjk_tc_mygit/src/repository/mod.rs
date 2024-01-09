use std::path::{Path, PathBuf};
use crate::database::Database;
use crate::index::Index;
use crate::refs::Refs;
use crate::workspace::Workspace;

pub struct Repository {
    pub database: Database,
    pub index: Index,
    pub refs: Refs,
    pub workspace: Workspace,

    pub root_path: PathBuf,
}

impl Repository {
    pub fn new(root_path: &Path) -> Repository {
        let git_path = root_path.join(".git");
        let db_path = git_path.join("objects");

        Repository {
            database: Database::new(&db_path),
            index: Index::new(&git_path.join("index")),
            refs: Refs::new(&git_path),
            workspace: Workspace::new(git_path.parent().unwrap()),

            root_path: root_path.to_path_buf(),
        }
    }
}