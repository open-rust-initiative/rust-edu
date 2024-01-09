pub mod virtual_disk;

use std::str;
use core::panic;
use ansi_rgb::Foreground;
use serde::{Deserialize, Serialize};
use std::{fmt, vec::Vec, string::String, usize};
use virtual_disk::{FatStatus, VirtualDisk, BLOCK_SIZE, BLOCK_COUNT};


#[derive(Serialize, Deserialize)]
pub struct DiskInfo {
    pub virtual_disk: VirtualDisk,
    pub cur_directory: Directory,
}


impl DiskInfo {
    // 创建新文件系统，返回DiskInfo对象,root_dir为空表示按默认设置创建
    pub fn new(root_dir: Option<Directory>) -> DiskInfo {
        print_info();
        println!("Creating new file system");
        // 创建VirtualDisk
        let mut disk = VirtualDisk::new();
        {
            // 创建根目录
            let dir_data: Vec<u8> = bincode::serialize(&root_dir).unwrap();
            disk.insert_data_by_offset(dir_data.as_slice(), 0);
        }
        disk.fat[0] = FatStatus::EOF;

        DiskInfo {
            virtual_disk: disk,
            cur_directory: match root_dir {
                // 默认根目录配置
                None => Directory {
                    name: String::from("root"),
                    files: vec![
                        Fcb {
                            name: String::from(".."),
                            file_type: FileType::Directory,
                            first_block: 0,
                            length: 0,
                        },
                        Fcb {
                            name: String::from("."),
                            file_type: FileType::Directory,
                            first_block: 0,
                            length: 0,
                        },
                    ],
                },
                Some(dir) => dir,
            },
        }
    }

    // 遍历查找第一个空闲块的块号
    // TODO 有优化的空间
    pub fn find_next_empty_fat(&self) -> Option<usize> {
        let mut res: Option<usize> = None;
        for i in 0..(self.virtual_disk.fat.len() - 1) {
            if let FatStatus::UnUsed = self.virtual_disk.fat[i] {
                res = Some(i);
                break;
            }
        }
        res
    }

    // 查询是否有指定数量的空闲块，如果有在FAT表中修改相关值，然后返回块号数组
    pub fn allocate_free_space_on_fat(
        &mut self,
        blocks_needed: usize,
    ) -> Result<Vec<usize>, &'static str> {
        print_info();
        println!("Allocating new space...");

        let mut blocks: Vec<usize> = Vec::with_capacity(blocks_needed);
        for i in 0..blocks_needed {
            // 找到一个空闲块
            blocks.push(match self.find_next_empty_fat() {
                Some(block) => block,
                _ => return Err("[ERROR]\tCannot find a NotUsed FatItem!"),
            });
            
            let cur_block: usize = blocks[i];

            // 对磁盘写入数据
            print_debug_info();
            println!("Found new empty block: {}", cur_block);
            if i != 0 {
                // 从第二块开始，将上一块的FAT值修改为当前块
                self.virtual_disk.fat[blocks[i - 1]] = FatStatus::NextBlock(cur_block);
            }
            // 每次都将当前块作为最后一块，防止出现没有空闲块提前退出的情况
            self.virtual_disk.fat[cur_block] = FatStatus::EOF;
        }

