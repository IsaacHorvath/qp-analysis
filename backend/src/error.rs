//use serde::{Deserialize, Serialize};
use common::models::BreakdownTypeParseError;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response}
};

pub enum AppError {
    BreakdownError(BreakdownTypeParseError),
    ConnectionPoolError,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::BreakdownError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "something went wrong - please refresh the page and try again".to_owned()),
            AppError::ConnectionPoolError => (StatusCode::INTERNAL_SERVER_ERROR, "our servers are very busy - please try again later".to_owned()),
        };

        (status, message).into_response()
    }
}

impl From<BreakdownTypeParseError> for AppError {
    fn from(e: BreakdownTypeParseError) -> Self {
        Self::BreakdownError(e)
    }
}
