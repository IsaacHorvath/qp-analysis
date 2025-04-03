use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use common::models::BreakdownTypeParseError;
use diesel::result::Error as DieselError;
use diesel_async::pooled_connection::bb8::RunError;
use tokio::sync::mpsc::error::SendError;

use crate::reaper::Message;

/// An error type for the backend

pub enum AppError {
    
    /// A generic, unrecoverable error. Translates to http status code `500
    /// Internal Server Error`.
    ///
    /// This error will cause frontend charts to enter a fail state and display
    /// "an error occurred". This state requires a reload.
    
    GenericError,
    
    /// An error indicating all connections in the connection pool are in use.
    /// Translates to http status code `503 Service Unavailable`.
    ///
    /// This error probably means the server is getting too many requests and/or
    /// the database has slowed to a crawl. For now it also causes charts to
    /// enter the same fail state as the GenericError.
    
    ConnectionPoolError,
    
    /// An error indicating the handler was cancelled by the reaper. Translates to
    /// http status code `204 No Content`.
    ///
    /// This error means the user navigated away from the page or sent a new search
    /// request, and the server was asked to kill existing queries. A chart that
    /// receives the corresponding status code `204` will not enter a fail state.
    
    Cancelled
}

// note: the message is unused at this time
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