        Ok(blocks)
    }

    // 获取以first_block为开头在FAT中所关联的所有文件块
    fn get_file_blocks(&self, first_block: usize) -> Result<Vec<usize>, String> {
        print_info();
        println!("Searching file blocks...");
        let mut blocks: Vec<usize> = Vec::new();
        let mut cur_block: usize = first_block;

        // 第一块
        blocks.push(first_block);

        // 循环读出之后所有块
        loop {
            match self.virtual_disk.fat[cur_block] {
                FatStatus::NextBlock(block) => {
                    print_debug_info();
                    println!("Found next block: {}.", block);
                    blocks.push(block);
                    cur_block = block;
                }
                FatStatus::EOF => {
                    print_debug_info();
                    println!("Found EoF block: {}.", cur_block);
                    break Ok(blocks);
                }
                FatStatus::UnUsed => {
                    break Err(format!(
                        "[ERROR]\tBad block detected at {}!",
                        cur_block
                    ))
                }
            }
        }
    }

    // 释放从first_block开始已经被分配的块
    fn delete_space_on_fat(&mut self, first_block: usize) -> Result<Vec<usize>, String> {
        print_info();
        println!("Deleting Fat space...");
        let blocks_result: Result<Vec<usize>, String> = self.get_file_blocks(first_block);
        let blocks: Vec<usize> = blocks_result.clone().unwrap();
        for block in blocks {
            self.virtual_disk.fat[block] = FatStatus::UnUsed;
        }

        blocks_result
    }

    // 计算写入文件需要的块数量——针对EoF
    // 返回（`bool`: 是否需要插入EoF，`usize`: 需要的总块数）
    fn calc_blocks_needed_with_eof(length: usize) -> (bool, usize) {
        // 需要的块数
        let mut blocks_needed: f32 = length as f32 / BLOCK_SIZE as f32;

        // 需要的块数为整数不需要写入结束标志，否则需要写入结束标志
        let insert_eof: bool = if (blocks_needed - blocks_needed as usize as f32) < 0.0000000001 {
            false
        } else {
            // 向上取整
            blocks_needed = blocks_needed.ceil();
            true
        };
        let blocks_needed: usize = blocks_needed as usize;

        (insert_eof, blocks_needed)
    }

    // 写入的数据到硬盘，返回first_block
    pub fn write_data_to_disk(&mut self, data: &[u8]) -> usize {
        print_info();
        println!("Writing data to disk...");

        let (insert_eof, blocks_needed) = DiskInfo::calc_blocks_needed_with_eof(data.len());

        let blocks: Vec<usize> = self.allocate_free_space_on_fat(blocks_needed).unwrap();

        self.virtual_disk.write_data_by_blocks_with_eof(data, blocks.as_slice(), insert_eof);

        print_debug_info();
        println!("Writing finished. Returned blocks: {:?}", blocks);

        blocks[0]
    }

    // 在当前目录中新建目录，并且写入磁盘
    pub fn new_directory_to_disk(&mut self, name: &str) -> Result<(), &'static str> {
        // 新文件夹写入磁盘块
        print_info();
        println!("Creating dir: {}.", name);
        print_debug_info();
        println!("Trying to write to disk...");

        if let Some(_fcb) = self.cur_directory.get_fcb_by_name(name) {
            return Err("[ERROR]\tThere's already a directory with a same name!");
        }

        // Directory对象是目录的数据，每个数据项是一个Fcb
        let mut new_directory: Directory = Directory::new(name);
        // 添加父目录，用于cd切换到父目录
        new_directory.files.push(Fcb {
            name: String::from(".."),
            file_type: FileType::Directory,
            first_block: self.cur_directory.files[1].first_block,
            length: 0,
        });
        // TODO: 为什么要加入自己？
        new_directory.files.push(Fcb {
            name: String::from("."),
            file_type: FileType::Directory,
            first_block: self.find_next_empty_fat().unwrap(),
            length: 0,
        });

        let bin_dir: Vec<u8> = bincode::serialize(&new_directory).unwrap();

        print_debug_info();
        println!("Dir bytes: {:?}", bin_dir);
        // 将新建的目录写入到硬盘
        let first_block: usize = self.write_data_to_disk(&bin_dir);

        print_debug_info();
        println!("Trying to add dir to current dir...");

        // 在当前目录添加新目录
        self.cur_directory.files.push(Fcb {
            name: String::from(name),
            file_type: FileType::Directory,
            first_block,
            length: 0,
        });
        print_debug_info();
        println!("Created dir {}.", name);

        // 这里并没有立即更新当前目录到硬盘，而是等切换目录或退出时再保存
        // 因为可能创建多个目录，如果每创建一个就更新一次效率会比较低
        // 但也会有新的问题，比如没有正常退出（如断电）会导致数据丢失
        Ok(())
    }

    // 根据首块块号，读出所有数据
    fn get_data_by_first_block(&self, first_block: usize) -> Vec<u8> {
        print_debug_info();
        println!("Getting data from disk by blocks...");

        let blocks: Vec<usize> = self.get_file_blocks(first_block).unwrap();
        let data: Vec<u8> = self
            .virtual_disk
            .read_data_by_blocks_without_eof(blocks.as_slice());

        print_debug_info();
        println!("Data read: {:?}", &data);

        data
    }

    // 通过FCB块找到目录数据
    fn get_directory_by_fcb(&self, dir_fcb: &Fcb) -> Directory {
        print_info();
        println!("Getting dir by FCB...\n\tFCB: {:?}", dir_fcb);
        match dir_fcb.file_type {
            FileType::Directory => {
                let data_dir = self.get_data_by_first_block(dir_fcb.first_block);
                print_debug_info();
                println!("Trying to deserialize data read from disk...");
                let dir: Directory = bincode::deserialize(data_dir.as_slice()).unwrap();
                print_debug_info();
                println!("Getting dir finished.");
                dir
            }
            _ => panic!("[ERROR]\tGet Directory recieved a non-Directory FCB!"),
        }
    }

    // 通过FCB块找到文件数据
    fn get_file_by_fcb(&self, fcb: &Fcb) -> Vec<u8> {
        print_info();
        println!("Getting file data by FCB...\n\tFCB: {:?}", fcb);
        match fcb.file_type {
            FileType::File => self.get_data_by_first_block(fcb.first_block),
            _ => panic!("[ERROR]\tGet File recieved a non-File FCB!"),
        }
    }


    // 在当前目录新建文件并写入数据
    pub fn create_file_with_data(&mut self, name: &str, data: &[u8]) {
        print_info();
        println!("Creating new file in current dir...");
        // 写入数据
        let first_block = self.write_data_to_disk(data);
        // 创建新FCB并插入当前目录中
        let fcb: Fcb = Fcb {
            name: String::from(name),
            file_type: FileType::File,
            first_block: first_block,
            length: data.len(),
        };
        self.cur_directory.files.push(fcb);
    }

    // 通过文件名读取文件
    pub fn read_file_by_name(&self, name: &str) -> Vec<u8> {
        let (_index, fcb) = self.cur_directory.get_fcb_by_name(name).unwrap();
        self.get_file_by_fcb(fcb)
    }

    // 通过文件名删除文件
    pub fn delete_file_by_name(&mut self, name: &str) -> Result<(), String> {
        let index: usize = self.cur_directory.get_index_by_name(name).unwrap();
        // 从dir中先删除fcb，如果删除失败再还回来
        print_debug_info();
        println!("Trying to delete file in dir file list...");
        let fcb: Fcb = self.cur_directory.files.remove(index);
        let res: Result<(), String> = self.delete_file_by_fcb_with_index(&fcb, None);

        if res.is_err() {
            self.cur_directory.files.push(fcb);
        }

        res
    }

    // 首先要清除文件分配表中占用的块，数据区可以不清零，然后还要从父目录中删除对应的FCB
    fn delete_file_by_fcb_with_index(
        &mut self,
        fcb: &Fcb,
        index: Option<usize>,
    ) -> Result<(), String> {
        if let FileType::Directory = fcb.file_type {
            let dir: Directory = self.get_directory_by_fcb(fcb);
            if dir.files.len() > 2 {
                return Err(String::from("[ERROR]\tThe Directory is not empty!"));
            }
        }
        print_debug_info();
        println!(
            "Trying to set all NotUsed clutster of file '{}' on FAT...",
            fcb.name
        );
        // 直接返回删除文件的结果
        if let Err(err) = self.delete_space_on_fat(fcb.first_block) {
            return Err(err);
        }
        // 若给定index非None，则删除目录下的FCB条目
        if let Some(i) = index {
            self.cur_directory.files.remove(i);
        }

        Ok(())
    }

    // 切换到指定目录
    pub fn change_current_directory(&mut self, name: &str) {
        // 先保存当前目录数据到硬盘
        let dir_cloned: Directory = self.cur_directory.clone();
        self.save_directory_to_disk(&dir_cloned);
        // 通过name获取要切换到的目录fcb
        let (_index, dir_fcb) = self.cur_directory.get_fcb_by_name(name).unwrap();

        let dir: Directory = self.get_directory_by_fcb(dir_fcb);
        self.cur_directory = dir;
    }

    // 保存当前目录数据到硬盘，返回第一个块号——更改被保存，原目录文件将在磁盘上被覆盖
    fn save_directory_to_disk(&mut self, dir: &Directory) -> usize {
        print_debug_info();
        println!("Trying to saving dir...");
        let data = bincode::serialize(dir).unwrap();
        let (insert_eof, blocks_needed) = DiskInfo::calc_blocks_needed_with_eof(data.len());
        // 删除原先的块
        self.delete_space_on_fat(self.cur_directory.files[1].first_block).unwrap();
        // 分配新的块
        let reallocated_blocks = self.allocate_free_space_on_fat(blocks_needed).unwrap();
        self.virtual_disk.write_data_by_blocks_with_eof(
            data.as_slice(),
            reallocated_blocks.as_slice(),
            insert_eof,
        );

        reallocated_blocks[0]
    }

    // 文件改名
    // 目录改名要复杂一些，这里没实现
    pub fn rename_file_by_name(&mut self, old: &str, new: &str) {
        let (index, fcb) = self.cur_directory.get_fcb_by_name(old).unwrap();
        let new_fcb: Fcb = Fcb {
            name: String::from(new),
            ..fcb.to_owned()
        };
        self.cur_directory.files[index] = new_fcb;
    }

    // 移动文件
    pub fn movie_file_by_name(&mut self, file_name: &str, path: &str) {
        let index = self.cur_directory.get_index_by_name(file_name).unwrap();
        // 从当前目录中删除fcb
        let fcb: Fcb = self.cur_directory.files.remove(index);
        self.save_directory_to_disk(&self.cur_directory.clone());
        
        let dir_names: Vec<&str> = path.split("/").collect();
        let mut cur_directory = self.cur_directory.clone();
        for dir_name in dir_names {
            if dir_name == "" {
                continue;
            }
            let (_, dir_fcb) = cur_directory.get_fcb_by_name(dir_name).unwrap();
            cur_directory = self.get_directory_by_fcb(dir_fcb);
        }
        cur_directory.files.push(fcb);
        let data: Vec<u8> = bincode::serialize(&cur_directory).unwrap();
        let (insert_eof, blocks_needed) = DiskInfo::calc_blocks_needed_with_eof(data.len());
        // 删除原先的块
        self.delete_space_on_fat(cur_directory.files[1].first_block).unwrap();
        // 分配新的块
        let reallocated_blocks: Vec<usize> = self.allocate_free_space_on_fat(blocks_needed).unwrap();
        self.virtual_disk.write_data_by_blocks_with_eof(
            data.as_slice(),
            reallocated_blocks.as_slice(),
            insert_eof,
        );
    }

    // 获取部分磁盘信息
    // 返回 磁盘总大小/Byte，已分配块数量、未分配块的数量
    pub fn get_disk_info(&self) -> (usize, usize, usize) {
        let disk_size: usize = BLOCK_SIZE * BLOCK_COUNT;
        let mut num_used: usize = 0usize;
        let mut num_not_used: usize = 0usize;

        for fat_item in &self.virtual_disk.fat {
            match fat_item {
                FatStatus::NextBlock(_no) => num_used += 1,
                FatStatus::EOF => num_used += 1,
                FatStatus::UnUsed => num_not_used += 1,
            }
        }

        (disk_size, num_used, num_not_used)
    }

    // FCB的移动
    // 这个也没用上
    pub fn move_fcb_between_dirs_by_name(&mut self, name: &str, des_dir: &mut Directory) {
        let fcb = self
            .cur_directory
            .files
            .remove(self.cur_directory.get_index_by_name(name).unwrap());
        des_dir.files.push(fcb);
    }

    // 复制文件
    pub fn copy_file_by_name(&mut self, raw_name: &str, new_name: &str) -> bool {
        let (_, fcb) = self.cur_directory.get_fcb_by_name(raw_name).unwrap();
        let data: Vec<u8> = self.get_file_by_fcb(fcb);
        self.create_file_with_data(new_name, &data);
        true
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum FileType {
    File,
    Directory,
}
impl fmt::Display for FileType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FileType::Directory => write!(f, "Directory"),
            FileType::File => write!(f, "File"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Fcb {
    name: String,         // 文件名
    file_type: FileType,  // 文件or目录
    first_block: usize,   // 起始块号
    length: usize,        // 文件大小
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Directory {
    name: String,
    files: Vec<Fcb>,
}
impl Directory {
    fn new(name: &str) -> Directory {
        Directory {
            name: String::from(name),
            files: Vec::with_capacity(2),
        }
    }

    // 通过文件名获取文件在files中的索引和文件FCB
    fn get_fcb_by_name(&self, name: &str) -> Option<(usize, &Fcb)> {
        let mut res: Option<(usize, &Fcb)> = None;
        for i in 0..self.files.len() {
            if self.files[i].name.as_str() == name {
                res = Some((i, &self.files[i]));
                break;
            }
        }

        res
    }

    // 通过文件名获取文件在files中的索引
    fn get_index_by_name(&self, name: &str) -> Option<usize> {
        let mut res: Option<usize> = None;
        for i in 0..self.files.len() {
            if self.files[i].name.as_str() == name {
                res = Some(i);
                break;
            }
        }

        res
    }
}

impl fmt::Display for Directory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // 仅将 self 的第一个元素写入到给定的输出流 `f`。返回 `fmt:Result`，此
        // 结果表明操作成功或失败。注意 `write!` 的用法和 `println!` 很相似。
        writeln!(f, "Directroy '{}' Files:", self.name)?;
        for file in &self.files {
            writeln!(
                f,
                "{}\t\t{}\t\tLength: {}",
                file.name, file.file_type, file.length
            )?;
        }

        fmt::Result::Ok(())
    }
}

pub fn print_debug_info() {
    print!("{}", "[DEBUG]\t".fg(ansi_rgb::magenta()));
}

pub fn print_info() {
    print!("{}", "[INFO]\t".fg(ansi_rgb::cyan_blue()));
}