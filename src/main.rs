mod config;
mod data;
mod errors;

use crate::config::app::init_config;
use crate::data::{
    entities::Player,
    repositories::{PlayerRepository, SearchAttributes},
};
use crate::errors::error::CustomError;
use data::db::*;

#[tokio::main]
async fn main() -> Result<(), CustomError> {
    let config = init_config()?;

    let db_pool = create_db_pool(&config)?;

    let pool = db_pool.clone();

    tokio::spawn(async move {
        let connection = get_connection(&pool).await.unwrap();
        let _ = data::db::init_db(&connection).await.unwrap();
    });

    let repository = PlayerRepository::new(db_pool.clone());

    let pl = repository
        .insert(&Player::with_name("Kajk".to_string()))
        .await?;

    let p = repository
        .find_by(SearchAttributes::Name("Joe".to_string()))
        .await?
        .expect("Not found");

    let np = repository.update_score(pl.id, 16).await?;

    println!("Player updated ::: {:?}", np);
    println!("Player found ::: {:?}", p);

    println!("state {:?}", repository.db_pool.state().await);

    Ok(())
}
