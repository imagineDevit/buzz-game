use crate::dto::requests::AddPlayerQuery;
use crate::web::handlers::BuzzHandlers;
use crate::BuzzService;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::{get, path, post, Filter, Rejection, Reply};

pub struct Routes {}

impl Routes {
    pub fn add_player(
        service: Arc<Mutex<BuzzService>>,
    ) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
        crate::routes!(get -> service, |s, q| async {
            BuzzHandlers::add_player(s, q).await
        })
    }

    pub fn register_buzz(
        service: Arc<Mutex<BuzzService>>,
    ) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
        crate::routes!(post -> "buzz", service, |s, r| async {
            BuzzHandlers::register_buzz(s, r).await
        })
    }

    pub fn register_answer(
        service: Arc<Mutex<BuzzService>>,
    ) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
        crate::routes!(post -> "answer", service, |s, r| async {
            BuzzHandlers::register_answer(s, r).await
        })
    }

    pub fn with_service(
        service: Arc<Mutex<BuzzService>>,
    ) -> impl Filter<Extract = (Arc<Mutex<BuzzService>>,), Error = Infallible> + Clone {
        warp::any().map(move || service.clone())
    }
}
