use std::cmp::Ordering;
use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::{env,fs, path};
use std::fs::File;
use std::error::Error;
use std::io::{Write, Read};
use std::path::{Path, PathBuf};
use crypto::{sha1::Sha1, digest::Digest};
use flate2::{Compression, read::ZlibEncoder, write::ZlibDecoder};
use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct Config{
    operate: String,
    argument: Vec<String>,
}

impl Config{
    /**
     *  'build'将minigit指令（通常是命令行输入）装配为Config（配置）
     * # 示例
     * ```
        let v = vec!["init","test"];
        let it = v.iter().map(|x|{x.to_string()});
        let config = Config::build(it).unwrap_or_else(|err| {
            eprintln!("error at test build: {err}");
            std::process::exit(1);
        });
        assert_eq!((config.operate,config.argument),(String::from("init"),String::from("test")));
     * ```
     */
    pub fn build(mut args: impl Iterator<Item = String>)-> Result<Config, &'static str>{
        args.next();
        let operate = match args.next(){
            Some(op) => op,
            None => return Err("Didn't get operate string in input")
        };
        let mut argument = Vec::new();
        while let Some(arg) = args.next(){
            argument.push(arg);
        };
        Ok(
            Config {
                operate,
                argument,
            }
        )
    }
}



/** 'run' 通过输入配置，通过运行对应函数来实现对应的minigit指令 \\
 * # 示例
 * ```
    let config: Config = Config{operate:"init".to_string(), argument:"test".to_string()};
    run(config);
    let path =  std::env::current_dir().unwrap_or_else(|err|{
        eprintln!("test_run failed at get current dir: {err}");
        std::process::exit(1);
    });
    assert!(path.join(config.argument).exists());      
 * ```
 */
pub fn run(config: &Config)-> Result<(), Box<dyn Error>>{
    match &config.operate as &str{
        "init" => {
            if config.argument.is_empty() {
                return Err("minigit init failed: repository name is empty".into());
            }
            init(&config.argument[0])?;
        },
        "add" => {
            add(&config.argument)?;
            println!("Successed add file: {:?}",&config.argument);
        },
        "rm" => {
            rm(&config.argument)?;
            println!("Successed remove file: {:?}",&config.argument);
        },
        "commit" => {
            let mut message = &String::new();
            if !config.argument.is_empty() {
                message = &config.argument[0];
            }
            let author = env::var("USERNAME")?;
            commit(&author, message)?;
            println!("Successed commit repository with message: \"{}\"",&config.argument[0]);
        },
        "branch" => {
            let arg = &config.argument;
            if arg.is_empty() {
                branch_check()?;
            }
            else if arg[0] == "-d".to_string() {
                if arg.len() < 2 {
                    return Err("minigit branch failed: branch name is empty".into());
                }
                branch_delete(&arg[1])?;
                println!("Successed delete branch {}",&arg[1]);
            }
            else {
                branch_create(&arg[0])?;
                println!("Successed create branch {}",&arg[0]);
            }
        },
        "checkout" => {
            let arg = &config.argument;
            let n = arg.len();
            if n == 0 {
                return Err("minigit checkout failed: branch name is empty".into());
            }
            else if n >= 2 && arg[0] == "-b"{
                checkout_new_branch(&arg[1])?;
                println!("Switched to branch {}", &arg[1]);
            }
            else{
                checkout(&arg[0])?;
                println!("Switched to branch {}", &arg[0]);
            }
        },
        "merge" => {
            if config.argument.is_empty() {
                return Err("Please input merge branch name".into());
            }
            merge(&config.argument[0])?;
        },
        _=> return Err("inviald operater string".into()),
    }
    Ok(())
}



/**
 * 'init'根据输入的仓库名字创建minigit仓库，如果该仓库已经存在会将配置初始化，仓库内容不变
 * # 示例
 * ```
 *      let name = "test".to_string();
        let _unused = init(name).unwrap_or_else(|err|{
            eprintln!("error at test_init: {err}");
        });
        let path = env::current_dir().unwrap();
        assert!(path.join("test").is_dir());
 * ```
 */
fn init(name: &String)-> Result<(), Box<dyn Error>>{
    let path = env::current_dir()?.join(name).join(".minigit");
    let mut is_first = true;
    if path.is_dir() {
        fs::remove_dir_all(&path)?;
        is_first = false;
    }
    fs::create_dir_all(&path)?;
    fs::create_dir_all(path.join("refs/heads"))?;
    fs::create_dir(path.join("objects"))?;
    File::create(path.join("index"))?;
    let mut head = File::create(path.join("HEAD"))?;
    head.write_all("master".as_bytes())?;
    if is_first { 
        println!("Initialized empty Git repository in {}",path.to_str().unwrap());
    }
    else{
        println!("Reinitialized empty Git repository in {}",path.to_str().unwrap());
    }
    Ok(())
}



/**
 * 'find_minigit'从此路径开始寻找'.minigit'文件夹（也就是minigit库配置文件存放的地方）
 */
fn find_minigit(path: &PathBuf)-> Result<PathBuf, Box<dyn Error>>{
    if !path.exists() {
        return Err("find minigit faild: invaild path".into());
    }
    let mut start_path = path.clone();
    if start_path.is_file() {
        if !start_path.pop() {
            return Err("Can't find minigit because path have no parent".into());
        }
    }
    let target = OsStr::new(".minigit");
    loop{
        for entry in start_path.read_dir()? {
            let child_path = entry?.path();
            if Some(target) == child_path.file_name() {
                return Ok(child_path);
            }
        }
        if !start_path.pop() {
            break;
        }
    }
    return Err("failed find minigit repository".into());
}


/**
 * 给定一个向量buf，使用二分查找在里面寻找path_str，返回下标（start, end）
 * 如果start <= end 则说明要找的元素在向量里面，且下标为（start + end） / 2
 * 如果start > end 则必定end == start - 1且要找的元素不在向量里面，而且有buf[end] < path_str < buf[start]
 */
