use crate::data::db::{get_connection, DBPool};
use crate::data::entities::Player;
use crate::errors::error::CustomError;
use mobc_postgres::tokio_postgres::types::ToSql;
use mobc_postgres::tokio_postgres::Row;

const INSERT_QUERY: &str = "INSERT INTO players (id, name, score) VALUES ($1, $2, $3) RETURNING *";
const EXISTS_BY_ID_QUERY: &str = "SELECT exists(SELECT 1 FROM players WHERE id = $1)";
const EXISTS_BY_NAME_QUERY: &str = "SELECT exists(SELECT 1 FROM players WHERE name = $1)";
const FIND_BY_NAME_QUERY: &str = "SELECT * FROM players WHERE name = $1";
const UPDATE_SCORE_QUERY: &str = "UPDATE players SET score = $1 WHERE id = $2 RETURNING *";

pub enum SearchAttributes {
    Name(String),
    Id(String),
}

/// ##Players data access layer
pub struct PlayerRepository {
    pub db_pool: DBPool,
}

impl PlayerRepository {
    /// ###Map from row to player
    ///
    /// __row__ : row to map to player
    fn map_row(row: &Row) -> Player {
        let x: u32 = row.get(2);
        Player {
            id: row.get(0),
            name: row.get(1),
            score: x as u8,
        }
    }

    /// ###Create a new player repository
    ///
    /// __conn__ : database connection associated to the created repository
    pub fn new(db_pool: DBPool) -> Self {
        Self { db_pool }
    }

    /// ###Insert a player into database
    ///
    /// __player__ : player to save
    pub async fn insert(&self, player: &Player) -> Result<Player, CustomError> {
        let exist = self
            .exist_by(SearchAttributes::Name(player.name.clone()))
            .await?;

        if exist {
            Err(CustomError::PlayerAlreadyExistWithNameError(
                player.name.clone(),
            ))
        } else {
            let row = crate::execute_query! {
                pool <- &self.db_pool,
                query <- String::from(INSERT_QUERY),
                params <- &[&player.id, &player.name, &(player.score as u32)]
            };

            Ok(PlayerRepository::map_row(&row))
        }
    }

    /// ##Check player existence with a given name
    ///
    /// __name__ : searched name
    pub async fn exist_by(&self, attribute: SearchAttributes) -> Result<bool, CustomError> {
        let (att, query) = match attribute {
            SearchAttributes::Name(name) => (name, String::from(EXISTS_BY_NAME_QUERY)),
            SearchAttributes::Id(id) => (id, String::from(EXISTS_BY_ID_QUERY)),
        };

        let row = crate::execute_query! {
            pool <- &self.db_pool,
            query <- query,
            params <- &[&att]
        };

        Ok(row.get(0))
    }

    /// ##Find a player with a given name
    ///
    /// __name__ : searched player name
    pub async fn find_by(
        &self,
        attribute: SearchAttributes,
    ) -> Result<Option<Player>, CustomError> {
        let (att, query) = match attribute {
            SearchAttributes::Name(name) => (name, String::from(FIND_BY_NAME_QUERY)),
            SearchAttributes::Id(_id) => panic!("unimplemented yet!!!"),
        };

        let row = crate::execute_query_opt! {
            pool <- &self.db_pool,
            query <- query,
            params <- &[&att]
        };

        Ok(row.map(|ref r| PlayerRepository::map_row(r)))
    }

    /// ##Update player score
    ///
    /// __player__ : updated player
    ///
    /// _return_ the player updated
    pub async fn update_score(
        &self,
        player_id: String,
        new_score: u8,
    ) -> Result<Player, CustomError> {
        let exist = self
            .exist_by(SearchAttributes::Id(player_id.clone()))
            .await?;

        if exist {
            let row = crate::execute_query! {
                pool <- &self.db_pool,
                query <- String::from(UPDATE_SCORE_QUERY),
                params <- &[&(new_score as u32), &player_id]
            };

            Ok(PlayerRepository::map_row(&row))
        } else {
            Err(CustomError::PlayerNodFoundWithIdError(player_id.clone()))
        }
    }
}
