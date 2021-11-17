use std::convert::Infallible;
use std::sync::Arc;

use tokio::sync::Mutex;
use warp::{get, path, post, Filter, Rejection, Reply};

use crate::dto::requests::AddPlayerQuery;
use crate::web::handlers::BuzzHandlers;
use crate::{BuzzService, GameInfo};

pub struct Routes {}

impl Routes {
    pub fn add_player(
        service: Arc<Mutex<BuzzService>>,
        game_info: Arc<Mutex<GameInfo>>,
    ) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
        crate::routes!(get -> service, game_info, |s, g, q| async {
            BuzzHandlers::add_player(s,g, q).await
        })
    }

    pub fn register_buzz(
        service: Arc<Mutex<BuzzService>>,
        game_info: Arc<Mutex<GameInfo>>,
    ) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
        crate::routes!(post -> "buzz", service, game_info, |s, g, r| async {
            BuzzHandlers::register_buzz(s, g, r).await
        })
    }

    pub fn register_answer(
        service: Arc<Mutex<BuzzService>>,
        game_info: Arc<Mutex<GameInfo>>,
    ) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
        crate::routes!(post -> "answer", service, game_info, |s, g, r| async {
            BuzzHandlers::register_answer(s, g, r).await
        })
    }

    pub fn with_service(
        service: Arc<Mutex<BuzzService>>,
    ) -> impl Filter<Extract = (Arc<Mutex<BuzzService>>,), Error = Infallible> + Clone {
        warp::any().map(move || service.clone())
    }

    pub fn with_game_info(
        game: Arc<Mutex<GameInfo>>,
    ) -> impl Filter<Extract = (Arc<Mutex<GameInfo>>,), Error = Infallible> + Clone {
        warp::any().map(move || game.clone())
    }
}