fn find_index(buf: &Vec<Vec<u8>>, path_str: &Vec<u8>)-> (usize, i32) {
    if buf.len() == 0 {
        return (0,-1);
    }
    let mut start: usize = 0;
    let mut end: usize = buf.len() - 1;
    while start <= end {
        let mid: usize = (end + start) / 2;
        let mid_path = buf[mid].clone();
        let (mid_path,_) = mid_path.split_at(mid_path.iter().position(|&x| x == b' ').unwrap());
        let mid_path = mid_path.to_vec();
        match mid_path.cmp(&path_str) {
            Ordering::Less=> {
                start = mid + 1;
            }
            Ordering::Greater=> {
                if mid == 0 {
                    return (0, -1);
                }
                end = mid - 1;
            }
            Ordering::Equal=> { 
                break;
            }
        }
    }
    (start, end as i32)
}

/**
 * 从路径path开始通过index里面的记录而不是实际文件系统来更新index内容（即buf）
 * 
 */
fn updata_index(buf: &mut Vec<Vec<u8>>, path: &PathBuf, root_path: &Path)-> Result<(), Box<dyn Error>> {
    let path = match path.parent() {
        None => return Err("updata index file failed: path don't have enough ancestor to minigit path's parent".into()),
        Some(p)=> p.to_path_buf(),
    };
    if !path.starts_with(root_path) {
        return Ok(());
    }
    // 根据index里面的记录而不是实际文件系统来更新path_ancestor
    let path_str = path.as_os_str().as_encoded_bytes().to_vec();
    let (start, end) = find_index(buf, &path_str);
    let mut find_ptr = start;
    if start as i32 <= end {
        find_ptr = (start + end as usize) / 2 + 1;
    }
    let buf_len = buf.len();
    let mut value = b"tree\0".to_vec();
    while find_ptr < buf_len {
        let find_position = buf[find_ptr].iter().position(|&b| b == b' ').unwrap();
        let find_path = &buf[find_ptr][0..find_position];
        if !find_path.starts_with(&path_str) {
            break;
        }
        let last_separator_index = find_path.windows(path::MAIN_SEPARATOR_STR.len()).rposition(|str| str == path::MAIN_SEPARATOR_STR.as_bytes()).unwrap();
        if find_path[0..last_separator_index] != path_str {
            find_ptr = find_ptr + 1;
            continue;
        }
        let mut find_key = buf[find_ptr][(find_position + 1)..].to_vec();
        let mut file_name = find_path[(last_separator_index + 1)..].to_vec();
        value.append(&mut find_key);
        value.push(b' ');
        value.append(&mut file_name);
        value.append(&mut "\0".as_bytes().to_vec());
        find_ptr = find_ptr + 1;
    }
    let key = save_value(&root_path.join(".minigit"), &value)?;
    let mut add_information = format!(" tree {key}").into_bytes();
    let mut path_str = path.as_os_str().as_encoded_bytes().to_vec();
    path_str.append(&mut add_information);
    if start as i32 <= end {
        buf[(start + end as usize) / 2] = path_str;
    }
    else {
        buf.insert(start, path_str);
    }
    updata_index(buf, &path, root_path)
}



fn start_updata_index(minigit_path: &PathBuf, path: &PathBuf)-> Result<(), Box<dyn Error>> {
    let root_path = match minigit_path.parent() {
        None=> return Err("update index file failed: minigit path have no parent".into()),
        Some(p)=> p,
    };
    let index_path = minigit_path.join("index");
    let mut read = fs::read(&index_path)?;
    let mut buf = [[].to_vec();0].to_vec();
    if !read.is_empty() {
        read.pop();
        buf = read.split(|&x| x == b'\n').map(|bytes| bytes.to_vec()).collect::<Vec<Vec<u8>>>();
    }
    // 接下来应该更新此路径上全部的key
    updata_index(&mut buf, path, root_path)?;
    // 最后将index清空后将buf写入index文件
    let buf: Vec<u8> = buf.iter().flat_map(|v| {let mut w = v.clone(); w.push(b'\n'); w}).collect();
    let mut index = File::create(&index_path)?;
    index.write_all(&buf)?;
    Ok(())
}


fn insert_index(minigit_path: &PathBuf, path: &PathBuf,key: &String)-> Result<(),Box<dyn Error>> {
    let mut path_type = "tree";
    if path.is_file() {
        path_type = "blob";
    }
    let mut add_information = format!(" {path_type} {key}").into_bytes();
    let mut path_str = path.as_os_str().as_encoded_bytes().to_vec();
    let index_path = minigit_path.join("index");
    let mut index = File::open(&index_path)?;
    let mut read = Vec::new();
    index.read_to_end(&mut read)?;
    let mut buf;
    if !read.is_empty() {
        read.pop();
        buf = read.split(|&x| x == b'\n').map(|bytes| bytes.to_vec()).collect::<Vec<Vec<u8>>>();
        let (start, end) = find_index(&buf, &path_str);
        if start as i32 > end {
            path_str.append(&mut add_information);
            buf.insert(start, path_str);
        }
        else { // start <= end 说明此时buf[(start + end) / 2]的路径与path_str一样，该更新而不是插入
            path_str.append(&mut add_information);
            let mid = (start + end as usize) / 2;
            buf[mid] = path_str;
        }
    }
    else {
        path_str.append(&mut add_information);
        buf = [path_str;1].to_vec();
    }
    let buf: Vec<u8> = buf.iter().flat_map(|v| {let mut w = v.clone(); w.push(b'\n'); w}).collect();
    let mut index = File::create(&index_path)?;
    index.write_all(&buf)?;
    Ok(())
}





