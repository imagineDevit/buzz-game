use serde::Serialize;

use crate::dto::messages::Messages;

#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StateChangeType {
    GameStart,
    GameEnd,
    CanBuzz,
    NewPlayerScore,
    NewQuestion,
    NewBuzz,
    NewAnswer,
    Error,
}

/// ##State representation
///
///   __change_type__ : the state change type
///
///   __can_buzz__ : true if the players can buzz
///
///   __message__ : the sending message
///
///   __players__ : list of players names
///
///   __required_nb_players__ : the required number of player
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StateChange {
    #[serde(rename = "type")]
    pub change_type: StateChangeType,
    //#[serde(skip_serializing_if = "Messages::is_none")]
    pub message: Messages,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub players: Vec<String>,
    pub required_nb_players: u8,
}

impl StateChange {
    pub fn start(players: Vec<String>, required_nb_players: u8) -> Self {
        Self {
            change_type: StateChangeType::GameStart,
            message: Messages::GameStart,
            players,
            required_nb_players,
        }
    }

    pub fn end() -> Self {
        Self {
            change_type: StateChangeType::GameEnd,
            message: Messages::None,
            players: vec![],
            required_nb_players: 0,
        }
    }

    pub fn with_can_buzz(can_buzz: bool) -> Self {
        Self {
            change_type: StateChangeType::CanBuzz,
            message: Messages::CanBuzz { can_buzz },
            players: vec![],
            required_nb_players: 0,
        }
    }

    pub fn with_score(score: Messages, players: Vec<String>, required_nb_players: u8) -> Self {
        Self {
            change_type: StateChangeType::NewPlayerScore,
            message: score,
            players,
            required_nb_players,
        }
    }

    pub fn with_question(question: Messages) -> Self {
        Self {
            change_type: StateChangeType::NewQuestion,
            message: question,
            players: vec![],
            required_nb_players: 0,
        }
    }

    pub fn with_buzz(buzz: Messages) -> Self {
        Self {
            change_type: StateChangeType::NewBuzz,
            message: buzz,
            players: vec![],
            required_nb_players: 0,
        }
    }

    pub fn with_answer(answer: Messages) -> Self {
        Self {
            change_type: StateChangeType::NewAnswer,
            message: answer,
            players: vec![],
            required_nb_players: 0,
        }
    }

    pub fn with_error(msg: String) -> Self {
        Self {
            change_type: StateChangeType::Error,
            message: Messages::Error { message: msg },
            players: vec![],
            required_nb_players: 0,
        }
    }
}
