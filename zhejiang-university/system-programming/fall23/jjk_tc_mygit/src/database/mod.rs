use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::prelude::*;

use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;

pub mod blob;
use blob::Blob;
pub mod object;
use object::Object;
pub mod commit;
use commit::Commit;
use crate::util::*;
pub mod tree;
use tree::{Tree, TREE_MODE};
use crate::index;

pub enum ParsedObject {
    Commit(Commit),
    Blob(Blob),
    Tree(Tree),
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct Entry {
    name: String,
    oid: String,
}

impl Entry {
    pub fn new(name: &str, oid: &str, mode: u32) -> Entry {
        Entry {
            name: name.to_string(),
            oid: oid.to_string(),
        }
    }

    // if user is allowed to executable, set mode to Executable,
    // else Regular
    // fn is_executable(&self) -> bool {
    //     (self.mode >> 6) & 0b1 == 1
    // }

    // fn mode(&self) -> u32 {
    //     if self.mode == TREE_MODE {
    //         return TREE_MODE;
    //     }
    //     if self.is_executable() {
    //         return 0o100755;
    //     } else {
    //         return 0o100644;
    //     }
    // }
}

impl From<&index::Entry> for Entry {
    fn from(entry: &index::Entry) -> Entry {
        Entry {
            name: entry.path.clone(),
            oid: entry.oid.clone(),
        }
    }
}

pub struct Database {
    path: PathBuf,
    objects: HashMap<String, ParsedObject>,
}

impl Database {
    pub fn new(path: &Path) -> Database {
        Database {
            path: path.to_path_buf(),
            objects: HashMap::new(),
        }
    }
    pub fn store<T>(&self, obj: &T) -> Result<(), std::io::Error>
    where
        T: Object,
    {
        let oid = obj.get_oid();
        let content = obj.get_content();

        self.write_object(oid, content)
    }
    fn write_object(&self, oid: String, content: Vec<u8>) -> Result<(), std::io::Error> {
        let object_path = self.object_path(&oid);

        // If object already exists, we are certain that the contents
        // have not changed. So there is no need to write it again.
        if object_path.exists() {
            return Ok(());
        }

        let dir_path = object_path.parent().expect("invalid parent path");
        fs::create_dir_all(dir_path)?;
        let mut temp_file_name = String::from("tmp_obj_");
        temp_file_name.push_str(&generate_temp_name());
        let temp_path = dir_path.join(temp_file_name);

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&temp_path)?;

        let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
        e.write_all(&content)?;
        let compressed_bytes = e.finish()?;

        file.write_all(&compressed_bytes)?;
        fs::rename(temp_path, object_path)?;
        Ok(())
    }
    fn object_path(&self, oid: &str) -> PathBuf {
        let dir: &str = &oid[0..2];
        let filename: &str = &oid[2..];

        self.path.as_path().join(dir).join(filename)
    }
}
