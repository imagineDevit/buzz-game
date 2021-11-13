use crate::Messages;

#[derive(Debug)]
pub enum InternalEvent {
    PlayerAdded {
        player_name: String,
        score: Messages,
    },
    BuzzRegistered(String),
    AnswerRegistered {
        answer_number: u8,
        player_name: String,
    },
}
