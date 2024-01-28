use thiserror::Error;

use super::auth::LoginError;
use super::timetable::ParseError as ParseTimetableError;

/// Error of request to api
#[derive(Debug, Error)]
pub enum RequestError {
    /// Authentication error
    #[error("{0}")]
    Login(#[from] LoginError),
    #[error("{0}")]
    Request(#[from] reqwest::Error),
    #[error("server returned unknown response: {0}")]
    UnknownResponse(&'static str),
    #[error("{0}")]
    ParseFailed(#[from] ParseTimetableError),
}

pub type RequestResult<T> = Result<T, RequestError>;
