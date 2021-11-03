use crate::config::db::DBConfig;

use serde::{Deserialize, Serialize};
use tokio::fs::read_to_string;

const ACTIVE_PROFILE: &str = "active-profile";
pub const CLASSPATH: &str = "./resources";
const CONFIG_FILE_BASE_NAME: &str = "config";
const YAML: &str = ".yaml";
const SEPARATOR: &str = "-";

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
pub async fn init_config() -> Result<AppConfig, crate::errors::error::CustomError> {
    let profile = extract_provided_profile();

    let profiles = extract_looked_profiles().await;

    let path = match profile {
        None => format!("{}/{}.yaml", CLASSPATH, CONFIG_FILE_BASE_NAME),
        Some(suffix) => {
            if !profiles.contains(&suffix) {
                panic!("Configuration file not found for profile {}", suffix);
            }
            format!("{}/{}-{}.yaml", CLASSPATH, CONFIG_FILE_BASE_NAME, suffix)
        }
    };

    let config_str = read_to_string(path).await?;

    let config: Config = serde_yaml::from_str(&config_str)?;

    Ok(config.app)
}

/// ##Extract active-profile from env variables
///
/// For example, if application is launched with the following command
///
/// ```
/// cargo run active-profile rec
/// ```
/// This method wil return Some("rec")
fn extract_provided_profile() -> Option<String> {
    let mut profile = Some(String::default());
    let mut args = std::env::args().collect::<Vec<String>>().into_iter();
    while let Some(p) = profile {
        profile = args.next();
        if p.eq(ACTIVE_PROFILE) {
            break;
        }
    }
    profile
}

/// ##Extract profiles for which config file exists in the CLASSPATH directory
///
/// For example, if the config-prod.yaml and config-rec.yaml exist
///
/// this method wil return ["prod", "rec"]
async fn extract_looked_profiles() -> Vec<String> {
    let mut profiles: Vec<String> = Vec::new();
    let mut dir = tokio::fs::read_dir(CLASSPATH).await.unwrap();

    let mut next = true;

    while next {
        match dir.next_entry().await.unwrap() {
            None => {
                next = false;
            }
            Some(entry) => {
                let metadata = entry.metadata().await.unwrap();
                if metadata.is_file() {
                    match entry.file_name().into_string() {
                        Ok(mut filename) => {
                            if filename.starts_with(CONFIG_FILE_BASE_NAME)
                                && filename.ends_with(YAML)
                            {
                                filename = filename
                                    .replace(CONFIG_FILE_BASE_NAME, "")
                                    .replace(YAML, "")
                                    .replace(SEPARATOR, "");

                                if !filename.is_empty() {
                                    profiles.push(filename);
                                }
                            }
                        }
                        Err(_) => {}
                    }
                }
            }
        }
    }
    profiles
}
