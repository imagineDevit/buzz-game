#![allow(dead_code)]
mod config;
mod data;
mod errors;

use crate::config::app::init_config;
use crate::data::repositories::PlayerRepository;
use crate::errors::error::CustomError;
use data::db::*;

#[tokio::main]
async fn main() -> Result<(), CustomError> {
    let config = init_config().await?;

    let db_pool = create_db_pool(&config)?;

    let pool = db_pool.clone();

    let handle = tokio::spawn(async move {
        let connection = get_connection(&pool).await.unwrap();
        let _ = data::db::init_db(&connection).await.unwrap();
    });

    let _repository = PlayerRepository::new(db_pool.clone());

    handle.await.unwrap();

    Ok(())
}
