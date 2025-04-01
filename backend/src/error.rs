use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use common::models::BreakdownTypeParseError;
use diesel::result::Error as DieselError;
use diesel_async::pooled_connection::bb8::RunError;
use tokio::sync::mpsc::error::SendError;

use crate::reaper::Message;

pub enum AppError {
    GenericError,
    ConnectionPoolError,
    Cancelled
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::Cancelled => (
                StatusCode::NO_CONTENT,
                "request cancelled".to_owned(),
            ),
            AppError::ConnectionPoolError => (
                StatusCode::SERVICE_UNAVAILABLE,
                "our servers are very busy - please try again later".to_owned(),
            ),
            AppError::GenericError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "something went wrong - please refresh the page and try again".to_owned(),
            ),
        };

        (status, message).into_response()
    }
}

impl From<BreakdownTypeParseError> for AppError {
    fn from(_: BreakdownTypeParseError) -> Self {
        Self::GenericError
    }
}

impl From<SendError<Message>> for AppError {
    fn from(_: SendError<Message>) -> Self {
        Self::GenericError
    }
}

impl From<DieselError> for AppError {
    fn from(_: DieselError) -> Self {
        Self::GenericError
    }
}

impl From<RunError> for AppError {
    fn from(_: RunError) -> Self {
        Self::ConnectionPoolError
    }
}
