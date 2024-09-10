// config.rs
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize, Default)]
pub struct DbConfig {
    pub host: String,
    pub port: String,
    pub username: String,
    pub password: String,
    pub database: String,
}

impl DbConfig {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database
        )
    }
}

pub fn save_db_config(config: &DbConfig) -> Result<(), Box<dyn std::error::Error>> {
    let config_str = serde_json::to_string(config)?;
    fs::write("db_config.json", config_str)?;
    Ok(())
}

pub fn load_db_config() -> Result<DbConfig, Box<dyn std::error::Error>> {
    let config_str = fs::read_to_string("db_config.json")?;
    let config: DbConfig = serde_json::from_str(&config_str)?;
    Ok(config)
}