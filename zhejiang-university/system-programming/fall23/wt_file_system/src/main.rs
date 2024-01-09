#![allow(dead_code)]

mod disk_info;

use std::str;
use std::fs;
use std::time::SystemTime;
use std::io::{Write, stdin, stdout};
use disk_info::*;
use disk_info::virtual_disk::*;

const FILE_NAME: &str = "./file_system";

const PROMPT: &str = "\
\n\t----------------------------------------------------------\
\n\t                    rust_file_system\
\n\t----------------------------------------------------------\
\n\tCommands:\
\n\t - cd <directory_name>: Change current directory.\
\n\t - touch <filename>: Create a new file.\
\n\t - ls : List all files and directory in current directory.\
\n\t - cat <filename>: Show the file content.\
\n\t - mkdir <directory name>: Create a new directory.\
\n\t - cp <filename> <new_filename>: Copy a file.\
\n\t - rename <raw_name> <new_name>: Rename a file.\
\n\t - rm <filename>: Delete a file on disk.\
\n\t - mv <filename> <path>: Move a file on disk.\
\n\t - save : Save this virtual disk to file 'file_system'.\
\n\t - diskinfo : Show some info about disk.\
\n\t - exit : Exit the system.\
\n\t - test create <file_name>: Create a random test file.\
\n";

fn main() {
    // 是否从文件读取数据
    let mut virtual_disk: DiskInfo = select_load_file_system(FILE_NAME);
    command_loop(&mut virtual_disk);
}

// 选择是否从文件加载虚拟文件系统
fn select_load_file_system(filename: &str) -> DiskInfo {
    let mut buf_str: String = String::new();
    loop {
        print_info();
        print!("load file system from disk? [Y/N] ");
        stdout().flush().unwrap();
        stdin().read_line(&mut buf_str).unwrap();
        let first_char: char = buf_str.as_str().trim().chars().next().unwrap();

        match first_char {
            'Y' | 'y' => {
                print_info();
                println!("load file system from disk\n");
                let data: Vec<u8> = fs::read(filename).unwrap();
                break bincode::deserialize(data.as_slice()).unwrap();
            }
            'N' | 'n' => {
                print_info();
                println!("new virtual file system\n");
                break DiskInfo::new(None);
            }
            _ => {
                println!("\nIncorrect command.");
                continue;
            }
        };
    }
}

// UI交互界面
fn command_loop(virtual_disk: &mut DiskInfo) {
    // 提示
    println!("{}", PROMPT);

    let mut buf_str: String = String::new();

    loop {
        buf_str.clear();    // 清空buffer
        print!(">  ");
        stdout().flush().unwrap();
        stdin().read_line(&mut buf_str).unwrap();
        // 去除首尾空格
        let command_line: String = String::from(buf_str.trim());

        // 创建文件
        if let Some(cl) = command_line.strip_prefix("touch ") {
            let file_name: &str = cl.trim();
            let data: String = format!("Generate file at {:?} .", SystemTime::now());
            virtual_disk.create_file_with_data(file_name, data.as_bytes());
        } else if command_line.starts_with("help") {
            // 显示菜单
            println!("{}", PROMPT);
        } else if command_line.starts_with("save") {
            // 保存系统
            print_info();
            println!("Saving virtual file system...");
            let data: Vec<u8> = bincode::serialize(&virtual_disk).unwrap();
            fs::write(FILE_NAME, data.as_slice()).unwrap();
            print_info();
            println!("The virtual file system has been saved.\n");
        } else if command_line.starts_with("exit") {
            // 退出文件系统
            print_info();
            println!("Exiting file system...\n");
            break;
        } else if command_line.starts_with("ls") {
            // 列出目录文件
            println!("{}", virtual_disk.cur_directory);
        } else if let Some(command_line) = command_line.strip_prefix("rm ") {
            let file_name: &str = command_line.trim();
            virtual_disk.delete_file_by_name(file_name)
                .expect("[ERROR]\tFile not found, please enter the correct file name!");
        } else if let Some(dir_name) = command_line.strip_prefix("cd ") {
            // 切换到当前目录的某个子目录
            print_info();

            println!("Change Current Directory to: {}", dir_name);
            virtual_disk.change_current_directory(dir_name);
        } else if let Some(command_line) = command_line.strip_prefix("cat ") {
            // 查看文件内容
            let file_name: &str = command_line.trim();
            let data: Vec<u8> = virtual_disk.read_file_by_name(file_name);

            println!("{}", str::from_utf8(data.as_slice()).unwrap());
        } else if let Some(command_line) = command_line.strip_prefix("cp ") {
            // 复制文件
            let name: Vec<&str> = command_line.trim().split(" ").collect();
            if name.len() != 2 {
                println!("Parameter Error!");
                continue;
            }
            virtual_disk.copy_file_by_name(name[0], name[1]);
        } else if command_line.starts_with("diskinfo") {
            // 统计磁盘使用情况
            let (total_size, already_used, unused) = virtual_disk.get_disk_info();
            println!("total size: {} Bytes\nalready use: {} Bytes\navailable: {} Bytes",
                    total_size,
                    BLOCK_SIZE * already_used,
                    BLOCK_SIZE * unused 
            );
        } else if let Some(command_line) = command_line.strip_prefix("mkdir ") {
            // 创建新目录
            let dir_name = command_line.trim();
            virtual_disk.new_directory_to_disk(dir_name).unwrap();
        }  else if let Some(command_line) = command_line.strip_prefix("mv ") {
            // 移动文件
            let name: Vec<&str> = command_line.trim().split(" ").collect();
            if name.len() != 2 {
                println!("Parameter Error!");
                continue;
            }
            if name[1].contains("/") {
                // 移动，path暂时只支持相对路径
                virtual_disk.movie_file_by_name(name[0], name[1]);
            }
        } else if let Some(command_line) = command_line.strip_prefix("rename ") {
            // 重命名
            let name: Vec<&str> = command_line.trim().split(" ").collect();
            if name.len() != 2 {
                println!("Parameter Error!");
                continue;
            }
            virtual_disk.rename_file_by_name(name[0], name[1]);
        } 
        else {
            // 不支持的命令
            println!("Unsupported command");
        }
    }
}