fn save_value(minigit_path: &PathBuf,value: &Vec<u8>)-> Result<String, Box<dyn Error>> {
    if !minigit_path.is_dir() {
        return Err(".minigit doesn't exists".into());
    }
    // 将value中的数据使用SHA1算法加密成key
    let mut hasher = Sha1::new();
    hasher.input(value);
    let key: &str = &hasher.result_str();
    let save_path = minigit_path.join("objects").join(&key[0..2]);
    if !save_path.is_dir(){
        fs::create_dir(&save_path)?;
    }
    let save_path = save_path.join(&key[2..]);
    if !save_path.is_file(){ 
        let mut save_file = File::create(save_path)?;
        save_file.write_all(value)?;
    }
    Ok(String::from(key))
}

fn save_blob(path: &PathBuf, minigit_path: &PathBuf)-> Result<String, Box<dyn Error>> {
    let file = File::open(path)?;
    // 将path代表的文件的二进制内容使用zlib压缩并且存入字符动态数组value中
    let mut value = Vec::new();
    value.append(&mut "blob\0".as_bytes().to_vec());
    let mut zlib = ZlibEncoder::new(file, Compression::fast());
    zlib.read_to_end(&mut value)?;
    let key = save_value(minigit_path, &value)?;
    insert_index(minigit_path, path, &key)?;
    Ok(key)
}

fn save_tree(path: &PathBuf, minigit_path: &PathBuf)-> Result<String, Box<dyn Error>> {
    let mut value = Vec::new();
    value.append(&mut "tree\0".as_bytes().to_vec());
    for entry in path.read_dir()? {
        let key: String;
        let child_type;
        let child_path = entry?.path();
        let child_name = child_path.file_name();
        let child_name = match child_name {
            None=> return Err(r"save_tree failed: child_path can't end with \..".into()),
            Some(name)=> name,
        };
        if child_path.is_file() {
            child_type = "blob";
            key = save_blob(&child_path, minigit_path)?;
        }
        else {
            child_type = "tree";
            key = save_tree(&child_path, minigit_path)?;
        }
        value.append(&mut format!("{child_type} {key} ").into_bytes());
        value.append(&mut child_name.as_encoded_bytes().to_vec());
        value.append(&mut "\0".as_bytes().to_vec());
    }
    let dir_key = save_value(minigit_path, &value)?;
    insert_index(minigit_path,path,&dir_key)?;
    Ok(dir_key)
}


fn save_object(path: &PathBuf)-> Result<(), Box<dyn Error>> {
    let minigit_path = &find_minigit(&path)?;
    if path == minigit_path {
        return Ok(());
    }
    if path.starts_with(minigit_path) {
        return Err("save_object failed: can't save sub path of minigit path".into());
    }
    if path.is_file() {
        save_blob(path, minigit_path)?;
    }
    else if path.is_dir() {
        let repository_path = match minigit_path.parent() {
            None => return Err("save_object failed: can't get repository path from minigit path".into()), 
            Some(father)=> father,
        };
        if path == repository_path {
            let ignore = OsStr::new(".minigit");
            for entry in path.read_dir()? {
                let entry = entry?;
                if entry.file_name() != ignore {
                    let file_type = entry.file_type()?;
                    if file_type.is_dir() {
                        save_tree(&entry.path(), minigit_path)?;
                    }
                    else if file_type.is_file() {
                        save_blob(&entry.path(), minigit_path)?;
                    }
                    else {
                        return Err("save object failed: can't save symlink file".into());
                    }
                    
                }
            }
            return start_updata_index(minigit_path, minigit_path);
        }
        else {
            save_tree(path, minigit_path)?;
        }
    }
    else {
        return Err("save_object failed: Invaid path".into())
    }
    start_updata_index(minigit_path,path)
}


/**
 * add 函数负责将一系列文件或者文件夹保存到索引，如果已经保存则检查是否有改变，如果有改变则保存改变后的新文件到索引
 */
fn add(paths: &Vec<String>)-> Result<(), Box<dyn Error>> {
    let current_path = env::current_dir()?;
    let paths = paths.iter().map(|str| {
                                    let mut cstr = str.clone();
                                    if  cstr.ends_with('.') {
                                        cstr.pop();
                                        cstr.push('*');
                                    }
                                    current_path.join(cstr)})
                                    .collect::<Vec<PathBuf>>();
    let tag = OsStr::new("*");
    for path in paths {
        let file_name = match path.file_name() {
            None=> return Err(r"add failed: path can't end with \..".into()),
            Some(name)=> name,
        };
        if file_name == tag{
            let mut save_path = path.clone();
            if !save_path.pop() {
                return Err(r"add failed: path have no parent and end with \. or \*".into());
            }
            for entry in save_path.read_dir()? {
                save_object(&entry?.path())?;
            }
        }
        else {
            save_object(&path)?;
        }
    }
    Ok(())
}




fn remove_index(minigit_path: &PathBuf, path: &PathBuf)-> Result<(), Box<dyn Error>> {
    let path_str = path.as_os_str().as_encoded_bytes().to_vec();
    let index_path = minigit_path.join("index");
    let mut index = File::open(&index_path)?;
    let mut buf = Vec::new();
    index.read_to_end(&mut buf)?;
    if buf.len() > 0 {
        buf.pop();
    }
    let mut buf = buf.split(|&x| x == b'\n').map(|bytes| bytes.to_vec()).collect::<Vec<Vec<u8>>>();
    let (start, end) = find_index(&buf, &path_str);
    if start as i32 <= end {
        buf.remove((start + end as usize) / 2);
    }
    // 接下来应该更新此路径上全部的key
    let root_path = match minigit_path.parent() {
        None=> return Err("update index file failed: minigit path have no parent".into()),
        Some(p)=> p,
    };
    updata_index(&mut buf, path, root_path)?;
    // 最后将index清空后将buf写入index文件
    let buf: Vec<u8> = buf.iter().flat_map(|v| {let mut w = v.clone(); w.push(b'\n'); w}).collect();
    let mut index = File::create(&index_path)?;
    index.write_all(&buf)?;
    Ok(())
}

