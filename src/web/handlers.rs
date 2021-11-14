use crate::dto::requests::{AddPlayerQuery, Requests};
use crate::BuzzService;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::{reject, Rejection};

pub struct BuzzHandlers {}

impl BuzzHandlers {
    pub async fn add_player(
        service: Arc<Mutex<BuzzService>>,
        query: AddPlayerQuery,
    ) -> Result<impl warp::Reply, Rejection> {
        let resp = service
            .lock()
            .await
            .add_player(query.to_request())
            .await
            .map_err(|e| reject::custom(e))?;
        Ok(warp::reply::json(&resp))
    }

    pub async fn register_buzz(
        service: Arc<Mutex<BuzzService>>,
        request: Requests,
    ) -> Result<impl warp::Reply, Rejection> {
        let resp = service
            .lock()
            .await
            .register_buzz(request)
            .map_err(|e| reject::custom(e))?;

        Ok(warp::reply::json(&resp))
    }

    pub async fn register_answer(
        service: Arc<Mutex<BuzzService>>,
        request: Requests,
    ) -> Result<impl warp::Reply, Rejection> {
        let resp = service
            .lock()
            .await
            .register_answer(request)
            .map_err(|e| reject::custom(e))?;

        Ok(warp::reply::json(&resp))
    }
}
