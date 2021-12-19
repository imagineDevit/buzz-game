use std::sync::atomic::Ordering;
use std::sync::Arc;

use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::Mutex;
use warp::http::StatusCode;

use crate::data::entities::Player;
use crate::data::repositories::SearchAttributes;
use crate::dto::requests::Requests;
use crate::dto::responses::Response;
use crate::utils::fn_utils::apply_with;
use crate::{Answer, CustomError, GameInfo, Messages, PlayerRepository, StateChange};

/// ## Buzz Service : Gestionnaire de la logique metier
///
/// __repostory__ : player repository
#[derive(Clone)]
pub struct BuzzService {
    pub repository: PlayerRepository,
}

impl BuzzService {
    /// ## Add new player
    ///
    /// __request__ : AddPlayer request
    ///
    /// __game_info__ : the shared game info
    ///
    /// __tx__ : event sender for the player to add
    pub async fn add_player(
        &mut self,
        request: Requests,
        game_info: Arc<Mutex<GameInfo>>,
        tx: UnboundedSender<StateChange>,
    ) -> Result<Response, CustomError> {
        let mut game_info = game_info.lock().await;

        // if game has not started yet
        return if !game_info.started.load(Ordering::Relaxed) {
            // if request is as expected
            return if let Requests::AddPlayer { name } = request {
                // save new entity into db
                self.repository
                    .insert(&Player::with_name(name.clone()))
                    .await?;

                // add player and its sender to game info
                let ready = game_info.add_player(name.clone(), tx).await;

                // send initial player score
                game_info
                    .send(Messages::PlayerScore {
                            player_name: name.clone(),
                            score: 0,
                            good_answer: String::default(),
                            update: false,
                        })
                    .await;

                // return PlayerAdded response
                Ok(Response::PlayerAdded(ready))
            } else {
                // if request type is not as expected
                crate::error!(
                    game -> game_info,
                    message -> format!("Bad request : expected : Requests::AddPlayer , found : {:?}", request),
                    status -> StatusCode::INTERNAL_SERVER_ERROR.as_u16()
                )
            };
        } else {
            // if game is already started
            crate::error!(
                game -> game_info,
                message -> "Game is already started".to_string(),
                status -> StatusCode::INTERNAL_SERVER_ERROR.as_u16()
            )
        };
    }

    /// ## Register Buzz
    ///
    /// __request__ : RegisterBuzz request
    ///
    /// __game_info__ : the shared game_info
    pub async fn register_buzz(
        &self,
        request: Requests,
        game_info: Arc<Mutex<GameInfo>>,
    ) -> Result<Response, CustomError> {
        let mut game_info = game_info.lock().await;

        // if buzz is released
        return if !game_info.buzzed.load(Ordering::Relaxed) {
            // if request type is as expected
            return if let Requests::RegisterBuzz { player_name } = request {
                // press buzz
                game_info.add_buzz(player_name.clone()).await;

                // send buzz message
                game_info
                    .send(Messages::Buzz {
                            author: player_name,
                        })
                    .await;

                // return BuzzRegistered response
                Ok(Response::BuzzRegistered)
            } else {
                // if request type is not as expected
                crate::error!(
                    game -> game_info,
                    message -> format!(
                            "Bad request : expected : Requests::RegisterBuzz , found : {:?}",
                            request
                        ),
                    status -> StatusCode::INTERNAL_SERVER_ERROR.as_u16()
                )
            };
        } else {
            // if buzz is already pressed
            crate::error!(
                game -> game_info,
                message -> "Someone else has already buzzed".to_string(),
                status -> StatusCode::INTERNAL_SERVER_ERROR.as_u16()
            )
        };
    }

    /// ## Register answer
    ///
    /// __request__ : RegisterAnswer request
    ///
    /// __game_info__ : the shared game_info
    pub async fn register_answer(
        &self,
        request: Requests,
        game_info: Arc<Mutex<GameInfo>>,
    ) -> Result<Response, CustomError> {
        let game_info = game_info.lock().await;

        // if request has appropriate type
        return if let Requests::RegisterAnswer {
            answer_number,
            player_name,
            question_number,
        } = request
        {
            // if the buzz and answer authors are the same
            return if game_info.check_answer_author(player_name.clone()) {
                return match game_info.current_question.clone() {
                    Some((
                        Messages::Question {
                            points,
                            number: q_num,
                            ..
                        },
                        Answer { number, label, .. },
                    )) => {
                        return if q_num == question_number {
                            apply_with(
                                || number == answer_number,
                                |good| async move {
                                    // send StateChange WithAnswer event
                                    game_info
                                        .send(Messages::PlayerAnswer {
                                                player_name: player_name.clone(),
                                                answer: Answer {
                                                    number: answer_number,
                                                    label: label.clone(),
                                                    good: good.clone(),
                                                },
                                            })
                                        .await;

                                    // if the given answer is the good one
                                    // update player score and load the resulted player
                                    match self
                                        .repository
                                        .find_by(SearchAttributes::Name(player_name.clone()))
                                        .await
                                    {
                                        Ok(Some(mut p)) => {
                                            if good {
                                                p = self
                                                    .repository
                                                    .update_score(
                                                        player_name.clone(),
                                                        p.score + points.clone(),
                                                    )
                                                    .await?;
                                            }

                                            // return Aswer regsiteerd response
                                            Ok(Response::ScoreUpdated(Messages::PlayerScore {
                                                player_name,
                                                score: p.score,
                                                good_answer: label,
                                                update: good,
                                            }))
                                        }
                                        Err(e) => {
                                            crate::error!(
                                                game -> game_info,
                                                message -> e.to_string(),
                                                status -> StatusCode::INTERNAL_SERVER_ERROR.as_u16()
                                            )
                                        }
                                        _ => {
                                            crate::error!(
                                                game -> game_info,
                                                message -> "Unexpected error occured".to_string(),
                                                status -> StatusCode::INTERNAL_SERVER_ERROR.as_u16()
                                            )
                                        }
                                    }
                                },
                            )
                            .await
                        } else {
                            crate::error!(
                                game -> game_info,
                                message -> format!(
                                    "Bad question number! Expected :{} ; found {}",
                                    q_num, question_number
                                ),
                                status -> StatusCode::BAD_REQUEST.as_u16()
                            )
                        };
                    }
                    _ => {
                        // if current question stored in game info is not as pexpected
                        crate::error!(
                            game -> game_info,
                            message -> format!("Inappropriate current question registered"),
                            status -> StatusCode::BAD_REQUEST.as_u16()
                        )
                    }
                };
            } else {
                // if the buzz and answer authors are not the same
                crate::error!(
                    game -> game_info,
                    message -> "Buzz author is different from answer author".to_string(),
                    status -> StatusCode::INTERNAL_SERVER_ERROR.as_u16()
                )
            };
        } else {
            //  if request type is not as expected
            crate::error!(
                game -> game_info,
                message -> format!(
                    "Bad request : expected : Requests::RegisterAnswer , found : {:?}",
                    request
                ),
                status -> StatusCode::INTERNAL_SERVER_ERROR.as_u16()
            )
        };
    }
}