fn remove_blob(minigit_path: &PathBuf, path: &PathBuf)-> Result<(), Box<dyn Error>> {
    fs::remove_file(path)?;
    remove_index(minigit_path, path)?;
    Ok(())
}

fn remove_tree(minigit_path: &PathBuf, path: &PathBuf)-> Result<(), Box<dyn Error>> {
    for entry in path.read_dir()? {
        let child_path = &entry?.path();
        if child_path.is_file() {
            remove_blob(minigit_path, child_path)?;
        }
        else {
            remove_tree(minigit_path, child_path)?;
        }
    }
    fs::remove_dir(path)?;
    remove_index(minigit_path, path)?;
    Ok(())
}


fn remove_object(path: &PathBuf)-> Result<(), Box<dyn Error>> {
    let minigit_path = &find_minigit(&path)?;
    if path == minigit_path {
        return Ok(());
    }
    if path.starts_with(minigit_path) {
        return Err("remove_object failed: can't delete sub path of minigit path".into());
    }
    if path.is_file() {
        remove_blob(minigit_path, path)?;
    }
    else if path.is_dir() {
        let repository_path = match minigit_path.parent() {
            None => return Err("remove_object failed: can't get repository path from minigit path".into()), 
            Some(father)=> father,
        };
        if path == repository_path {
            let ignore = OsStr::new(".minigit");
            for entry in path.read_dir()? {
                let entry = entry?;
                if entry.file_name() != ignore {
                    let file_type = entry.file_type()?;
                    if file_type.is_dir() {
                        remove_tree(minigit_path, &entry.path())?;
                    }
                    else if file_type.is_file() {
                        remove_blob(minigit_path, &entry.path())?;
                    }
                    else {
                        return Err("remove object failed: can't delete symlink file".into());
                    }
                    
                }
            }
        }
        else {
            remove_tree(minigit_path, path)?;
        }
    }
    else {
        return Err("remove_object failed: Invaid path".into())
    }
    start_updata_index(minigit_path,path)
}


fn rm(paths: &Vec<String>)->Result<(), Box<dyn Error>> {
    let current_path = env::current_dir()?;
    let paths = paths.iter().map(|str| {
                                    let mut cstr = str.clone();
                                    if  cstr.ends_with('.') {
                                        cstr.pop();
                                        cstr.push('*');
                                    }
                                    current_path.join(cstr)})
                                    .collect::<Vec<PathBuf>>();
    let tag = OsStr::new("*");
    for path in paths {
        let file_name = match path.file_name() {
            None=> return Err(r"add failed: path can't end with \..".into()),
            Some(name)=> name,
        };
        if file_name == tag{
            let mut remove_path = path.clone();
            if !remove_path.pop() {
                return Err(r"add failed: path have no parent and end with \. or \*".into());
            }
            for entry in remove_path.read_dir()? {
                remove_object(&entry?.path())?;
            }
        }
        else {
            remove_object(&path)?;
        }
    }
    Ok(())
}




fn create_tree_from_index(minigit_path: &PathBuf)-> Result<Vec<u8>, Box<dyn Error>> {
    let index_path = minigit_path.join("index");
    if !index_path.is_file() {
        return Err("commit failed: no such index file".into());
    }
    let mut index = File::open(index_path)?;
    let mut buf = [0;256];
    let n = index.read(&mut buf)?;
    if n == 0 {
        return Ok("\0".as_bytes().to_vec());
    }
    let start = buf.iter().position(|&b| b == b' ').unwrap() + 6;
    let end = buf.iter().position(|&b| b == b'\n').unwrap();
    let key = buf[start..end].to_vec();
    Ok(key)
}


fn commit(author: &String, message: &String)-> Result<(), Box<dyn Error>> {
    let minigit_path = &find_minigit(& env::current_dir()?)?;
    let mut tree_key = create_tree_from_index(minigit_path)?;
    let mut head = File::open(minigit_path.join("HEAD"))?;
    let mut current_commit = String::new();
    head.read_to_string(&mut current_commit)?;
    let current_commit = minigit_path.join(minigit_path.join("refs").join("heads").join(&current_commit));
    let mut parent_commit = String::new();
    if !current_commit.is_file() {
        parent_commit = "\0".to_string();
    }
    else {
        let mut head = File::open(&current_commit)?;
        head.read_to_string(&mut parent_commit)?;
    }
    let now: DateTime<Utc> = Utc::now();
    let mut commit_value = format!("commit\0parent {parent_commit}\nauthor {author}\ndatetime {now}\nnote {message}\ntree ").into_bytes();
    commit_value.append(&mut tree_key);
    let key = save_value(minigit_path, &commit_value)?;
    let mut head = File::create(&current_commit)?;
    head.write_all(&key.into_bytes())?;
    Ok(())
}


fn branch_create(name: &String)-> Result<(), Box<dyn Error>> {
    let minigit_path = find_minigit(&env::current_dir()?)?;
    let branch_path = minigit_path.join("refs").join("heads").join(name);
    if branch_path.is_file() {
        return Err("create branch failed: branch {name} is existing, you can't create a existing branch".into());
    }
    let now_branch_name = fs::read_to_string(minigit_path.join("HEAD"))?;
    let last_commit_key = fs::read(minigit_path.join("refs").join("heads").join(&now_branch_name))?;
    fs::write(branch_path, last_commit_key)?;
    Ok(())
}

fn branch_check()-> Result<(), Box<dyn Error>> {
    let minigit_path = find_minigit(&env::current_dir()?)?;
    let branchs_path = minigit_path.join("refs").join("heads");
    let now_branch_name = fs::read_to_string(minigit_path.join("HEAD"))?;
    for entry in branchs_path.read_dir()? {
        let branch_name = entry?.file_name().into_string().unwrap();
        if branch_name == now_branch_name {
            println!("* {}",branch_name);
        }
        else{
            println!(" {}",branch_name);
        }
    }
    Ok(())
}

