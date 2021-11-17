use std::convert::Infallible;

use warp::http::StatusCode;
use warp::{Rejection, Reply};

use crate::dto::responses::Response;
use crate::CustomError;

pub async fn handle_error(error: Rejection) -> Result<impl Reply, Infallible> {
    let mut status_code = StatusCode::INTERNAL_SERVER_ERROR;
    let mut msg = String::new();

    if let Some(e) = error.find::<CustomError>() {
        msg = format!("{}", e.to_string());
        match e {
            CustomError::CreateDBPoolError(_) => {}
            CustomError::GetDBConnectionError(_) => {}
            CustomError::ExecuteDBQueryError { .. } => {}
            CustomError::OpenFileError(_) => {}
            CustomError::ReadFileError(_) => {}
            CustomError::YamlDeserializationError(_) => {}
            CustomError::PlayerAlreadyExistWithNameError(_) => {}
            CustomError::PlayerNotFoundWithNameError(_) => {}
            CustomError::SendEventError(_) => {
                status_code = StatusCode::BAD_GATEWAY;
            }
            CustomError::BadRequestTypeError { .. } => {
                status_code = StatusCode::BAD_REQUEST;
            }
        }
    }

    Ok(warp::reply::with_status(
        warp::reply::json(&Response::Error {
            message: msg,
            code: status_code.as_u16(),
        }),
        status_code,
    ))
}
