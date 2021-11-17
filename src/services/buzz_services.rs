use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::Mutex;
use warp::http::StatusCode;

use crate::data::entities::Player;
use crate::data::repositories::SearchAttributes;
use crate::dto::requests::Requests;
use crate::dto::responses::Response;
use crate::utils::fn_utils::apply_with;
use crate::{Answer, CustomError, GameInfo, Messages, PlayerRepository, StateChange};

/// ##Buzz Service : Gestionnaire de la logique metier
///
/// __repostory__ : player repository
///
/// __tx__ : internal events sender

#[derive(Clone)]
pub struct BuzzService {
    pub repository: PlayerRepository,
}

impl BuzzService {
    /// ##Add new player
    ///
    /// __request__ : AddPlayer request
    pub async fn add_player(
        &mut self,
        request: Requests,
        game_info: Arc<Mutex<GameInfo>>,
        tx: UnboundedSender<StateChange>,
    ) -> Result<Response, CustomError> {
        let mut game_info = game_info.lock().await;

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
                    .send(
                        Messages::PlayerScore {
                            player_name: name.clone(),
                            score: 0,
                            good_answer: String::default(),
                            update: false,
                        },
                        false,
                    )
                    .await;

                // if game is ready to start
                if ready {
                    // start the game by sending starting message
                    game_info.send(Messages::None, true).await;

                    // send next question
                    game_info.next_question().await;
                }

                // return PlayerAdded response
                Ok(Response::PlayerAdded)
            } else {
                // if request type is not as expected
                // return Error response
                Ok(Response::Error {
                    message: format!(
                        "Bad request : expected : Requests::AddPlayer , found : {:?}",
                        request
                    ),
                    code: 500,
                })
            };
        } else {
            Ok(Response::Error {
                message: "Game is already started".to_string(),
                code: 500,
            })
        };
    }

    /// ## Register Buzz
    ///
    /// __request__ : RegisterBuzz request
    pub async fn register_buzz(
        &self,
        request: Requests,
        game_info: Arc<Mutex<GameInfo>>,
    ) -> Result<Response, CustomError> {
        let mut game_info = game_info.lock().await;

        return if !game_info.buzzed.load(Ordering::Relaxed) {
            // if request type is as expected
            return if let Requests::RegisterBuzz { player_name } = request {
                game_info.add_buzz(player_name.clone()).await;

                // send buzz message
                game_info
                    .send(
                        Messages::Buzz {
                            author: player_name,
                        },
                        false,
                    )
                    .await;

                // return BuzzRegistered response
                Ok(Response::BuzzRegistered)
            } else {
                // if request type is not as expected
                // return Error response
                Ok(Response::Error {
                    message: format!(
                        "Bad request : expected : Requests::RegisterBuzz , found : {:?}",
                        request
                    ),
                    code: 500,
                })
            };
        } else {
            Ok(Response::Error {
                message: "Someone else has already buzzed".to_string(),
                code: 500,
            })
        };
    }

    /// ## Register answer
    ///
    /// __request__ : RegisterAnswer request
    pub async fn register_answer(
        &self,
        request: Requests,
        game_info: Arc<Mutex<GameInfo>>,
    ) -> Result<Response, CustomError> {
        // if request has appropriate type
        return if let Requests::RegisterAnswer {
            answer_number,
            player_name,
            question_number,
        } = request
        {
            let mut game_info = game_info.lock().await;

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
                        if q_num != question_number {
                            let error_msg = format!(
                                "Bad question number! Expected :{} ; found {}",
                                q_num, question_number
                            );
                            game_info
                                .send(
                                    Messages::Error {
                                        message: error_msg.clone(),
                                    },
                                    false,
                                )
                                .await;
                            return Ok(Response::Error {
                                message: error_msg,
                                code: StatusCode::BAD_REQUEST.as_u16(),
                            });
                        }

                        apply_with(
                            || number == answer_number,
                            |good| async move {
                                // send StateChange WithAnswer event
                                game_info
                                    .send(
                                        Messages::PlayerAnswer {
                                            player_name: player_name.clone(),
                                            answer: Answer {
                                                number: answer_number,
                                                label: label.clone(),
                                                good: good.clone(),
                                            },
                                        },
                                        false,
                                    )
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

                                        game_info
                                            .send(
                                                Messages::PlayerScore {
                                                    player_name,
                                                    score: p.score,
                                                    good_answer: label,
                                                    update: good,
                                                },
                                                false,
                                            )
                                            .await;
                                    }
                                    _ => {}
                                }

                                // After 1 second, send try to next question
                                std::thread::sleep(Duration::from_secs(1));

                                game_info.next_question().await;

                                Ok::<(), CustomError>(())
                            },
                        )
                        .await?;

                        // return Aswer regsiteerd response
                        Ok(Response::AnswerRegistered)
                    }
                    _ => {
                        let error_msg = format!("Inappropriate current question registered");
                        game_info
                            .send(
                                Messages::Error {
                                    message: error_msg.clone(),
                                },
                                false,
                            )
                            .await;
                        // return Aswer regsiteerd response
                        Ok(Response::Error {
                            message: error_msg,
                            code: StatusCode::BAD_REQUEST.as_u16(),
                        })
                    }
                };
            } else {
                game_info
                    .send(
                        Messages::Error {
                            message: "Buzz author is different from answer author".to_string(),
                        },
                        false,
                    )
                    .await;
                Ok(Response::Error {
                    message: format!("Buzz author is different from answer author"),
                    code: StatusCode::BAD_REQUEST.as_u16(),
                })
            };
        } else {
            //  if request type is not as expected
            // return Error response
            Ok(Response::Error {
                message: format!(
                    "Bad request : expected : Requests::RegisterAnswer , found : {:?}",
                    request
                ),
                code: 500,
            })
        };
    }
}
