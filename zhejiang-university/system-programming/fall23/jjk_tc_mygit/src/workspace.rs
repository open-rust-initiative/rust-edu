use std::fs::{self, File};
use std::io::prelude::*;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use lazy_static::lazy_static;

lazy_static! {
    static ref IGNORE_PATHS: Vec<&'static str> = {
        let v = vec![".git", "target"];
        v
    };
}

pub struct Workspace {
    path: PathBuf,
}

impl Workspace {
    pub fn new(path: &Path) -> Workspace {
        Workspace {
            path: path.to_path_buf(),
        }
    }

    /// List contents of directory. Does NOT list contents of
    /// subdire

    /// Return list of files in dir. Nested files are flattened
    /// strings eg. `a/b/c/inner.txt`
    /// 3
    pub fn list_files(&self, dir: &Path) -> Result<Vec<String>, std::io::Error> {
        if dir.is_file() {
            return Ok(vec![dir
                .strip_prefix(&self.path)
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()]);
        }

        if IGNORE_PATHS.contains(&dir.file_name().unwrap().to_str().unwrap()) {
            return Ok(vec![]);
        }

        let mut files = vec![];
        for file in fs::read_dir(dir)? {
            let path = file?.path();
            files.extend_from_slice(&self.list_files(&path)?);
        }
        Ok(files)
    }

    // 1
    pub fn read_file(&self, file_name: &str) -> Result<String, std::io::Error> {
        let file = File::open(self.path.as_path().join(file_name))?;
        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();

        buf_reader.read_to_string(&mut contents)?;
        Ok(contents)
    }
    //2
    pub fn stat_file(&self, file_name: &str) -> Result<fs::Metadata, std::io::Error> {
        fs::metadata(self.path.join(file_name))
    }
}
