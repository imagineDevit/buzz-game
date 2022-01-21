use serde::Serialize;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

/// ##Player answer representation
///
/// __number__ : the answer number
///
/// __label__ : the answer label
///
/// __good__ : if answer is the good one or not
#[derive(Serialize, Debug, Eq, Clone)]
pub struct Answer {
    pub number: u8,
    pub label: String,
    pub good: bool,
}

impl PartialOrd for Answer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.number.cmp(&other.number))
    }
}

impl Hash for Answer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(self.label.as_bytes());
        state.finish();
    }
}

impl PartialEq for Answer {
    fn eq(&self, other: &Self) -> bool {
        self.label.eq(&other.label)
    }
}

/// Messages representation
///
/// Messages can be of type :
///
/// * Question
///     __number__ : question number
///
///     __label__ : question label
///
///     __points__ : number of points given by the question
///
///     __answers__ : list of possible answers
///
/// * PlayerAnswer
///
///     __player_name__ : author of the answer
///
///     __answer__ : answer label
///
/// * Buzz
///
///     __author__ : the buzz authors name
///
/// * PlayerScore
///
///     __player_name__ : the player name
///
///     __score__ : player new score
///
///     __good_answer__ : true if the score has been updated due to a good answer
///
///     __update__ : true if the score has been updated
///
///  * CanBuzz,
///
///  * None
#[derive(Serialize, Debug, PartialEq, Clone, Eq)]
#[serde(untagged)]
pub enum Messages {
    Question {
        number: u8,
        label: String,
        points: u32,
        answers: HashSet<Answer>,
    },

    #[serde(rename_all = "camelCase")]
    PlayerAnswer {
        player_name: String,
        answer: Answer,
    },

    Buzz {
        author: String,
    },

    #[serde(rename_all = "camelCase")]
    PlayerScore {
        player_name: String,
        score: u32,
        good_answer: String,
        update: bool,
    },

    #[serde(rename_all = "camelCase")]
    CanBuzz {
        can_buzz: bool,
    },

    Error {
        message: String,
    },

    GameStart,

    None,
}

impl Messages {
    pub fn is_none(&self) -> bool {
        match self {
            Messages::None => true,
            _ => false,
        }
    }
}
