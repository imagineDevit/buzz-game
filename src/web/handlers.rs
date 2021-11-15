use std::convert::Infallible;
use std::sync::Arc;

use futures_util::Stream;
use tokio::sync::Mutex;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_stream::StreamExt;
use warp::sse::Event;
use warp::{reject, sse, Rejection};

use crate::dto::requests::{AddPlayerQuery, Requests};
use crate::{BuzzService, StateChangeWrapper};

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

    pub async fn emit_events(
        stream: Arc<Mutex<UnboundedReceiverStream<StateChangeWrapper<'static>>>>,
    ) -> Result<impl warp::Reply, Rejection> {
        Ok(sse::reply(
            sse::keep_alive().stream(BuzzHandlers::stream_state(stream).await),
        ))
    }

    fn sse_state(state: StateChangeWrapper<'static>) -> Result<Event, Infallible> {
        let s = serde_json::to_string(&state.state).unwrap();
        Ok(sse::Event::default().data(s))
    }

    async fn stream_state(
        stream: Arc<Mutex<UnboundedReceiverStream<StateChangeWrapper<'static>>>>,
    ) -> impl Stream<Item = Result<Event, Infallible>> + Send {
        let s = stream.lock().await;

        let p = s.map(BuzzHandlers::sse_state);

        p
    }
}
