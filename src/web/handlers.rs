use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_stream::StreamExt;
use warp::sse::Event;
use warp::{reject, sse, Rejection};

use crate::dto::requests::{AddPlayerQuery, Requests};
use crate::dto::responses::Response;
use crate::{BuzzService, GameInfo, Messages, StateChange};

pub struct BuzzHandlers {}

impl BuzzHandlers {
    pub async fn add_player(
        service: Arc<Mutex<BuzzService>>,
        game_info: Arc<Mutex<GameInfo>>,
        query: AddPlayerQuery,
    ) -> Result<impl warp::Reply, Rejection> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<StateChange>();

        let resp = service
            .lock()
            .await
            .add_player(query.to_request(), game_info.clone(), tx)
            .await
            .map_err(|e| reject::custom(e))?;

        let rx = UnboundedReceiverStream::new(rx);

        let stream = rx.map(|state| {
            let data = serde_json::to_string(&state).unwrap();
            Ok::<Event, Infallible>(sse::Event::default().data(data))
        });

        if let Response::PlayerAdded(ready) = resp {
            if ready {
                let g_i = game_info.clone();

                tokio::spawn(async move {
                    // start the game by sending starting message
                    g_i.lock().await.send(Messages::None, true).await;

                    std::thread::sleep(Duration::from_secs(1));

                    // send next question
                    g_i.lock().await.next_question().await;
                });
            }
        }

        Ok(sse::reply(sse::keep_alive().stream(stream)))
    }

    pub async fn register_buzz(
        service: Arc<Mutex<BuzzService>>,
        game_info: Arc<Mutex<GameInfo>>,
        request: Requests,
    ) -> Result<impl warp::Reply, Rejection> {
        let resp = service
            .lock()
            .await
            .register_buzz(request, game_info.clone())
            .await
            .map_err(|e| reject::custom(e))?;

        Ok(warp::reply::json(&resp))
    }

    pub async fn register_answer(
        service: Arc<Mutex<BuzzService>>,
        game_info: Arc<Mutex<GameInfo>>,
        request: Requests,
    ) -> Result<impl warp::Reply, Rejection> {
        let resp = service
            .lock()
            .await
            .register_answer(request, game_info.clone())
            .await
            .map_err(|e| reject::custom(e))?;

        let g_i = game_info.clone();

        tokio::spawn(async move {
            // send updated score
            if let Response::ScoreUpdated(score) = resp {
                g_i.lock().await.send(score, false).await;
            }
            //send next question
            g_i.lock().await.next_question().await;
        });

        Ok(warp::reply::json(&Response::AnswerRegistered))
    }
}