fn branch_delete(name: &String)-> Result<(), Box<dyn Error>> {
    let minigit_path = find_minigit(&env::current_dir()?)?;
    let branch_path = minigit_path.join("refs").join("heads").join(name);
    let now_branch_name = fs::read_to_string(minigit_path.join("HEAD"))?;
    if &now_branch_name == name {
        return Err("delete branch failed: can't delete now branch".into());
    }
    if branch_path.is_file() {
        fs::remove_file(branch_path)?;
        return Ok(())
    }
    else {
        return Err("delete branch failed: no such branch".into());
    }
}

fn checkout_new_branch(branch_name: &String)-> Result<(), Box<dyn Error>> {
    branch_create(branch_name)?;
    let minigit_path = find_minigit(&env::current_dir()?)?;
    fs::write(minigit_path.join("HEAD"), &branch_name)?;
    Ok(())
}


fn get_value_from_key(minigit_path: &PathBuf, key: &String)-> Result<Vec<u8>, std::io::Error> {
    fs::read(minigit_path.join("objects").join(&key[0..2]).join(&key[2..]))
}

fn create_file_from_key(minigit_path: &PathBuf, path: &PathBuf, key: &String)-> Result<(), Box<dyn Error>> {
    let value = & get_value_from_key(minigit_path, key)?;
    if &value[0..5] != b"blob\0" {
        return Err("create file from key failed: value type isn't blob".into());
    }
    // 创建文件并将解压的文件内容写入
    let mut z = ZlibDecoder::new(File::create(path)?);
    z.write_all(&value[5..])?;
    Ok(())
}


fn create_tree_from_key(minigit_path: &PathBuf, path: &PathBuf, key: &String)-> Result<(), Box<dyn Error>> {
    fs::create_dir_all(path)?;
    let value = get_value_from_key(minigit_path, key)?;
    if &value[0..5] != b"tree\0" {
        return Err("create tree from key failed: value type isn't tree".into());
    }
    let mut dirs = value[5..].split(|&b| b == b'\0').map(|b| b.to_vec()).collect::<Vec<Vec<u8>>>();
    dirs.pop();
    for dir in dirs.iter() {
        let file_type = &dir[0..4];
        let file_key = &String::from_utf8(dir[5..45].to_vec())?;
        let file_name_as_byte = dir[46..].to_vec();
        let file_name = unsafe{OsString::from_encoded_bytes_unchecked(file_name_as_byte)};
        let file_path = path.join(file_name);
        if file_type == b"blob" {
            create_file_from_key(minigit_path, &file_path, file_key)?;
        }
        else if file_type == b"tree" {
            create_tree_from_key(minigit_path, &file_path, file_key)?;
        }
    }
    Ok(())
}


fn checkout(branch_name: &String)-> Result<(), Box<dyn Error>> {
    let minigit_path = find_minigit(& env::current_dir()?)?;
    let root_path = match minigit_path.parent() {
        None=> return Err("checkout failed: can't get repository path".into()),
        Some(r)=> r.to_path_buf(),
    };
    let now_branch_name = fs::read_to_string(minigit_path.join("HEAD"))?;
    if &now_branch_name == branch_name {
        return Ok(());
    }
    let branch_path = minigit_path.join("refs").join("heads").join(branch_name);
    if !branch_path.is_file() {
        return Err(format!("checkout branch {} failed: no such branch",branch_name).into());
    }
    let commit_key = fs::read_to_string(branch_path)?;
    let commit_value = get_value_from_key(&minigit_path, &commit_key)?;
    // get root_tree_key
    let tree_index = commit_value.iter().rposition(|&b| b == b'\n').unwrap();
    let tree_key = String::from_utf8(commit_value[(tree_index + 6)..].to_vec())?;
    // delete_all root_path without minigit path
    let ignore = OsString::from(".minigit");
    for entry in root_path.read_dir()? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let file_name = entry.file_name();
        let file_path = entry.path();
        if file_type.is_file() {
            fs::remove_file(file_path)?;
        }
        else if file_type.is_dir() {
            if file_name != ignore {
                fs::remove_dir_all(file_path)?;
            }
        }
        else {
            return Err("checkout failed: can't delete symlink file under repository path".into());
        }
    }
    // build new repository without index
    create_tree_from_key(&minigit_path, &root_path, &tree_key)?;
    // update index file. how to do? (choose 1 : rm and create a new idnex file ,then save_object(root_path))
    File::create(minigit_path.join("index"))?;
    save_object(&root_path)?;
    // move HEAD ptr
    fs::write(minigit_path.join("HEAD"), branch_name)?;
    Ok(())
}




fn get_parent_commit(minigit_path: &PathBuf, commit_key: &String)-> Result<String, Box<dyn Error>> {
    let commit_value = get_value_from_key(minigit_path, commit_key)?;
    if &commit_value[0..7] != b"commit\0" {
        return Err("get_parent_commit failed: key type isn't commit".into());
    }
    if commit_value[14] == b'\0' {
        return Ok(String::from("\0"));
    }
    return Ok(String::from_utf8(commit_value[14..54].to_vec())?);
}

fn find_both_ancestor(minigit_path: &PathBuf, commit_key1: &String, commit_key2: &String)-> Result<String, Box<dyn Error>> {
    let mut k1 = commit_key1.clone();
    let mut k2 = commit_key2.clone();
    while k1 != k2 {
        if k1 == "\0".to_string() {
            k1 = commit_key2.clone();
        }
        else {
            k1 = get_parent_commit(minigit_path, &k1)?;
        }
        if k2 == "\0".to_string() {
            k2 = commit_key1.clone();
        }
        else {
            k2 = get_parent_commit(minigit_path, &k2)?;
        }
    }
    Ok(k1)
}



fn get_diff_from_vec<T>(v1: &Vec<T>, v2: &Vec<T>)-> Result<Vec<T>, Box<dyn Error>> {
    todo!();
}

