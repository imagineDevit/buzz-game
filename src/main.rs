#![allow(dead_code)]

use std::collections::HashSet;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_stream::StreamExt;

use data::db::*;

use crate::config::app::init_config;
use crate::data::entities::Player;
use crate::data::repositories::{PlayerRepository, SearchAttributes};
use crate::dto::messages::{Answer, Messages};
use crate::dto::requests::Requests;
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

    let handle = tokio::spawn(async move {
        let connection = get_connection(&pool).await.unwrap();
        let _ = data::db::init_db(&connection).await.unwrap();
    });

    handle.await.unwrap();

    let repository = PlayerRepository::new(db_pool.clone());

    let mut game_info = GameInfo::default();

    let questions = vec![
        Messages::Question {
            number: 0,
            label: "What's your name".to_string(),
            points: 1,
            answers: HashSet::from([
                Answer {
                    number: 0,
                    label: "Joe".to_string(),
                    good: false,
                },
                Answer {
                    number: 1,
                    label: "Chlo".to_string(),
                    good: true,
                },
            ]),
        },
        Messages::Question {
            number: 1,
            label: "Are u ready".to_string(),
            points: 2,
            answers: HashSet::from([
                Answer {
                    number: 0,
                    label: "Joe".to_string(),
                    good: false,
                },
                Answer {
                    number: 1,
                    label: "Chlo".to_string(),
                    good: true,
                },
            ]),
        },
    ];

    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<StateChange>();

    let mut emitters = EventEmitters {
        state_changes: tx,
        question_iterator: questions.iter(),
    };

    let mut stream = UnboundedReceiverStream::new(rx);

    tokio::spawn(async move {
        loop {
            while let Some(state) = stream.next().await {
                println!("received {:?}", state)
            }
        }
    });

    let player = Player::with_name("Joe".to_string());

    repository.insert(&player).await?;

    emitters.on_game_started(&mut game_info)?;
    emitters.on_buzz_registered(Requests::RegisterBuzz {
        player_name: "Joe".to_string(),
    })?;

    let p = repository
        .find_by(SearchAttributes::Name("Joe".to_string()))
        .await?;

    let repo = repository.clone();

    emitters
        .on_answer_registered(
            Requests::RegisterAnswer {
                player_name: "Joe".to_string(),
                question_number: 0,
                answer_number: 1,
            },
            p.unwrap(),
            |(p, point)| async move { repo.update_score(p.id, p.score + point).await },
            &mut game_info,
        )
        .await?;

    let p = repository
        .find_by(SearchAttributes::Name("Joe".to_string()))
        .await?;

    let repo = repository.clone();
    emitters
        .on_answer_registered(
            Requests::RegisterAnswer {
                player_name: "Joe".to_string(),
                question_number: 1,
                answer_number: 1,
            },
            p.unwrap(),
            |(p, point)| async move { repo.update_score(p.id, p.score + point).await },
            &mut game_info,
        )
        .await?;

    Ok(())
}
