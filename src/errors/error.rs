use crate::dto::states::StateChange;
use crate::event::internal_events::InternalEvent;
use mobc_postgres::tokio_postgres;
use thiserror::Error;
use warp::reject::Reject;

#[derive(Error, Debug)]
pub enum CustomError {
    #[error("Error occurred while creating database pool : {0}")]
    CreateDBPoolError(#[from] tokio_postgres::Error),
    #[error("Error occurred while getting database pool connection: {0}")]
    GetDBConnectionError(#[from] mobc::Error<tokio_postgres::Error>),
    #[error("Error occurred while executing query {query}")]
    ExecuteDBQueryError {
        #[source]
        source: tokio_postgres::Error,
        query: String,
    },
    #[error("Error occurred while opening file: {0}")]
    OpenFileError(#[from] std::io::Error),
    #[error("Error occurred while reading file: {0}")]
    ReadFileError(#[from] std::string::FromUtf8Error),
    #[error("Error occurred deserializing yaml file: {0}")]
    YamlDeserializationError(#[from] serde_yaml::Error),
    #[error("Error occurred while trying to insert player with name {0}")]
    PlayerAlreadyExistWithNameError(String),
    #[error("Error occurred while searching player with id {0}")]
    PlayerNotFoundWithNameError(String),
    #[error("Error occurred while sending event message")]
    SendEventError(#[from] tokio::sync::mpsc::error::SendError<StateChange>),
    #[error("Error occurred while sending internal event message")]
    SendInternalEventError(#[from] tokio::sync::mpsc::error::SendError<InternalEvent>),
    #[error("Error occurred while tring to send bad message type")]
    BadRequestTypeError { message: String },
}

impl Reject for CustomError {}
