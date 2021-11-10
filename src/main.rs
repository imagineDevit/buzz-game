#![allow(dead_code)]

use std::collections::HashSet;

use rand::seq::SliceRandom;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_stream::StreamExt;

use data::db::*;

use crate::config::app::init_config;
use crate::data::repositories::PlayerRepository;
use crate::dto::messages::{Answer, Messages};
use crate::dto::states::StateChange;
use crate::errors::error::CustomError;
use crate::event::emitters::EventEmitters;
use crate::game_info::GameInfo;

mod config;
mod data;
mod dto;
mod errors;
mod event;
mod game_info;
mod utils;

#[tokio::main]
async fn main() -> Result<(), CustomError> {
    let config = init_config().await?;

    let db_pool = create_db_pool(&config)?;

    let pool = db_pool.clone();

    tokio::spawn(async move {
        let connection = get_connection(&pool).await.unwrap();
        let _ = data::db::init_db(&connection).await.unwrap();
    })
    .await
    .unwrap();

    let _repository = PlayerRepository::new(db_pool.clone());

    let _game_info = GameInfo::default();

    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<StateChange>();

    let _emitters = EventEmitters {
        state_changes: tx,
        question_iterator: list_of_questions().iter(),
    };

    let _stream = UnboundedReceiverStream::new(rx);

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
