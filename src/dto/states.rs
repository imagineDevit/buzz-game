use serde::Serialize;

use crate::dto::messages::Messages;

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum StateChangeType {
    GameStart,
    GameEnd,
    CanBuzz,
    NewPlayerScore,
    NewQuestion,
    NewBuzz,
    NewAnswer,
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
    change_type: StateChangeType,
    can_buzz: bool,
    #[serde(skip_serializing_if = "Messages::is_none")]
    message: Messages,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    players: Vec<String>,
    required_nb_players: u8,
}

impl StateChange {
    pub fn start() -> Self {
        Self {
            change_type: StateChangeType::GameStart,
            can_buzz: false,
            message: Messages::None,
            players: vec![],
            required_nb_players: 0,
        }
    }

    pub fn end() -> Self {
        Self {
            change_type: StateChangeType::GameEnd,
            can_buzz: false,
            message: Messages::None,
            players: vec![],
            required_nb_players: 0,
        }
    }

    pub fn with_can_buzz(can_buzz: bool) -> Self {
        Self {
            change_type: StateChangeType::CanBuzz,
            can_buzz,
            message: Messages::None,
            players: vec![],
            required_nb_players: 0,
        }
    }

    pub fn with_score(score: Messages, players: Vec<String>, required_nb_players: u8) -> Self {
        Self {
            change_type: StateChangeType::NewPlayerScore,
            can_buzz: false,
            message: score,
            players,
            required_nb_players,
        }
    }

    pub fn with_question(question: Messages) -> Self {
        Self {
            change_type: StateChangeType::NewQuestion,
            can_buzz: false,
            message: question,
            players: vec![],
            required_nb_players: 0,
        }
    }

    pub fn with_buzz(buzz: Messages) -> Self {
        Self {
            change_type: StateChangeType::NewBuzz,
            can_buzz: false,
            message: buzz,
            players: vec![],
            required_nb_players: 0,
        }
    }

    pub fn with_answer(answer: Messages) -> Self {
        Self {
            change_type: StateChangeType::NewAnswer,
            can_buzz: false,
            message: answer,
            players: vec![],
            required_nb_players: 0,
        }
    }
}
