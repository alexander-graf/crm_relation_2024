// config.rs
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::default::Default;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DbConfig {
    pub host: String,
    pub port: String,
    pub username: String,
    pub password: String,
    pub database: String,
}

pub static DB_CONFIG: Lazy<Mutex<Option<DbConfig>>> = Lazy::new(|| Mutex::new(None));

impl Default for DbConfig {
    fn default() -> Self {
        DbConfig {
            host: String::from("localhost"),
            port: String::from("5432"),
            username: String::from(""),
            password: String::from(""),
            database: String::from(""),
        }
    }
}

impl DbConfig {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database
        )
    }

    pub fn load(config_path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let mut file = fs::File::open(config_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let config: DbConfig = serde_json::from_str(&contents)?;
        Ok(config)
    }

    pub fn save(&self, config_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let contents = serde_json::to_string_pretty(self)?;
        let mut file = fs::File::create(config_path)?;
        file.write_all(contents.as_bytes())?;
        Ok(())
    }
}

pub fn initialize_db_config(config_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    match DbConfig::load(config_path) {
        Ok(config) => {
            *DB_CONFIG.lock().unwrap() = Some(config);
            Ok(())
        }
        Err(e) => Err(e),
    }
}

pub fn load_db_config(
    config_path: &PathBuf,
) -> Result<DbConfig, Box<dyn std::error::Error + Send + Sync>> {
    let contents = fs::read_to_string(config_path)?;
    let config: DbConfig = serde_json::from_str(&contents)?;
    Ok(config)
}

pub fn save_db_config(
    config: &DbConfig,
    config_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    config.save(config_path)
}