fn diff(minigit_path: &PathBuf, key1: &String, key2: &String)-> Result<Vec<u8>, Box<dyn Error>> {
    let mut blobs_value = Vec::new();
    for key in vec![key1, key2] {
        let buf = get_value_from_key(minigit_path, key)?;
        // 解压value
        let mut z = flate2::read::ZlibDecoder::new(&buf[..]);
        let mut value = Vec::new();
        z.read(&mut value)?;
        let value_line: Vec<Vec<u8>>;
        if &value[0..5] != b"blob\0" {
            return Err("merge blob failed: key type is not blob".into());
        }
        if value.len() == 5 {
            value_line = Vec::new();
        }
        else {
            value_line = value[5..].split(|&b| b == b'\n').map(|str| str.to_vec()).collect::<Vec<Vec<u8>>>();
        }
        blobs_value.push(value_line);
    }
    // 对比两个向量，通过算法找到不同
    let _re = get_diff_from_vec(&blobs_value[0], &blobs_value[1])?;
    todo!();
}




fn merge_blob(branch_name: &String, minigit_path: &PathBuf, path: &PathBuf, blobs_key: &Vec<String>)-> Result<bool, Box<dyn Error>> {
    let mut blobs_value: Vec<Vec<u8>> = Vec::new();
    let mut no_conflict = true;
    for key in blobs_key {
        let buf = get_value_from_key(minigit_path, key)?;
        if &buf[..5] != b"blob\0" {
            return Err("merge blob failed: key type is not blob".into());
        }
        // 解压value
        let mut z = flate2::read::ZlibDecoder::new(&buf[5..]);
        let mut value = Vec::new();
        z.read_to_end(&mut value)?;
        blobs_value.push(value);
    }
    // 合并文件数据，并且标出冲突
    let file_value;
    if blobs_key[0] == blobs_key[1] {
        file_value = blobs_value[0].clone();
    }
    else {
        no_conflict = false;
        println!("Conflict at: {}", path.display());
        let now_branch_name = fs::read_to_string(minigit_path.join("HEAD"))?;
        let mut v = vec![format!("========== {now_branch_name}").into_bytes(), blobs_value[0].clone(),
                                   format!("========== {branch_name}").into_bytes(), blobs_value[1].clone()];
        if blobs_key.len() == 3 {
            v.push(b"========== common ancestor".to_vec());
            v.push(blobs_value[2].clone());
        }
        file_value = v.join(&b'\n');
    }
    fs::write(path, &file_value)?;
    Ok(no_conflict)
}


