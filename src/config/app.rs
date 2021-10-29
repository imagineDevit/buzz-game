use crate::config::db::DBConfig;

use serde::{Deserialize, Serialize};

pub const CLASSPATH: &str = "./resources";

/// ##Configuration
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Config {
    app: AppConfig,
}

/// ##Application whole configuration
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct AppConfig {
    pub name: String,
    pub db: DBConfig,
}

/// ##Init configuration
///
/// Extract configuration from config file located at ./resources/config.yaml
pub fn init_config() -> Result<AppConfig, crate::errors::error::CustomError> {
    let path = format!("{}/config.yaml", CLASSPATH);
    let config_str = std::fs::read_to_string(path)?;
    let config: Config = serde_yaml::from_str(&config_str)?;
    Ok(config.app)
}
