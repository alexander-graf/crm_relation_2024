// config.rs
use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use std::default::Default;

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

    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_str = fs::read_to_string("db_config.json")?;
        let config: DbConfig = serde_json::from_str(&config_str)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_str = serde_json::to_string(self)?;
        fs::write("db_config.json", config_str)?;
        Ok(())
    }
}

pub fn initialize_db_config() -> Result<(), Box<dyn std::error::Error>> {
    match DbConfig::load() {
        Ok(config) => {
            *DB_CONFIG.lock().unwrap() = Some(config);
            Ok(())
        }
        Err(e) => Err(e),
    }
}
