use mobc_postgres::tokio_postgres::Row;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// ##Player entity representation
#[derive(Deserialize, Serialize, Debug)]
pub struct Player {
    pub id: String,
    pub name: String,
    pub score: u32,
}

impl From<Row> for Player {
    fn from(row: Row) -> Self {
        Self {
            id: row.get("id"),
            name: row.get("name"),
            score: row.get("score"),
        }
    }
}

impl Player {
    /// ##Create a new player with a given name
    ///
    /// __name__ : name of player to create
    pub fn with_name(name: String) -> Player {
        Player {
            id: Uuid::new_v4().to_string(),
            name,
            score: 0,
        }
    }
}
