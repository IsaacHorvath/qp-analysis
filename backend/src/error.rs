//use serde::{Deserialize, Serialize};
use common::models::BreakdownTypeParseError;
use diesel_async::pooled_connection::bb8::RunError;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response}
};

pub enum AppError {
    BreakdownError,
    ConnectionPoolError,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::ConnectionPoolError => (StatusCode::SERVICE_UNAVAILABLE, "our servers are very busy - please try again later".to_owned()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "something went wrong - please refresh the page and try again".to_owned()),
        };

        (status, message).into_response()
    }
}

impl From<BreakdownTypeParseError> for AppError {
    fn from(_: BreakdownTypeParseError) -> Self {
        Self::BreakdownError
    }
}

impl From<RunError> for AppError {
    fn from(_: RunError) -> Self {
        Self::ConnectionPoolError
    }
}
