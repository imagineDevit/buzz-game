pub(crate) mod buzz_services;
mod utils;

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::sync::Arc;

    use rstest::*;
    use tokio::sync::Mutex;
    use tokio_stream::wrappers::UnboundedReceiverStream;
    use tokio_stream::StreamExt;

    use crate::config::app::init_config;
    use crate::data::db::create_db_pool;
    use crate::dto::messages::Messages;
    use crate::dto::requests::Requests;
    use crate::dto::responses::Response;
    use crate::{
        clear_db, get_connection, init_db, Answer, BuzzService, GameInfo, PlayerRepository,
        StateChange,
    };

    #[fixture]
    async fn repo() -> PlayerRepository {
        let config = init_config().await.unwrap();
        let pool = create_db_pool(&config).unwrap().clone();
        let pool_clone = pool.clone();

        tokio::spawn(async move {
            let connection = get_connection(&pool_clone).await.unwrap();
            clear_db(&connection).await.unwrap();
            init_db(&connection).await.unwrap();
        })
        .await
        .unwrap();

        PlayerRepository::new(pool.clone())
    }

    #[fixture]
    fn game_info() -> Arc<Mutex<GameInfo>> {
        let mut game_info = GameInfo::new(vec![]);
        game_info.min_players = 1;
        Arc::new(Mutex::new(game_info))
    }

    #[rstest]
    #[trace]
    async fn add_player_test(
        #[future]
        #[notrace]
        repo: PlayerRepository,
        game_info: Arc<Mutex<GameInfo>>,
    ) {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<StateChange>();
        let mut s = BuzzService {
            repository: repo.await,
        };

        let mut rx = UnboundedReceiverStream::new(rx);

        let name = "Tom".to_string();

        let resp = s
            .add_player(Requests::AddPlayer { name: name.clone() }, game_info, tx)
            .await;

        match resp {
            Ok(Response::PlayerAdded(ready)) => assert!(ready),
            _ => assert!(false),
        }

        let event = rx.next().await;

        assert!(event.is_some());

        match event.unwrap().message {
            Messages::PlayerScore {
                player_name,
                score,
                update,
                good_answer,
            } => {
                assert_eq!(name, player_name);
                assert_eq!(0, score);
                assert_eq!(false, update);
                assert_eq!("".to_string(), good_answer);
            }
            _ => assert!(false),
        }
    }

    #[rstest]
    #[trace]
    async fn register_buzz_test(
        #[future]
        #[notrace]
        repo: PlayerRepository,
        game_info: Arc<Mutex<GameInfo>>,
    ) {
        let service = BuzzService {
            repository: repo.await,
        };

        let name = "Tom".to_string();

        let resp = service
            .register_buzz(
                Requests::RegisterBuzz {
                    player_name: name.clone(),
                },
                game_info,
            )
            .await;

        match resp {
            Ok(Response::BuzzRegistered) => {}
            _ => assert!(false),
        }
    }

    #[rstest]
    #[trace]
    async fn register_answer_test(
        #[future]
        #[notrace]
        repo: PlayerRepository,
        game_info: Arc<Mutex<GameInfo>>,
    ) {
        let name = "Tom".to_string();

        let service = BuzzService {
            repository: repo.await,
        };

        let g = game_info.clone();

        let mut game = g.lock().await;

        game.current_question = Some((
            Messages::Question {
                number: 0,
                label: "".to_string(),
                points: 2,
                answers: HashSet::new(),
            },
            Answer {
                number: 2,
                label: "".to_string(),
                good: true,
            },
        ));

        game.add_buzz(name.clone()).await;

        let resp = service
            .register_answer(
                Requests::RegisterAnswer {
                    player_name: name.clone(),
                    question_number: 0,
                    answer_number: 2,
                },
                game_info,
            )
            .await;

        match resp {
            Ok(Response::ScoreUpdated(Messages::PlayerScore {
                player_name,
                score,
                good_answer,
                update,
            })) => {
                assert_eq!(player_name, name.clone());
                assert_eq!(score, 2);
                assert_eq!(String::default(), good_answer);
                assert!(update);
            }
            _ => assert!(false),
        }
    }
}
