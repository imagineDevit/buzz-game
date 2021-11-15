use std::future::Future;
use std::slice::Iter;
use std::time::Duration;

use tokio::sync::mpsc::UnboundedSender;

use crate::data::entities::Player;
use crate::dto::messages::{Answer, Messages};
use crate::dto::states::{StateChange, StateChangeWrapper};
use crate::errors::error::CustomError;
use crate::game_info::GameInfo;
use crate::utils::fn_utils::apply_with;

/// ## StateChange events emitters
///
/// __state_changes__ : event sender
///
/// __question_iterator__ : question list iterator
pub struct EventEmitters<'a> {
    pub state_changes: UnboundedSender<StateChangeWrapper<'a>>,
    pub question_iterator: Iter<'a, Messages>,
}

impl EventEmitters<'static> {
    /// ## On game started
    ///
    /// __game_info__ : shared game info
    pub async fn on_game_started(&mut self, game_info: &mut GameInfo) -> Result<(), CustomError> {
        // send StateChange Start event

        self.state_changes.send(StateChange::start())?;

        // try to find and emit next question
        self.next_question(game_info).await;

        // return empty response
        Ok(())
    }

    /// ## Emit mew player score
    ///
    /// __score__ : score to emit
    ///
    /// __players__ : list of player's names
    ///
    /// __nb_min__players__ : minimal numbers of players
    pub fn emit_score(
        &self,
        score: Messages,
        players: Vec<String>,
        nb_min_plapers: u8,
    ) -> Result<(), CustomError> {
        // send StateChange WithScore event
        self.state_changes
            .send(StateChange::with_score(score, players, nb_min_plapers))?;
        Ok(())
    }

    /// ## On Buzz registered
    ///
    /// __player_name__ : buzz author's name
    ///
    /// __game_info__ : shared game info
    pub async fn on_buzz_registered(
        &self,
        player_name: String,
        game_info: &mut GameInfo,
    ) -> Result<(), CustomError> {
        if game_info.add_buzz().await {
            // emit StateChange WithCanBuzz event
            self.disable_buzz();

            // send StateChange WithBuzz evet
            self.state_changes
                .send(StateChange::with_buzz(Messages::Buzz {
                    author: player_name,
                }))?;
        }

        Ok(())
    }

    /// ## On Anwer Registered
    ///
    /// __answer_number__ : number of answer selected by the anwser author
    ///
    /// __player__ : answer author
    ///
    /// __update_score__ : action to apply to update the player score
    ///
    /// __game_info__ : shared game info
    pub async fn on_answer_registered<F: Future<Output = Result<Player, CustomError>>>(
        &mut self,
        answer_number: u8,
        player: Player,
        update_score: impl FnOnce((String, u32)) -> F,
        game_info: &mut GameInfo,
    ) -> Result<(), CustomError> {
        //if game info has appropriate current question registered
        match game_info.current_question.clone() {
            Some((Messages::Question { points, .. }, Answer { number, label, .. })) => {
                apply_with(
                    || number == answer_number,
                    |good| async move {
                        // send StateChange WithAnswer event
                        self.state_changes.send(StateChange::with_answer(
                            Messages::PlayerAnswer {
                                player_name: player.name.clone(),
                                answer: Answer {
                                    number: answer_number,
                                    label: label.clone(),
                                    good,
                                },
                            },
                        ))?;

                        // if the given answer is the good one
                        // update player score and load the resulted player
                        let p = if good {
                            update_score((player.name, points)).await?
                        } else {
                            player
                        };

                        // Emit new player score
                        self.emit_score(
                            Messages::PlayerScore {
                                player_name: p.name,
                                score: p.score,
                                good_answer: label,
                                update: good,
                            },
                            game_info
                                .players
                                .clone()
                                .into_iter()
                                .collect::<Vec<String>>(),
                            game_info.min_players,
                        )?;

                        // After 1 second, send try to next question
                        std::thread::sleep(Duration::from_secs(1));
                        self.next_question(game_info).await;

                        Ok::<(), CustomError>(())
                    },
                )
                .await
            }
            _ => {
                self.state_changes.send(StateChange::with_error(
                    "Inappropriate current question registered".to_string(),
                ))?;

                Ok::<(), CustomError>(())
            }
        }
    }

    async fn next_question(&mut self, game_info: &mut GameInfo) {
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

            self.enable_buzz(game_info).await;

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

    async fn enable_buzz(&self, game_info: &mut GameInfo) {
        game_info.release_buzz().await;

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
