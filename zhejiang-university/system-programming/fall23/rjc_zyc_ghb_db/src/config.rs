use serde_derive::Deserialize;

use crate::error::Result;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub default_port: String,
    pub default_ip: String,
    pub default_prompt: String,
    pub data_dir: String,
}

impl Config {
    pub fn new(file: &str) -> Result<Self> {
        Ok(config::Config::builder()
            .set_default("default_port", "3306")?
            .set_default("default_ip", "127.0.0.1")?
            .set_default("default_prompt", "waterdb")?
            .set_default("data_dir", "./data")?
            .add_source(config::File::with_name(file))
            .add_source(config::Environment::with_prefix("WATERDB"))
            .build()?
            .try_deserialize()?)
    }
}
