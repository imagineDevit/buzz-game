use crate::config::app::AppConfig;
use crate::errors::error::*;

use mobc::{Connection, Pool};
use mobc_postgres::tokio_postgres::{Config, NoTls};
use mobc_postgres::PgConnectionManager;
use std::str::FromStr;
use std::time::Duration;
use tokio::fs::read_to_string;

// Database pool constants
const DB_POOL_MAX_OPEN: u64 = 32;
const DB_POOL_MAX_IDLE: u64 = 8;
const DB_POOL_TIMEOUT_SECONDS: u64 = 15;

// Sql file path
const INIT_SQL_FILE: &str = "./resources/db/init.sql";
const DROP_SQL_FILE: &str = "./resources/db/drop.sql";

// -- Custom types

/// Connection of PgConnectionManager
pub type DBConn = Connection<PgConnectionManager<NoTls>>;

/// Pool of PgConnectionManager
pub type DBPool = Pool<PgConnectionManager<NoTls>>;

//-- Methods

/// Create a database pool
///
/// __app_config__ : application configuration
pub fn create_db_pool(app_config: &AppConfig) -> Result<DBPool, CustomError> {
    // Retrieve config from db url
    let config = Config::from_str(app_config.db.to_string().as_str())?;

    // Initialize a NoTls Connection manager
    let manager = PgConnectionManager::new(config, NoTls);

    // Build and return the db pool connection
    Ok(Pool::builder()
        .max_open(DB_POOL_MAX_OPEN)
        .max_idle(DB_POOL_MAX_IDLE)
        .get_timeout(Some(Duration::from_secs(DB_POOL_TIMEOUT_SECONDS)))
        .build(manager))
}

/// Get connection from database pool
///
/// __pool__ : database pool
pub async fn get_connection(pool: &DBPool) -> Result<DBConn, CustomError> {
    Ok(pool.get().await?)
}

/// Init a database
///
/// __connection__ : database connection
pub async fn init_db(connection: &DBConn) -> Result<(), CustomError> {
    let init_sql = read_to_string(INIT_SQL_FILE).await?;

    let _ = crate::execute_batch! {
        connection <- connection,
        query <- init_sql
    };

    Ok(())
}

/// Clear a database
///
/// __connection__ : database connection
pub async fn clear_db(connection: &DBConn) -> Result<(), CustomError> {
    let sql = read_to_string(DROP_SQL_FILE).await?;

    let _ = crate::execute_batch! {
        connection <- connection,
        query <- sql
    };

    Ok(())
}