fn merge_tree(branch_name: &String, minigit_path: &PathBuf, path: &PathBuf, trees_key: &Vec<String>)-> Result<bool, Box<dyn Error>> {
    fs::create_dir_all(&path)?;
    let mut re = true;
    let mut trees_value: Vec<Vec<Vec<u8>>> = Vec::new();
    let mut hashmap: HashMap<Vec<u8>, Vec<u8>> = HashMap::new();
    if trees_key.len() == 3 {
        let mut buf = get_value_from_key(minigit_path, &trees_key[2])?;
        if &buf[0..5] != b"tree\0" {
            return Err("merge tree failed: key type is not tree".into());
        }
        if buf.len() > 5 {
            buf.pop();
            for common_value in buf[5..].split(|&b| b == b'\0') {
                let mut map_key = common_value[46..].to_vec();
                map_key.append(&mut common_value[..4].to_vec());
                let map_value = common_value[5..45].to_vec();
                hashmap.insert(map_key, map_value);
            }
        }
    }
    for key in &trees_key[..2] {
        let mut buf = get_value_from_key(minigit_path, key)?;
        let mut value: Vec<Vec<u8>>;
        if &buf[0..5] != b"tree\0" {
            return Err("merge tree failed: key type is not tree".into());
        }
        if buf.len() == 5 {
            value = Vec::new();
        }
        else {
            buf.pop();
            value = buf[5..].split(|&b| b == b'\0').map(|v| v.to_vec()).collect::<Vec<Vec<u8>>>();
            value.sort_unstable_by(|x,y| {
                                        let ord = x[46..].cmp(&y[46..]);
                                        match ord {
                                            Ordering::Equal => return x[0..4].cmp(&y[0..4]),
                                            _=> return ord, 
                                        }});
        }
        trees_value.push(value);
    }
    let (v1, v2) = (&trees_value[0], &trees_value[1]);
    let (n1, n2)  = (v1.len(), v2.len());
    let (mut it1, mut it2) = (0, 0);
    while it1 < n1 && it2 < n2 {
        let (type1, key1, name1) = (&v1[it1][..4], &v1[it1][5..45], &v1[it1][46..]);
        let (type2, key2, name2) = (&v2[it2][..4], &v2[it2][5..45], &v2[it2][46..]);
        let mut ord = name1.cmp(name2);
        if let Ordering::Equal = ord {
            ord = type1.cmp(type2);
        }
        match ord {
            Ordering::Equal=> {
                it1 = it1 + 1;
                it2 = it2 + 1;
                let mut keys: Vec<String> = vec![String::from_utf8(key1.to_vec())?, String::from_utf8(key2.to_vec())?];
                if trees_key.len() == 3 {
                    let mut com_key = name1.to_vec();
                    com_key.append(&mut type1.to_vec());
                    let com_value = hashmap.get(&com_key);
                    if let Some(c) = com_value {
                        keys.push(String::from_utf8(c.clone())?);
                    }
                }
                let name = unsafe{OsString::from_encoded_bytes_unchecked(name1.to_vec())};
                if type1 == b"tree" {
                    re = re && merge_tree(branch_name, minigit_path, &path.join(&name), &keys)?;
                }
                else {
                    re = re && merge_blob(branch_name, minigit_path, &path.join(&name), &keys)?;
                }
            },
            Ordering::Less=> {
                it1 = it1 + 1;
                let mut is_file_in_common_commit = false;
                if trees_key.len() == 3 {
                    let mut com_key = name1.to_vec();
                    com_key.append(&mut type1.to_vec());
                    let com_value = hashmap.get(&com_key);
                    if let Some(_c) = com_value {
                        is_file_in_common_commit = true;
                    }
                }
                let name = unsafe{OsString::from_encoded_bytes_unchecked(name1.to_vec())};
                if !is_file_in_common_commit {
                    if type1 == b"tree" {
                        create_tree_from_key(minigit_path, &path.join(&name), &String::from_utf8(key1.to_vec())?)?;
                    }
                    else {
                        create_file_from_key(minigit_path, &path.join(&name), &String::from_utf8(key1.to_vec())?)?;
                    }
                }
            },
            Ordering::Greater=> {
                it2 = it2 + 1;
                let mut is_file_in_common_commit = false;
                if trees_key.len() == 3 {
                    let mut com_key = name2.to_vec();
                    com_key.append(&mut type2.to_vec());
                    let com_value = hashmap.get(&com_key);
                    if let Some(_c) = com_value {
                        is_file_in_common_commit = true;
                    }
                }
                let name = unsafe{OsString::from_encoded_bytes_unchecked(name2.to_vec())};
                if !is_file_in_common_commit {
                    if type2 == b"tree" {
                        create_tree_from_key(minigit_path, &path.join(&name), &String::from_utf8(key2.to_vec())?)?;
                    }
                    else {
                        create_file_from_key(minigit_path, &path.join(&name), &String::from_utf8(key2.to_vec())?)?;
                    }
                }
            },
        }
    }
    // 这两个循环只可能运行一个
    while it1 < n1 {
        let (type1, key1, name1) = (&v1[it1][..4], &v1[it1][5..45], &v1[it1][46..]);
        it1 = it1 + 1;
        let mut is_file_in_common_commit = false;
        if trees_key.len() == 3 {
            let mut com_key = name1.to_vec();
            com_key.append(&mut type1.to_vec());
            let com_value = hashmap.get(&com_key);
            if let Some(_c) = com_value {
                is_file_in_common_commit = true;
            }
        }
        let name = unsafe{OsString::from_encoded_bytes_unchecked(name1.to_vec())};
        if !is_file_in_common_commit {
            if type1 == b"tree" {
                create_tree_from_key(minigit_path, &path.join(&name), &String::from_utf8(key1.to_vec())?)?;
            }
            else {
                create_file_from_key(minigit_path, &path.join(&name), &String::from_utf8(key1.to_vec())?)?;
            }
        }
    }
    // 这两个循环只可能运行一个
    while it2 < n2 {
        let (type2, key2, name2) = (&v2[it2][..4], &v2[it2][5..45], &v2[it2][46..]);
        it2 = it2 + 1;
        let mut is_file_in_common_commit = false;
        if trees_key.len() == 3 {
            let mut com_key = name2.to_vec();
            com_key.append(&mut type2.to_vec());
            let com_value = hashmap.get(&com_key);
            if let Some(_c) = com_value {
                is_file_in_common_commit = true;
            }
        }
        let name = unsafe{OsString::from_encoded_bytes_unchecked(name2.to_vec())};
        if !is_file_in_common_commit {
            if type2 == b"tree" {
                create_tree_from_key(minigit_path, &path.join(&name), &String::from_utf8(key2.to_vec())?)?;
            }
            else {
                create_file_from_key(minigit_path, &path.join(&name), &String::from_utf8(key2.to_vec())?)?;
            }
        }
    }
    Ok(re)
}



fn merge(branch_name: &String)-> Result<(), Box<dyn Error>> {
    let minigit_path = find_minigit(&env::current_dir()?)?;
    let now_branch_name = fs::read_to_string(minigit_path.join("HEAD"))?;
    if *branch_name == now_branch_name {
        return Ok(())
    }
    let branchs_path = minigit_path.join("refs").join("heads");
    let now_branch_path = branchs_path.join(&now_branch_name);
    let branch_path = branchs_path.join(branch_name);
    if !branch_path.is_file() {
        return Err(format!("merge failed: no such branch named {branch_name}").into());
    }
    if !now_branch_path.is_file() {
        return Err("merge failed: can't find now branch".into());
    }
    let commit_key = fs::read_to_string(&branch_path)?;
    let now_commit_key = fs::read_to_string(&now_branch_path)?;
    let common_commit_key = find_both_ancestor(&minigit_path, &commit_key, &now_commit_key)?;
    if common_commit_key == "\0".to_string() {
        return Err(format!("merge failed: branch {branch_name} and branch {now_branch_name} have no common ancestor commit").into());
    }
    // 如果没有分支，则快速合并，将指针移动到最新提交即可
    if common_commit_key == commit_key {
        // 说明此时已经在最新提交上，不用操作直接返回
        return Ok(())
    }
    if common_commit_key == now_commit_key {
        // 说明此时要合并的分支比现在的分支进度更远，将指针移动到要合并的分支的最新提交即可
        fs::write(&now_branch_path, &commit_key)?;
    }
    // 如果有分支，则需要三路合并
    // 获得三个提交的tree-key
    let commits_key = vec![&now_commit_key, &commit_key, &common_commit_key];
    let mut trees_key = Vec::new();
    for key in commits_key {
        let value = get_value_from_key(&minigit_path, key)?;
        let tree_index = value.iter().rposition(|&b| b == b'\n').unwrap();
        let tree_key = String::from_utf8(value[(tree_index + 6)..].to_vec())?;
        trees_key.push(tree_key);
    }
    // delete_all root_path without minigit path
    let root_path = match minigit_path.parent() {
        None=> return Err("checkout failed: can't get repository path".into()),
        Some(r)=> r.to_path_buf(),
    };
    let ignore = OsString::from(".minigit");
    for entry in root_path.read_dir()? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let file_name = entry.file_name();
        let file_path = entry.path();
        if file_type.is_file() {
            fs::remove_file(file_path)?;
        }
        else if file_type.is_dir() {
            if file_name != ignore {
                fs::remove_dir_all(file_path)?;
            }
        }
        else {
            return Err("checkout failed: can't delete symlink file under repository path".into());
        }
    }
    // 进行三路合并
    let no_conflict = merge_tree(branch_name, &minigit_path, &root_path, &trees_key)?;
    // 提交合并后的工作目录
    if no_conflict {
        let author = env::var("USERNAME")?;
        commit(&author, &format!("merge {branch_name} to {now_branch_name}"))?;
    }
    Ok(())
}




