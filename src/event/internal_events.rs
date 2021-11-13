use crate::Messages;

#[derive(Debug)]
pub enum InternalEvent {
    PlayerAdded(Messages),
    BuzzRegistered(String),
    AnswerRegistered {
        answer_number: u8,
        player_name: String,
    },
}
