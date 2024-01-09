
use chrono::{DateTime, Local};
pub enum FileNode {
    // 普通文件
    File {
        base_info: FileBaseInfo,
        content: String
    },
    // 目录文件
    Directory {
        base_info: FileBaseInfo,
        files: Vec<FileNode>
    }
}

struct FileBaseInfo {
    pub name: String,
    create_time: DateTime<Local>,
    modify_time: DateTime<Local>,
    full_path: String,
    size: u32
}

pub fn generate_file(name: String, content: String, path: String) -> FileNode {
    // 根目录的文件path传""
    let file = FileBaseInfo {
        name: name.clone(),
        create_time: Local::now(),
        modify_time: Local::now(),
        full_path: format!("{}/{}", path, name),
        size: 0
    };
    return FileNode::File { base_info: file, content }
}

pub fn generate_diorectory(name: String, path: String) -> FileNode {
    // 根目录两个参数都传""，根目录的子目录path传""
    let file = FileBaseInfo {
        name: name.clone(),
        create_time: Local::now(),
        modify_time: Local::now(),
        full_path: format!("{}/{}", path, name),
        size: 0
    };
    return FileNode::Directory { base_info: file, files: vec![] }
}

impl FileNode {

    // 添加文件或目录
    pub fn add_file(&mut self, file: FileNode) -> Option<&mut FileNode> {
        match self {
            FileNode::File { .. } => None,
            FileNode::Directory { files, .. } => {
                files.push(file);
                return Some(files.last_mut().unwrap());
            }
        }
    }

    // 列出当前节点下的所有文件
    pub fn list_all_files(&self, prefix: String, is_root: bool) {
        if is_root {
            println!("path                type                size                todo");
            println!("----------------------------------------------------------------");
        }

        match self {
            FileNode::File { base_info , ..} => {
                let full_path = format!("{}{}", prefix, base_info.name);
                println!("{:<20}file", full_path);
            },
            FileNode::Directory { base_info, files, .. } => {
                let full_path = format!("{}{}", prefix, base_info.name);
                println!("{:<20}directory", full_path);
                for file in files {
                    if prefix == "" {    // 根目录
                        file.list_all_files(String::from("/"), false);
                    } else {
                        file.list_all_files(format!("{}{}/", prefix, base_info.name), false);
                    }
                }
            }
        }
    }
}