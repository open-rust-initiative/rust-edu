use std::path::PathBuf;

use sha1::{Digest, Sha1};

use crate::models::Hash;

use super::util;

/// 管理.mit仓库的读写
pub struct Store {
    store_path: PathBuf,
}

/**Store负责管理objects
 * 每一个object文件名与内容的hash值相同
 */
impl Store {
    fn calc_hash(data: &String) -> String {
        let mut hasher = Sha1::new();
        hasher.update(data);
        let hash = hasher.finalize();
        hex::encode(hash)
    }

    pub fn new() -> Store {
        util::check_repo_exist();
        let store_path = util::get_storage_path().unwrap();
        Store { store_path }
    }
    pub fn load(&self, hash: &String) -> String {
        /* 读取文件内容 */
        let mut path = self.store_path.clone();
        path.push("objects");
        path.push(hash);
        match std::fs::read_to_string(path) {
            Ok(content) => content,
            Err(_) => panic!("储存库疑似损坏，无法读取文件"),
        }
    }

    /** 根据前缀搜索，有歧义时返回 None */
    pub fn search(&self, hash: &String) -> Option<Hash> {
        if hash.is_empty() {
            return None;
        }
        let objects = util::list_files(self.store_path.join("objects").as_path()).unwrap();
        // 转string
        let objects = objects
            .iter()
            .map(|x| x.file_name().unwrap().to_str().unwrap().to_string())
            .collect::<Vec<String>>();
        let mut result = None;
        for object in objects {
            if object.starts_with(hash) {
                if result.is_some() {
                    return None;
                }
                result = Some(object);
            }
        }
        result
    }

    pub fn save(&self, content: &String) -> Hash {
        /* 保存文件内容 */
        let hash = Self::calc_hash(content);
        let mut path = self.store_path.clone();
        path.push("objects");
        path.push(&hash);
        // println!("Saved to: [{}]", path.display());
        if path.exists() {
            // IO优化，文件已存在，不再写入
            return hash;
        }
        match std::fs::write(path, content) {
            Ok(_) => hash,
            Err(_) => panic!("储存库疑似损坏，无法写入文件"),
        }
    }

    pub fn dry_save(&self, content: &String) -> Hash {
        /* 不实际保存文件，返回Hash */
        #[warn(clippy::let_and_return)]
        let hash = Self::calc_hash(content);
        // TODO more such as  check
        hash
    }
}
#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::utils::test;

    #[test]
    fn test_new_success() {
        test::setup_with_clean_mit();
        let _ = Store::new();
    }

    #[test]
    #[should_panic]
    fn test_new_fail() {
        test::setup_without_mit();
        let _ = Store::new();
    }

    #[test]
    fn test_save_and_load() {
        test::setup_with_clean_mit();
        let store = Store::new();
        let content = "hello world".to_string();
        let hash = store.save(&content);
        let content2 = store.load(&hash);
        assert_eq!(content, content2, "内容不一致");
    }

    #[test]
    fn test_search() {
        test::setup_with_clean_mit();
        let hashs = vec!["1234567890".to_string(), "1235467891".to_string(), "4567892".to_string()];
        for hash in hashs.iter() {
            let mut path = util::get_storage_path().unwrap();
            path.push("objects");
            path.push(hash);
            fs::write(path, "hello world").unwrap();
        }
        let store = Store::new();
        assert!(store.search(&"123".to_string()).is_none()); // 有歧义
        assert!(store.search(&"1234".to_string()).is_some()); // 精确
        assert!(store.search(&"4".to_string()).is_some()); // 精确
        assert!(store.search(&"1234567890123".to_string()).is_none()); // 不匹配
    }
}
