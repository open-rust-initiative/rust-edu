use crate::system::pwd::Password;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::{fs, io};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct DbSystem {
    sys_name: String,
    sys_username: String,
    sys_password: Password,
}

impl DbSystem {
    pub fn load_cfg(&mut self) -> io::Result<()> {
        let file = File::open("cfg.bin")?;
        *self = serde_json::from_reader(file)?;
        Ok(())
    }
    pub fn new() -> DbSystem {
        let mut sys = DbSystem {
            sys_name: "".to_string(),
            sys_username: "".to_string(),
            sys_password: Password {
                hashed_password: "".to_string(),
            },
        };
        sys.load_cfg().unwrap();
        sys
    }

    pub fn login(&self, username: String, password: String) -> bool {
        let mut status = true;
        status &= username == self.sys_username;
        status &= self.sys_password.check_password(&*password.to_string());
        status
    }

    pub fn update_info(&self) -> io::Result<()> {
        let serialized_data = serde_json::to_string(&self)?;
        let mut file = File::create("cfg.bin")?;
        file.write_all(serialized_data.as_bytes())?;
        Ok(())
    }

    pub fn change_pwd(&mut self, pwd: String) -> io::Result<()> {
        self.sys_password.set_password(pwd.as_str());
        self.update_info()
    }

    pub fn init_cfg() {
        let file_path = "cfg.bin".to_string();
        if fs::metadata(file_path).is_ok() {
            return;
        } else {
            eprintln!("Initializing the SimpleDB System.");
            DbSystem {
                sys_name: "SimpleDB System Based on Rust".to_string(),
                sys_username: "root".to_string(),
                sys_password: Password::new("123456"),
            }
                .update_info()
                .unwrap();
            if !fs::metadata("sql_files").is_ok() {
                fs::create_dir("sql_files").unwrap();
            }
            eprintln!("Complete initialization.\nDefault username root, password 123456");
            return;
        }
    }
}
