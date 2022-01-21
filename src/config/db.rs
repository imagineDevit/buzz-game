use serde::{Deserialize, Serialize};

/// ##Database configuration
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct DBConfig {
    pub host: String,
    pub port: u32,
    pub username: String,
    pub password: String,
    pub database: String,
}

/// ToString trait implementation
impl ToString for DBConfig {
    /// ##ToString
    ///
    /// Returns the database url
    fn to_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.password, self.username, self.host, self.port, self.database
        )
    }
}
