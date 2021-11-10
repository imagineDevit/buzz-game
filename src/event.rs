pub(crate) mod emitters;

#[cfg(test)]
mod event_emitters_test {
    use crate::data::entities::Player;
    use crate::dto::requests::Requests;
    use crate::dto::states::StateChangeType;
    use crate::{Answer, EventEmitters, GameInfo, Messages, StateChange};
    use rstest::*;
    use std::collections::HashSet;
    use tokio_stream::wrappers::UnboundedReceiverStream;
    use tokio_stream::StreamExt;

    #[rstest]
    #[trace]
    async fn test() {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<StateChange>();

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

        let mut emitters = EventEmitters {
            state_changes: tx,
            question_iterator: questions.iter(),
        };

        let mut stream = UnboundedReceiverStream::new(rx);

        /* start game */
        emitters.on_game_started(&mut game_info).unwrap();

        let state = stream.next().await;
        assert!(state.is_some());
        assert_eq!(StateChangeType::GameStart, state.unwrap().change_type);

        let state = stream.next().await;
        assert!(state.is_some());
        assert_eq!(StateChangeType::CanBuzz, state.unwrap().change_type);

        let state = stream.next().await;
        assert!(state.is_some());
        let change = state.unwrap();
        assert_eq!(StateChangeType::NewQuestion, change.change_type);
        let message = change.message;
        match message {
            m @ Messages::Question { .. } => {
                let cq = game_info.current_question.clone().unwrap();
                assert_eq!(cq.0, m);
                assert!(cq.1.good)
            }
            _ => assert!(false),
        }

        emitters
            .on_buzz_registered(Requests::RegisterBuzz {
                player_name: "Joe".to_string(),
            })
            .unwrap();
        let state = stream.next().await;
        assert!(state.is_some());
        assert_eq!(StateChangeType::CanBuzz, state.unwrap().change_type);

        let state = stream.next().await;
        assert!(state.is_some());
        assert_eq!(StateChangeType::NewBuzz, state.unwrap().change_type);

        emitters
            .on_answer_registered(
                Requests::RegisterAnswer {
                    player_name: "Joe".to_string(),
                    question_number: 0,
                    answer_number: 1,
                },
                Player::with_name("Joe".to_string()),
                |(pl, _)| async { Ok(pl) },
                &mut game_info,
            )
            .await
            .unwrap();

        let state = stream.next().await;
        assert!(state.is_some());
        assert_eq!(StateChangeType::NewAnswer, state.unwrap().change_type);

        let state = stream.next().await;
        assert!(state.is_some());
        assert_eq!(StateChangeType::NewPlayerScore, state.unwrap().change_type);

        let state = stream.next().await;
        assert!(state.is_some());
        assert_eq!(StateChangeType::CanBuzz, state.unwrap().change_type);

        let state = stream.next().await;
        assert!(state.is_some());
        let change = state.unwrap();
        assert_eq!(StateChangeType::NewQuestion, change.change_type);
        let message = change.message;
        match message {
            m @ Messages::Question { .. } => {
                let cq = game_info.current_question.clone().unwrap();
                assert_eq!(cq.0, m);
                assert!(cq.1.good)
            }
            _ => assert!(false),
        }

        emitters
            .on_buzz_registered(Requests::RegisterBuzz {
                player_name: "Joe".to_string(),
            })
            .unwrap();
        let state = stream.next().await;
        assert!(state.is_some());
        assert_eq!(StateChangeType::CanBuzz, state.unwrap().change_type);

        let state = stream.next().await;
        assert!(state.is_some());
        assert_eq!(StateChangeType::NewBuzz, state.unwrap().change_type);

        emitters
            .on_answer_registered(
                Requests::RegisterAnswer {
                    player_name: "Joe".to_string(),
                    question_number: 0,
                    answer_number: 1,
                },
                Player::with_name("Joe".to_string()),
                |(pl, _)| async { Ok(pl) },
                &mut game_info,
            )
            .await
            .unwrap();

        let state = stream.next().await;
        assert!(state.is_some());
        assert_eq!(StateChangeType::NewAnswer, state.unwrap().change_type);

        let state = stream.next().await;
        assert!(state.is_some());
        assert_eq!(StateChangeType::NewPlayerScore, state.unwrap().change_type);

        let state = stream.next().await;
        assert!(state.is_some());
        assert_eq!(StateChangeType::GameEnd, state.unwrap().change_type);
    }
}
