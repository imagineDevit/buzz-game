use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::ops::Range;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::Mutex;

use crate::dto::messages::{Answer, Messages};
use crate::StateChange;

#[derive(Debug)]
pub struct GameInfo {
    pub started: AtomicBool,
    pub buzzed: AtomicBool,
    pub buzz_author: Option<String>,
    pub max_players: u8,
    pub min_players: u8,
    pub number_of_players: AtomicU8,
    pub current_question: Option<(Messages, Answer)>,
    pub senders: Arc<Mutex<HashMap<String, UnboundedSender<StateChange>>>>,
    pub questions: Vec<Messages>,
    pub already_called: Vec<usize>,
}

impl GameInfo {
    pub fn new(questions: Vec<Messages>) -> Self {
        Self {
            started: AtomicBool::new(false),
            buzzed: AtomicBool::new(false),
            buzz_author: None,
            max_players: 6,
            min_players: 3,
            number_of_players: AtomicU8::new(0),
            current_question: None,
            senders: Arc::new(Mutex::new(HashMap::new())),
            already_called: vec![],
            questions,
        }
    }

    pub async fn start(&self) {
        if self.number_of_players.load(Ordering::Relaxed) >= self.min_players
            && !self.started.load(Ordering::Relaxed)
        {
            self.started.store(true, Ordering::Relaxed);
        }
    }

    pub async fn add_player(&mut self, name: String, tx: UnboundedSender<StateChange>) -> bool {
        let mut senders = self.senders.lock().await;

        if !self.started.load(Ordering::Relaxed) && senders.get(name.clone().as_str()).is_none() {
            senders.insert(name.clone(), tx);

            let nb = self.number_of_players.load(Ordering::Relaxed);

            self.number_of_players.store(nb + 1, Ordering::Relaxed);

            return if self.number_of_players.load(Ordering::Relaxed) >= self.min_players {
                self.start().await;
                true
            } else {
                false
            };
        }

        false
    }

    pub fn load_current_question(&mut self, question: Messages, answer: Answer) {
        match question.clone() {
            Messages::Question { .. } => {
                self.current_question = Some((question, answer));
            }
            _ => panic!("Try to store inappropriate message type as GameInfo current_question"),
        }
    }

    pub async fn add_buzz(&mut self, author: String) -> bool {
        if !self.buzzed.load(Ordering::Relaxed) {
            self.buzzed.store(true, Ordering::Relaxed);
            self.buzz_author = Some(author);
            self.send(Messages::CanBuzz { can_buzz: false }, false)
                .await;
            return true;
        }
        false
    }

    pub async fn release_buzz(&mut self) {
        self.buzzed.store(false, Ordering::Relaxed);
        self.buzz_author = None;
        self.send(Messages::CanBuzz { can_buzz: true }, false).await;
    }

    pub async fn send(&self, message: Messages, start: bool) {
        let senders = self.senders.lock().await;
        let players = senders
            .keys()
            .into_iter()
            .map(|s| String::from(s.as_str()))
            .collect::<Vec<String>>();

        senders.clone().iter().for_each(|(_name, tx)| {
            let s = match message.clone() {
                Messages::Question { .. } => StateChange::with_question(message.clone()),
                Messages::PlayerAnswer { .. } => StateChange::with_answer(message.clone()),
                Messages::Buzz { .. } => StateChange::with_buzz(message.clone()),
                Messages::PlayerScore { .. } => {
                    StateChange::with_score(message.clone(), players.clone(), self.min_players)
                }
                Messages::CanBuzz { can_buzz } => StateChange::with_can_buzz(can_buzz),
                Messages::Error { message } => StateChange::with_error(message.clone()),
                Messages::None => {
                    return if start {
                        StateChange::start();
                    } else {
                        StateChange::end();
                    }
                }
            };

            tx.send(s).unwrap();
        });
    }

    pub async fn next_question(&mut self) {
        let qs = self.questions.clone();

        let mut i = 100;

        while i > qs.len() || self.already_called.contains(&i) {
            i = thread_rng().gen_range(Range {
                start: 0,
                end: self.questions.len(),
            });
        }

        let qst = qs.get(i);

        return if let Some(Messages::Question {
            number,
            label,
            points,
            answers,
            ..
        }) = qst
        {
            let q = Messages::Question {
                number: *number,
                label: String::from(label.as_str()),
                points: *points,
                answers: answers.clone(),
            };

            self.release_buzz().await;

            self.send(q.clone(), false).await;

            if let Some(g_answer) = answers.into_iter().filter(|a| a.good).next() {
                self.load_current_question(q, g_answer.clone());
            }
        } else {
            self.send(Messages::None, false).await;
        };
    }

    pub fn check_answer_author(&self, answer_author: String) -> bool {
        return if let Some(buzzer) = self.buzz_author.clone() {
            return if buzzer.eq(&answer_author) {
                true
            } else {
                false
            };
        } else {
            false
        };
    }
}

#[cfg(test)]
mod game_info_tests {
    use crate::dto::messages::{Answer, Messages};
    use crate::game_info::GameInfo;
    use crate::StateChange;
    use rstest::*;
    use std::collections::HashSet;
    use std::sync::atomic::Ordering;

    #[fixture]
    fn default_game_info() -> GameInfo {
        GameInfo::new(vec![])
    }

    #[rstest(default_game_info as info)]
    async fn start_test(info: GameInfo) {
        let mut info = info;

        let names = vec!["a", "b", "c"];

        for name in names {
            assert_eq!(false, info.started.load(Ordering::Relaxed));
            let (tx, _) = tokio::sync::mpsc::unbounded_channel::<StateChange>();
            info.add_player(String::from(name), tx).await;
            info.start().await;
        }

        assert_eq!(true, info.started.load(Ordering::Relaxed));
    }

    #[rstest(default_game_info as info)]
    fn load_current_question_test(info: GameInfo) {
        let mut info = info;

        let question = Messages::Question {
            number: 0,
            label: "".to_string(),
            points: 0,
            answers: HashSet::new(),
        };

        let answer = Answer {
            number: 0,
            label: "".to_string(),
            good: false,
        };

        info.load_current_question(question.clone(), answer.clone());

        assert!(info.current_question.is_some())
    }

    #[rstest(default_game_info as info)]
    async fn add_release_buzz_test(mut info: GameInfo) {
        assert_eq!(false, info.buzzed.load(Ordering::Relaxed));

        let added = info.add_buzz("Joe".to_string()).await;
        assert_eq!(true, info.buzzed.load(Ordering::Relaxed));
        assert_eq!(true, added);

        let added = info.add_buzz("Joe".to_string()).await;
        assert_eq!(true, info.buzzed.load(Ordering::Relaxed));
        assert_eq!(false, added);

        info.release_buzz().await;
        assert_eq!(false, info.buzzed.load(Ordering::Relaxed));
    }
}
