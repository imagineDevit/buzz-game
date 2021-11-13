use tokio::sync::mpsc::UnboundedSender;

use crate::data::entities::Player;
use crate::dto::requests::Requests;
use crate::dto::responses::Response;
use crate::event::internal_events::InternalEvent;
use crate::{CustomError, Messages, PlayerRepository};

/// ##Buzz Service : Gestionnaire de la logique metier
///
/// __repostory__ : player repository
///
/// __tx__ : internal events sender
pub struct BuzzService {
    pub repository: PlayerRepository,
    pub tx: UnboundedSender<InternalEvent>,
}

impl BuzzService {
    /// ##Add new player
    ///
    /// __request__ : AddPlayer request
    pub async fn add_player(&mut self, request: Requests) -> Result<Response, CustomError> {
        // if request is as expected
        return if let Requests::AddPlayer { name } = request {
            // save new entity into db
            self.repository
                .insert(&Player::with_name(name.clone()))
                .await?;

            // send a PlayerAdded internal event
            self.tx
                .send(InternalEvent::PlayerAdded(Messages::PlayerScore {
                    player_name: name.clone(),
                    score: 0,
                    good_answer: String::default(),
                    update: false,
                }))?;

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
            })
        };
    }

    /// ## Register Buzz
    ///
    /// __request__ : RegisterBuzz request
    pub fn register_buzz(&self, request: Requests) -> Result<Response, CustomError> {
        // if request type is as expected
        return if let Requests::RegisterBuzz { player_name } = request {
            // send an internal  BuzzRegsitered event
            self.tx.send(InternalEvent::BuzzRegistered(player_name))?;

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
            })
        };
    }

    /// ## Register answer
    ///
    /// __request__ : RegisterAnswer request
    pub fn register_answer(&self, request: Requests) -> Result<Response, CustomError> {
        // if request has appropriate type
        return if let Requests::RegisterAnswer {
            answer_number,
            player_name,
            ..
        } = request
        {
            // send Answer registered internal Event
            self.tx.send(InternalEvent::AnswerRegistered {
                answer_number,
                player_name,
            })?;

            // return Aswer regsiteerd response
            Ok(Response::AnswerRegistered)
        } else {
            //  if request type is not as expected
            // return Error response
            Ok(Response::Error {
                message: format!(
                    "Bad request : expected : Requests::RegisterAnswer , found : {:?}",
                    request
                ),
            })
        };
    }
}
