use std::future::Future;
use std::slice::Iter;
use std::time::Duration;

use tokio::sync::mpsc::UnboundedSender;

use crate::data::entities::Player;
use crate::dto::messages::{Answer, Messages};
use crate::dto::requests::Requests;
use crate::dto::states::StateChange;
use crate::errors::error::CustomError;
use crate::game_info::GameInfo;
use crate::utils::fn_utils::apply_with;

pub struct EventEmitters<'a> {
    pub state_changes: UnboundedSender<StateChange>,
    pub question_iterator: Iter<'a, Messages>,
}

impl<'a> EventEmitters<'a> {
    
    pub fn on_game_started(&mut self, game_info: &mut GameInfo) -> Result<(), CustomError> {
        game_info.start();
        self.state_changes.send(StateChange::start())?;
        self.next_question(game_info);
        Ok(())
    }

    pub fn emit_score(
        &self,
        player: Player,
        good_answer: String,
        update: bool,
        players: Vec<String>,
        nb_min_plapers: u8,
    ) -> Result<(), CustomError> {
        let score = Messages::PlayerScore {
            player_name: player.name,
            score: player.score,
            good_answer,
            update,
        };

        self.state_changes
            .send(StateChange::with_score(score, players, nb_min_plapers))?;

        Ok(())
    }

    pub fn on_buzz_registered(&self, buzz: Requests) -> Result<(), CustomError> {
        if let Requests::RegisterBuzz { player_name } = buzz {
            self.disable_buzz();
            self.state_changes
                .send(StateChange::with_buzz(Messages::Buzz {
                    author: player_name,
                }))?;
            return Ok(());
        }

        Err(CustomError::BadRequestTypeError {
            message: format!(
                "Bad request! Expected: Requests::RegisterBuzz  , Found: {:?}",
                buzz
            ),
        })
    }

    pub async fn on_answer_registered<F: Future<Output = Result<Player, CustomError>>>(
        &mut self,
        answer: Requests,
        player: Player,
        update_score: impl FnOnce((Player, u32)) -> F,
        game_info: &mut GameInfo,
    ) -> Result<(), CustomError> {
        match answer {
            Requests::RegisterAnswer { answer_number, .. } => {
                match game_info.current_question.clone() {
                    Some((Messages::Question { points, .. }, Answer { number, label, .. })) => {
                        apply_with(
                            || number == answer_number,
                            |good| async move {
                                self.state_changes.send(StateChange::with_answer(
                                    Messages::PlayerAnswer {
                                        player_name: player.name.clone(),
                                        answer: Answer {
                                            number,
                                            label: label.clone(),
                                            good,
                                        },
                                    },
                                ))?;

                                let p = if good {
                                    update_score((player, points)).await?
                                } else {
                                    player
                                };

                                self.emit_score(p, label, good, vec![], game_info.min_players)?;

                                std::thread::sleep(Duration::from_secs(1));
                                self.next_question(game_info);

                                Ok::<(), CustomError>(())
                            },
                        )
                        .await
                    }
                    _ => Err(CustomError::BadRequestTypeError {
                        message: format!(
                            "Bad request! Expected: Request::RegisterAnswer , Found: {:?}",
                            answer
                        ),
                    }),
                }
            }
            _ => Err(CustomError::BadRequestTypeError {
                message: format!(
                    "Bad request! Expected: Request::RegisterAnswer , Found: {:?}",
                    answer
                ),
            }),
        }
    }

    fn next_question(&mut self, game_info: &mut GameInfo) {
        if let Some(Messages::Question {
            number,
            label,
            points,
            answers,
            ..
        }) = self.question_iterator.next()
        {
            let q = Messages::Question {
                number: *number,
                label: String::from(label.as_str()),
                points: *points,
                answers: answers.clone(),
            };

            self.enable_buzz();

            self.state_changes
                .send(StateChange::with_question(q.clone()))
                .unwrap();

            if let Some(g_answer) = answers.into_iter().filter(|a| a.good).next() {
                game_info.load_current_question(q.clone(), g_answer.clone());
            }
        } else {
            self.state_changes.send(StateChange::end()).unwrap();
        }
    }

    fn enable_buzz(&self) {
        self.state_changes
            .send(StateChange::with_can_buzz(true))
            .unwrap();
    }

    fn disable_buzz(&self) {
        self.state_changes
            .send(StateChange::with_can_buzz(false))
            .unwrap();
    }
}