#[cfg(test)]
mod test{

    use super::*;
    #[test]
    fn test_build(){
        let v = vec!["init","test"];
        let it = v.iter().map(|x|{x.to_string()});
        let config = Config::build(it).unwrap_or_else(|err| {
            eprintln!("error at test build: {err}");
            std::process::exit(1);
        });
        assert_eq!((config.operate,config.argument),(String::from("init"),vec![String::from("test")]));
        //dbg!(config);
    }

    #[test]
    fn test_run(){
        let config: Config = Config{operate:"init".to_string(), argument:vec!["test".to_string()]};
        run(&config).unwrap_or_else(|err|{
            eprintln!("error at test_init: {err}");
        });
        let path =  std::env::current_dir().unwrap_or_else(|err|{
            eprintln!("test_run failed at get current dir: {err}");
            std::process::exit(1);
        });
        assert!(path.join(&config.argument[0]).exists());
    }

    #[test]
    fn test_init(){
        let name = "test".to_string();
        let _unused = init(&name).unwrap_or_else(|err|{
            eprintln!("error at test_init: {err}");
        });
        let path = env::current_dir().unwrap();
        assert!(path.join(name).is_dir());
    }

    #[test]
    fn test_add()-> std::io::Result<()>{
        let name = "test".to_string();
        init(&name).unwrap_or_else(|err|{
            eprintln!("error at test_add: {err}");
        });
        let mut path = env::current_dir().unwrap().join(name);
        env::set_current_dir(&path)?;
        let mut file1 = File::create(path.join("1.txt")).unwrap();
        file1.write_all(b"Hello First World!")?;
        path = path.join("test_dir");
        fs::create_dir_all(&path)?;
        let mut file2 = File::create(path.join("2.txt")).unwrap();
        file2.write_all(b"Hello Second World!")?;
        add(&vec!["*".to_string()]).unwrap_or_else(|err|{
            eprintln!("error at test_add: {err}");
        });
        Ok(())
    }

    #[test]
    fn test_rm()-> std::io::Result<()>{
        test_add()?;
        rm(&vec!["test_dir\\2.txt".to_string()]).unwrap_or_else(|err| {
            eprintln!("error at test_rm: {err}");
        });
        Ok(())
    }

    #[test]
    fn test_commit()-> std::io::Result<()> {
        test_add()?;
        let author_name = "master".to_string();
        let message = "test first commit".to_string();
        commit(&author_name, &message).unwrap_or_else(|err|{
            eprintln!("error at test_commit: {err}");
        });
        Ok(())
    }

    #[test]
    fn test_branch()-> Result<(), Box<dyn Error>> {
        test_commit()?;
        println!("before create");
        branch_check()?;
        branch_create(&"second_branch".to_string())?;
        println!("after create");
        branch_check()?;
        branch_delete(&"second_branch".to_string())?;
        println!("after delete");
        branch_check()?;
        Ok(())
    }

    #[test]
    fn test_checkout()-> Result<(), Box<dyn Error>> {
        env::set_current_dir(&env::current_dir()?.join("test"))?;
        let minigit_path = find_minigit(&env::current_dir()?)?;
        let root_path = minigit_path.parent().unwrap();
        rm(&vec!["*".to_string()])?;
        env::set_current_dir(root_path.parent().unwrap())?;
        test_commit()?;
        println!("before create");
        branch_check()?;
        branch_create(&"second_branch".to_string())?;
        println!("after create second branch");
        branch_check()?;
        fs::write(root_path.join("master.txt"), "This is master branch")?;
        add(&vec!["master.txt".to_string()])?;
        commit(&"master".to_string(), &"master commit".to_string())?;
        checkout(&"second_branch".to_string())?;
        println!("after checkout new branch");
        branch_check()?;
        fs::write(root_path.join("second.txt"), "Test checkout")?;
        add(&vec!["second.txt".to_string()])?;
        commit(&"second".to_string(), &"test_second".to_string())?;
        checkout(&"master".to_string())?;
        println!("after checkout master branch");
        branch_check()?;
        Ok(())
    }


    #[test]
    fn test_merge()-> Result<(), Box<dyn Error>> {
        test_commit()?;
        let root_path = env::current_dir()?;
        println!("root path = {}", root_path.display());
        println!("before create");
        branch_check()?;
        branch_create(&"second_branch".to_string())?;
        println!("after create");
        branch_check()?;
        fs::write(&root_path.join("test_merge.txt"), &"my\nfirst\ntest\nmerge\nin\nbranch\nmaster".as_bytes())?;
        fs::create_dir_all(&root_path.join("test_master"))?;
        add(&vec!["*".to_string()])?;
        commit(&"master".to_string(), &"master_commmit".to_string())?;
        checkout(&"second_branch".to_string())?;
        println!("after checkout");
        branch_check()?;
        fs::write(&root_path.join("test_merge.txt"), &"my\nfirst\ntest\nmerge\nin\nbranch\nsecond branch".as_bytes())?;
        fs::create_dir_all(&root_path.join("test_second"))?;
        add(&vec!["*".to_string()])?;
        commit(&"second branch".to_string(), &"second_commit".to_string())?;
        println!("ready to merge");
        merge(&"master".to_string())?;
        Ok(())
    }

    #[test]
    fn test() {
        for (key, value) in env::vars() {
            println!("{}: {}",key,value);
        }
    }
}
