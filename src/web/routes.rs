use std::convert::Infallible;
use std::sync::Arc;

use tokio::sync::Mutex;
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::{get, path, post, Filter, Rejection, Reply};

use crate::dto::requests::AddPlayerQuery;
use crate::web::handlers::BuzzHandlers;
use crate::{BuzzService, StateChangeWrapper};

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

    pub fn send_events(
        stream: Arc<Mutex<UnboundedReceiverStream<StateChangeWrapper<'static>>>>,
    ) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
        warp::path!("events" / "states")
            .and(get())
            .and(Routes::with_stream(stream))
            .and_then(|s| async { BuzzHandlers::emit_events(s).await })
    }

    pub fn with_service(
        service: Arc<Mutex<BuzzService>>,
    ) -> impl Filter<Extract = (Arc<Mutex<BuzzService>>,), Error = Infallible> + Clone {
        warp::any().map(move || service.clone())
    }

    pub fn with_stream(
        stream: Arc<Mutex<UnboundedReceiverStream<StateChangeWrapper<'static>>>>,
    ) -> impl Filter<
        Extract = (Arc<Mutex<UnboundedReceiverStream<StateChangeWrapper<'static>>>>,),
        Error = Infallible,
    > + Clone {
        warp::any().map(move || stream.clone())
    }
}
