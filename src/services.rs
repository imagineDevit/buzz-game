pub(crate) mod buzz_services;

#[cfg(test)]
mod tests {
    use rstest::*;
    use tokio_stream::wrappers::UnboundedReceiverStream;
    use tokio_stream::StreamExt;

    use crate::config::app::init_config;
    use crate::data::db::create_db_pool;
    use crate::dto::messages::Messages;
    use crate::dto::requests::Requests;
    use crate::dto::responses::Response;
    use crate::{clear_db, get_connection, init_db, BuzzService, InternalEvent, PlayerRepository};

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

    #[rstest]
    #[trace]
    async fn add_player_test(
        #[future]
        #[notrace]
        repo: PlayerRepository,
    ) {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<InternalEvent>();
        let mut s = BuzzService {
            tx,
            repository: repo.await,
        };

        let mut rx = UnboundedReceiverStream::new(rx);

        let name = "Tom".to_string();

        let resp = s
            .add_player(Requests::AddPlayer { name: name.clone() })
            .await;

        match resp {
            Ok(Response::PlayerAdded) => {}
            _ => assert!(false),
        }

        let event = rx.next().await;

        assert!(event.is_some());

        match event.unwrap() {
            InternalEvent::PlayerAdded(score) => match score {
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
            },
            _ => assert!(false),
        }
    }

    #[rstest]
    #[trace]
    async fn register_buzz_test(
        #[future]
        #[notrace]
        repo: PlayerRepository,
    ) {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<InternalEvent>();
        let service = BuzzService {
            tx,
            repository: repo.await,
        };

        let mut rx = UnboundedReceiverStream::new(rx);

        let name = "Tom".to_string();

        let resp = service.register_buzz(Requests::RegisterBuzz {
            player_name: name.clone(),
        });

        match resp {
            Ok(Response::BuzzRegistered) => {}
            _ => assert!(false),
        }

        let event = rx.next().await;

        assert!(event.is_some());

        match event.unwrap() {
            InternalEvent::BuzzRegistered(player_name) => {
                assert_eq!(player_name, name.clone())
            }
            _ => assert!(false),
        }
    }

    #[rstest]
    #[trace]
    async fn register_answer_test(
        #[future]
        #[notrace]
        repo: PlayerRepository,
    ) {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<InternalEvent>();
        let service = BuzzService {
            tx,
            repository: repo.await,
        };

        let mut rx = UnboundedReceiverStream::new(rx);

        let name = "Tom".to_string();

        let resp = service.register_answer(Requests::RegisterAnswer {
            player_name: name.clone(),
            question_number: 0,
            answer_number: 2,
        });

        match resp {
            Ok(Response::AnswerRegistered) => {}
            _ => assert!(false),
        }

        let event = rx.next().await;

        assert!(event.is_some());

        match event.unwrap() {
            InternalEvent::AnswerRegistered {
                answer_number,
                player_name,
            } => {
                assert_eq!(player_name, name.clone());
                assert_eq!(2, answer_number);
            }
            _ => assert!(false),
        }
    }
}
