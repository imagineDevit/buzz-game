use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::dto::messages::{Answer, Messages};

#[derive(Debug)]
pub struct GameInfo {
    pub started: Arc<Mutex<AtomicBool>>,
    pub buzzed: Arc<Mutex<AtomicBool>>,
    pub max_players: u8,
    pub min_players: u8,
    pub number_of_players: AtomicU8,
    pub current_question: Option<(Messages, Answer)>,
    pub players: HashSet<String>,
}

impl Default for GameInfo {
    fn default() -> Self {
        Self {
            started: Arc::new(Mutex::new(AtomicBool::new(false))),
            buzzed: Arc::new(Mutex::new(AtomicBool::new(false))),
            max_players: 6,
            min_players: 3,
            number_of_players: AtomicU8::new(0),
            current_question: None,
            players: HashSet::new(),
        }
    }
}

impl GameInfo {
    pub async fn start(&self) {
        if self.number_of_players.load(Ordering::Relaxed) >= self.min_players
            && !self.started.lock().await.load(Ordering::Relaxed)
        {
            self.started.lock().await.store(true, Ordering::Relaxed);
        }
    }

    pub async fn add_player(&mut self, name: String) -> bool {
        if !self.started.lock().await.load(Ordering::Relaxed) && !self.players.contains(&name) {
            self.players.insert(name);
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

    pub async fn add_buzz(&self) -> bool {
        if !self.buzzed.lock().await.load(Ordering::Relaxed) {
            self.buzzed.lock().await.store(true, Ordering::Relaxed);
            return true;
        }
        false
    }

    pub async fn release_buzz(&self) {
        self.buzzed.lock().await.store(false, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod game_info_tests {
    use crate::dto::messages::{Answer, Messages};
    use crate::game_info::GameInfo;
    use rstest::*;
    use std::collections::HashSet;
    use std::sync::atomic::Ordering;

    #[fixture]
    fn default_game_info() -> GameInfo {
        GameInfo::default()
    }

    #[rstest(default_game_info as info)]
    async fn start_test(info: GameInfo) {
        let mut info = info;

        let names = vec!["a", "b", "c"];

        for name in names {
            assert_eq!(false, info.started.lock().await.load(Ordering::Relaxed));
            info.add_player(String::from(name)).await;
            info.start().await;
        }

        assert_eq!(true, info.started.lock().await.load(Ordering::Relaxed));
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
    async fn add_release_buzz_test(info: GameInfo) {
        assert_eq!(false, info.buzzed.lock().await.load(Ordering::Relaxed));

        let added = info.add_buzz().await;
        assert_eq!(true, info.buzzed.lock().await.load(Ordering::Relaxed));
        assert_eq!(true, added);

        let added = info.add_buzz().await;
        assert_eq!(true, info.buzzed.lock().await.load(Ordering::Relaxed));
        assert_eq!(false, added);

        info.release_buzz().await;
        assert_eq!(false, info.buzzed.lock().await.load(Ordering::Relaxed));
    }
}
