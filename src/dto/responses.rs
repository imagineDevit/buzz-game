use serde::Serialize;

/// ##Response representation
///
/// Response can be of 4 types:
///
///  * Error
///
///    __message__ : error message
///  * GameStarted
///  * PlayerAdded
///  * BuzzRegistered
///  * AnswerRegistered
#[derive(Serialize, Debug)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Response {
    Error { message: String, code: u16 },
    GameStarted,
    PlayerAdded,
    BuzzRegistered,
    AnswerRegistered,
}
