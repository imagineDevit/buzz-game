#![allow(dead_code)]

use std::collections::HashSet;
use std::sync::Arc;

use rand::seq::SliceRandom;
use tokio::sync::Mutex;
use warp::Filter;

use data::db::*;

use crate::config::app::init_config;
use crate::data::repositories::PlayerRepository;
use crate::dto::messages::{Answer, Messages};
use crate::dto::states::StateChange;
use crate::errors::error::CustomError;
use crate::game_info::GameInfo;
use crate::services::buzz_services::BuzzService;
use crate::web::routes::Routes;

mod config;
mod data;
mod dto;
mod errors;
mod game_info;
mod services;
mod utils;
mod web;

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
    });

    let game_info = Arc::new(Mutex::new(GameInfo::new(list_of_questions())));

    let service = Arc::new(Mutex::new(BuzzService {
        repository: PlayerRepository::new(db_pool.clone()),
    }));

    warp::serve(
        Routes::add_player(service.clone(), game_info.clone())
            .or(Routes::register_buzz(service.clone(), game_info.clone()))
            .or(Routes::register_answer(service.clone(), game_info.clone()))
            .with(
                warp::cors()
                    .allow_any_origin()
                    .allow_methods(vec!["POST", "GET", "OPTIONS"])
                    .allow_headers(vec![
                        "Accept",
                        "User-Agent",
                        "Sec-Fetch-Mode",
                        "Referer",
                        "Origin",
                        "Access-Control-Request-Method",
                        "Access-Control-Request-Headers",
                        "Access-Control-Allow-Origin",
                        "Content-type",
                    ]),
            )
            .recover(crate::web::exception_handlers::handle_error),
    )
    .run(([127, 0, 0, 1], 3030))
    .await;

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
