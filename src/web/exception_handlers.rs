use std::convert::Infallible;

use warp::http::StatusCode;
use warp::{Rejection, Reply};

use crate::dto::responses::Response;
use crate::CustomError;

pub async fn handle_error(error: Rejection) -> Result<impl Reply, Infallible> {
    let mut status_code = StatusCode::INTERNAL_SERVER_ERROR;
    let mut msg = "";

    if let Some(e) = error.find::<CustomError>() {
        match e {
            CustomError::CreateDBPoolError(_) => {
                status_code = StatusCode::INTERNAL_SERVER_ERROR;
                msg = "";
            }
            CustomError::GetDBConnectionError(_) => {}
            CustomError::ExecuteDBQueryError { .. } => {
                status_code = StatusCode::BAD_REQUEST;
                msg = "Could not execute request";
            }
            CustomError::OpenFileError(_) => {}
            CustomError::ReadFileError(_) => {}
            CustomError::YamlDeserializationError(_) => {}
            CustomError::PlayerAlreadyExistWithNameError(_) => {}
            CustomError::PlayerNotFoundWithNameError(_) => {}
            CustomError::SendEventError(_) => {}
            CustomError::SendInternalEventError(_) => {}
            CustomError::BadRequestTypeError { .. } => {}
        }
    }

    Ok(warp::reply::with_status(
        warp::reply::json(&Response::Error {
            message: msg.to_string(),
        }),
        status_code,
    ))
}
