#![allow(dead_code)]

use std::collections::HashSet;
use std::convert::Infallible;
use std::ops::Deref;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use futures_util::Stream;
use rand::seq::SliceRandom;
use tokio::sync::Mutex;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_stream::StreamExt;
use warp::sse::Event;
use warp::{sse, Filter, Reply};

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
use crate::web::routes::Routes;

mod config;
mod data;
mod dto;
mod errors;
mod event;
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

    // Instantiate the game_info bean
    let mut game_info = GameInfo::default();

    // Instanciation of unbounded channels for events sending/receiving
    let (state_change_sender, state_change_receiver) =
        tokio::sync::mpsc::unbounded_channel::<StateChange>();

    let (internal_event_sender, internal_event_receiver) =
        tokio::sync::mpsc::unbounded_channel::<InternalEvent>();

    // Instanciation of application beans
    let repository = PlayerRepository::new(db_pool.clone());

    let service = Arc::new(Mutex::new(BuzzService {
        tx: internal_event_sender,
        repository: repository.clone(),
    }));

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
                    InternalEvent::PlayerAdded(score) => {
                        if !started.lock().await.load(Ordering::Relaxed) {
                            if let Messages::PlayerScore { player_name, .. } = score.clone() {
                                let ready = game_info.add_player(player_name).await;

                                emitters
                                    .emit_score(score, p.clone(), game_info.min_players)
                                    .unwrap();

                                if ready {
                                    emitters.on_game_started(&mut game_info).await.unwrap();
                                }
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

    // web

    let _state_changes_stream = UnboundedReceiverStream::new(state_change_receiver);

    warp::serve(
        Routes::add_player(service.clone())
            .or(Routes::register_buzz(service.clone()))
            .or(Routes::register_answer(service.clone()))
            .with(warp::cors().allow_any_origin())
            .recover(crate::web::exception_handlers::handle_error),
    )
    .run(([127, 0, 0, 1], 3030))
    .await;

    Ok(())
}

fn sse_state(state: StateChange) -> Result<Event, Infallible> {
    let s = serde_json::to_string(&state).unwrap();
    Ok(sse::Event::default().data(s))
}

fn stream_state(
    stream: BroadcastStream<StateChange>,
) -> impl Stream<Item = Result<Event, Infallible>> + Send + 'static {
    stream.map(|state| sse_state(state.unwrap()))
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
