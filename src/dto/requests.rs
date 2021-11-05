use serde::Deserialize;

/// ##Request representation
///
/// Request can be of 3 types:
///
///  * AddPlayer
///
///     __name__ : name of the player to add
///
///  * RegisterBuzz
///
///     __player_name__ : name of player who has buzzed
///
///  * RegisterAnswer
///
///     __player_name__ : name of player who answered
///
///     __question_number__ : number of the current question
///
///     __answer_number__ : number of the answer
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Requests {
    #[serde(rename_all = "camelCase")]
    RegisterAnswer {
        player_name: String,
        question_number: u8,
        answer_number: u8,
    },

    AddPlayer {
        name: String,
    },

    #[serde(rename_all = "camelCase")]
    RegisterBuzz {
        player_name: String,
    },
}
