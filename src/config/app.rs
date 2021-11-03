use crate::config::db::DBConfig;

use serde::{Deserialize, Serialize};
use tokio::fs::read_to_string;

pub const CLASSPATH: &str = "./resources";
const CONFIG_FILE_BASE_NAME: &str = "config";

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
pub async fn init_config(
    profile: Option<String>,
) -> Result<AppConfig, crate::errors::error::CustomError> {
    let path = match profile {
        None => format!("{}/{}.yaml", CLASSPATH, CONFIG_FILE_BASE_NAME),
        Some(prefix) => format!("{}/{}-{}.yaml", CLASSPATH, CONFIG_FILE_BASE_NAME, prefix),
    };

    let config_str = read_to_string(path).await?;

    let config: Config = serde_yaml::from_str(&config_str)?;

    Ok(config.app)
}
