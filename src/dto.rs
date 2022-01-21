pub(crate) mod messages;
pub(crate) mod requests;
pub(crate) mod responses;
pub(crate) mod states;

#[cfg(test)]
mod requests_tests {

    use crate::dto::requests::Requests;
    use rstest::*;

    #[fixture(name = "Joe".to_string())]
    fn add_player_json(name: String) -> String {
        format!("{} \"name\": \"{}\" {}", "{", name, "}")
    }

    #[fixture(name = "Joe".to_string())]
    fn register_buzz_json(name: String) -> String {
        format!("{} \"playerName\": \"{}\" {}", "{", name, "}")
    }

    #[fixture(name = "Joe".to_string(), question = 1, answer = 2)]
    fn register_answer_json(name: String, question: u8, answer: u8) -> String {
        format!(
            "{} \"playerName\": \"{}\", \"questionNumber\" : {}, \"answerNumber\" : {}  {}",
            "{", name, question, answer, "}"
        )
    }

    #[rstest(add_player_json as json)]
    fn deserialize_add_player_request_test(json: String) {
        println!("{}", json);
        let r: Requests = serde_json::from_str(json.as_str()).unwrap();
        match r {
            Requests::AddPlayer { name } => {
                assert!(true);
                assert_eq!(name, "Joe".to_string())
            }
            _ => assert!(false),
        }
    }

    #[rstest(register_buzz_json as json)]
    fn deserialize_register_buzz_request_test(json: String) {
        let r: Requests = serde_json::from_str(json.as_str()).unwrap();
        match r {
            Requests::RegisterBuzz { player_name } => {
                assert!(true);
                assert_eq!(player_name, "Joe".to_string())
            }
            _ => assert!(false),
        }
    }

    #[rstest(register_answer_json as json)]
    fn deserialize_register_answer_request_test(json: String) {
        let r = serde_json::from_str(json.as_str()).unwrap();

        println!("{:?}", r);
        match r {
            Requests::RegisterAnswer {
                player_name,
                question_number,
                answer_number,
            } => {
                assert!(true);
                assert_eq!(player_name, "Joe".to_string());
                assert_eq!(question_number, 1);
                assert_eq!(answer_number, 2);
            }
            _ => assert!(false),
        }
    }
}

#[cfg(test)]
mod responses_tests {

    use crate::dto::responses::Response;
    use rstest::*;

    #[fixture(msg = "error occurred".to_string())]
    fn error_json(msg: String) -> String {
        format!(
            "{}\"type\":\"{}\",\"message\":\"{}\"{}",
            "{", "ERROR", msg, "}"
        )
    }

    fn response_json(t: String) -> String {
        format!("{}\"type\":\"{}\"{}", "{", t, "}")
    }

    #[fixture(msg = "error occurred".to_string())]
    fn error_response(msg: String) -> Response {
        Response::Error {
            message: msg,
            code: 500,
        }
    }

    #[rstest(error_json as json)]
    #[trace]
    fn error_response_test(json: String, error_response: Response) {
        let str = serde_json::to_string(&error_response).unwrap();
        assert_eq!(str, json);
    }

    #[rstest]
    #[case("GAME_STARTED".to_string(), Response::GameStarted)]
    #[case("BUZZ_REGISTERED".to_string(), Response::BuzzRegistered)]
    #[case("ANSWER_REGISTERED".to_string(), Response::AnswerRegistered)]
    #[trace]
    fn response_test(#[case] t: String, #[case] response: Response) {
        let str = serde_json::to_string(&response).unwrap();
        assert_eq!(str, response_json(t));
    }
}
