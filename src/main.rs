#![allow(dead_code)]

use std::collections::HashSet;
use std::sync::atomic::Ordering;

use rand::seq::SliceRandom;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_stream::StreamExt;

use data::db::*;

use crate::config::app::init_config;
use crate::data::repositories::{PlayerRepository, SearchAttributes};
use crate::dto::messages::{Answer, Messages};
use crate::dto::states::StateChange;
use crate::errors::error::CustomError;
use crate::event::emitters::EventEmitters;
use crate::event::internal_events::InternalEvent;
use crate::game_info::GameInfo;
use crate::services::buzz_services::BuzzService;

mod config;
mod data;
mod dto;
mod errors;
mod event;
mod game_info;
mod services;
mod utils;

#[tokio::main]
async fn main() -> Result<(), CustomError> {
    // Initialize application config
    let config = init_config().await?;

    // Initialize database
    let db_pool = create_db_pool(&config)?;

    let pool = db_pool.clone();

    tokio::spawn(async move {
        let connection = get_connection(&pool).await.unwrap();
        let _ = data::db::init_db(&connection).await.unwrap();
    })
    .await
    .unwrap();

    // Instantiate the game_info bean
    let mut game_info = GameInfo::default();

    // Instanciation of unbounded channels for events sending/receiving
    let (state_change_sender, state_change_receiver) =
        tokio::sync::mpsc::unbounded_channel::<StateChange>();

    let (internal_event_sender, internal_event_receiver) =
        tokio::sync::mpsc::unbounded_channel::<InternalEvent>();

    // Instanciation of application beans
    let repository = PlayerRepository::new(db_pool.clone());

    let _service = BuzzService {
        tx: internal_event_sender,
        repository: repository.clone(),
    };

    let _state_changes_stream = UnboundedReceiverStream::new(state_change_receiver);

    // handle internal events

    let repositry_clone = repository.clone();

    let mut internal_events_stream = UnboundedReceiverStream::new(internal_event_receiver);

    let started = game_info.started.clone();
    let buzzed = game_info.buzzed.clone();

    tokio::spawn(async move {
        let questions = list_of_questions();

        let mut emitters = EventEmitters {
            state_changes: state_change_sender,
            question_iterator: questions.iter(),
        };

        loop {
            while let Some(internal_event) = internal_events_stream.next().await {
                let p = game_info
                    .players
                    .clone()
                    .into_iter()
                    .collect::<Vec<String>>();

                match internal_event {
                    InternalEvent::PlayerAdded { player_name, score } => {
                        if !started.lock().await.load(Ordering::Relaxed) {
                            let ready = game_info.add_player(player_name).await;

                            emitters
                                .emit_score(score, p.clone(), game_info.min_players)
                                .unwrap();

                            if ready {
                                emitters.on_game_started(&mut game_info).await.unwrap();
                            }
                        }
                    }

                    InternalEvent::BuzzRegistered(player_name) => {
                        if !buzzed.lock().await.load(Ordering::Relaxed) {
                            emitters
                                .on_buzz_registered(player_name, &mut game_info)
                                .await
                                .unwrap();
                        }
                    }

                    InternalEvent::AnswerRegistered {
                        player_name,
                        answer_number,
                    } => {
                        match repositry_clone
                            .find_by(SearchAttributes::Name(player_name))
                            .await
                        {
                            Ok(Some(p)) => {
                                emitters
                                    .on_answer_registered(
                                        answer_number,
                                        p.clone(),
                                        |(name, points)| {
                                            repositry_clone.update_score(name, p.score + points)
                                        },
                                        &mut game_info,
                                    )
                                    .await
                                    .unwrap();
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    });

    Ok(())
}

fn list_of_questions() -> Vec<Messages> {
    let mut q = vec![
        Messages::Question {
            number: 0,
            label: "Lequel de ces noms ne correspond pas à un langage informatique ?".to_string(),
            points: 1,
            answers: HashSet::from([
                Answer {
                    number: 0,
                    label: "Elm".to_string(),
                    good: false,
                },
                Answer {
                    number: 1,
                    label: "Rust".to_string(),
                    good: false,
                },
                Answer {
                    number: 2,
                    label: "Dark".to_string(),
                    good: true,
                },
            ]),
        },
        Messages::Question {
            number: 1,
            label: "Dans le langage RUST quel mot clé est utilisé pour désigné une fonction ?"
                .to_string(),
            points: 2,
            answers: HashSet::from([
                Answer {
                    number: 0,
                    label: "fun".to_string(),
                    good: false,
                },
                Answer {
                    number: 1,
                    label: "fn".to_string(),
                    good: true,
                },
                Answer {
                    number: 2,
                    label: "func".to_string(),
                    good: false,
                },
            ]),
        },
        Messages::Question {
            number: 2,
            label: "Dans quelle version de java est apparu le mot clé 'default' ?".to_string(),
            points: 1,
            answers: HashSet::from([
                Answer {
                    number: 0,
                    label: "Java 8".to_string(),
                    good: true,
                },
                Answer {
                    number: 1,
                    label: "Java 11".to_string(),
                    good: false,
                },
                Answer {
                    number: 2,
                    label: "Java 14".to_string(),
                    good: false,
                },
            ]),
        },
        Messages::Question {
            number: 3,
            label: "Dans le langage RUST quel mot clé est utilisé pour définir une interface ?"
                .to_string(),
            points: 2,
            answers: HashSet::from([
                Answer {
                    number: 0,
                    label: "trait".to_string(),
                    good: true,
                },
                Answer {
                    number: 1,
                    label: "inter".to_string(),
                    good: false,
                },
                Answer {
                    number: 2,
                    label: "impl".to_string(),
                    good: false,
                },
            ]),
        },
        Messages::Question {
            number: 4,
            label: "Dans le sigle WASI, que signifie le SI ?".to_string(),
            points: 3,
            answers: HashSet::from([
                Answer {
                    number: 0,
                    label: "System Interface".to_string(),
                    good: true,
                },
                Answer {
                    number: 1,
                    label: "Social Information".to_string(),
                    good: false,
                },
                Answer {
                    number: 2,
                    label: "Systeme Informatique".to_string(),
                    good: false,
                },
            ]),
        },
        Messages::Question {
            number: 5,
            label: "En Kotlin quel mot clé est utilisé pour une fonction asynchrone ?".to_string(),
            points: 2,
            answers: HashSet::from([
                Answer {
                    number: 0,
                    label: "async".to_string(),
                    good: false,
                },
                Answer {
                    number: 1,
                    label: "await".to_string(),
                    good: false,
                },
                Answer {
                    number: 2,
                    label: "suspend".to_string(),
                    good: true,
                },
            ]),
        },
        Messages::Question {
            number: 6,
            label: "Dans le language GO, quel mot clé permet de créer un thread ?".to_string(),
            points: 3,
            answers: HashSet::from([
                Answer {
                    number: 0,
                    label: "go".to_string(),
                    good: true,
                },
                Answer {
                    number: 1,
                    label: "thread".to_string(),
                    good: false,
                },
                Answer {
                    number: 2,
                    label: "th".to_string(),
                    good: false,
                },
            ]),
        },
        Messages::Question {
            number: 7,
            label: "Dans quel lanquage retrouve t-on le mot clé 'defer' ?".to_string(),
            points: 2,
            answers: HashSet::from([
                Answer {
                    number: 0,
                    label: "Python".to_string(),
                    good: false,
                },
                Answer {
                    number: 1,
                    label: "Go".to_string(),
                    good: true,
                },
                Answer {
                    number: 2,
                    label: "Java".to_string(),
                    good: false,
                },
            ]),
        },
    ];

    q.shuffle(&mut rand::thread_rng());

    q
}
