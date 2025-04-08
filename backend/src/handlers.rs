use crate::AppState;
use crate::db::*;
use crate::dummy_db::*;
use crate::error::AppError;
use crate::reaper::{ActiveQuery, Message};
use axum::{
    extract::{Path, State},
    Json,
};
use common::models::*;
use std::str::FromStr;
use tokio_util::sync::CancellationToken;

/// Return all speakers in the database.

pub async fn speakers(State(state): State<AppState>) -> Result<Json<Vec<SpeakerResponse>>, AppError> {
    if let Some(pool) = state.connection_pool {
        let mut conn = pool.get().await?;
        Ok(Json(get_speakers(&mut conn).await?))
    } else {
        Ok(Json(dummy_get_speakers()))
    }
}

/// Return all speeches matching the given word and breakdown type. See db call for
/// description of return columns.
///
/// This handler registers a cancellation token with the reaper, and will return
/// status 204 if cancelled.

pub async fn breakdown(
    State(state): State<AppState>,
    Path(breakdown_type): Path<String>,
    Json(payload): Json<DataRequest>,
) -> Result<Json<Vec<BreakdownResponse>>, AppError> {
    let breakdown_type = BreakdownType::from_str(breakdown_type.as_str())?;
    if let Some(pool) = state.connection_pool {
        let mut conn = pool.get().await?;
        let conn_id = get_connection_id(&mut conn).await?;

        let token = CancellationToken::new();

        state
            .sender
            .send(Message::Register((
                ActiveQuery {
                    uuid: payload.uuid.clone(),
                    conn_id,
                    speech: false,
                },
                token.clone(),
            )))
            .await?;

        let search = payload.search.to_lowercase().replace(
            |c: char| !(c.is_ascii_alphanumeric() || c == ' ' || c == '-'),
            "",
        );

        let response = tokio::select! {
            res = get_breakdown_word_count(&mut conn, breakdown_type, &search) => {
                Ok(Json(res?))
            }
            _ = token.cancelled() => {
                Err(AppError::Cancelled)
            }
        };

        // todo don't send this if we cancelled anyway
        state
            .sender
            .send(Message::Deregister(ActiveQuery {
                uuid: payload.uuid,
                conn_id,
                speech: false,
            }))
            .await?;

        response
    } else {
        Ok(Json(dummy_get_breakdown_word_count(breakdown_type)))
    }
}

/// Return population data matching the given word. See db call for description of
/// return columns.
///
/// This handler registers a cancellation token with the reaper, and will return
/// status 204 if cancelled.

pub async fn population(
    State(state): State<AppState>,
    Json(payload): Json<DataRequest>,
) -> Result<Json<Vec<PopulationResponse>>, AppError> {
    if let Some(pool) = state.connection_pool {
        let mut conn = pool.get().await?;
        let conn_id = get_connection_id(&mut conn).await?;

        let token = CancellationToken::new();

        state
            .sender
            .send(Message::Register((
                ActiveQuery {
                    uuid: payload.uuid.clone(),
                    conn_id,
                    speech: false,
                },
                token.clone(),
            )))
            .await?;

        let search = payload.search.to_lowercase().replace(
            |c: char| !(c.is_ascii_alphanumeric() || c == ' ' || c == '-'),
            "",
        );

        let response = tokio::select! {
            res = get_population_word_count(&mut conn, &search) => {
                Ok(Json(res?))
            }
            _ = token.cancelled() => {
                Err(AppError::Cancelled)
            }
        };

        state
            .sender
            .send(Message::Deregister(ActiveQuery {
                uuid: payload.uuid,
                conn_id,
                speech: false,
            }))
            .await?;

        response
    } else {
        Ok(Json(dummy_get_population_word_count()))
    }
}

/// Return all speeches matching the given word, breakdown type, and id. See db call
/// for description of return columns.
///
/// This handler registers a cancellation token with the reaper, and will return
/// status 204 if cancelled.

pub async fn speeches(
    Path((breakdown_type, id)): Path<(String, i32)>,
    State(state): State<AppState>,
    Json(payload): Json<DataRequest>,
) -> Result<Json<Vec<SpeechResponse>>, AppError> {
    if let Some(pool) = state.connection_pool {
        let mut conn = pool.get().await?;
        let conn_id = get_connection_id(&mut conn).await?;

        let token = CancellationToken::new();

        state
            .sender
            .send(Message::Register((
                ActiveQuery {
                    uuid: payload.uuid.clone(),
                    conn_id,
                    speech: true,
                },
                token.clone(),
            )))
            .await?;

        let search = payload.search.to_lowercase().replace(
            |c: char| !(c.is_ascii_alphanumeric() || c == ' ' || c == '-'),
            "",
        );
        let breakdown_type = BreakdownType::from_str(breakdown_type.as_str())?;

        let response = tokio::select! {
            res = get_speeches(&mut conn, breakdown_type, id, &search) => {
                Ok(Json(res?))
            }
            _ = token.cancelled() => {
                Err(AppError::Cancelled)
            }
        };

        state
            .sender
            .send(Message::Deregister(ActiveQuery {
                uuid: payload.uuid,
                conn_id,
                speech: true,
            }))
            .await?;

        response
    } else {
        return Ok(Json(dummy_get_speeches()));
    }
}

/// Cancel all current requests associated with the uuid in the payload.

pub async fn cancel(
    State(state): State<AppState>,
    Json(payload): Json<CancelRequest>,
) -> Result<(), AppError> {
    state.sender.send(Message::Kill(payload.uuid)).await?;
    Ok(())
}

/// Cancel all current speech requests associated with the uuid in the payload.

pub async fn cancel_speech(
    State(state): State<AppState>,
    Json(payload): Json<CancelRequest>,
) -> Result<(), AppError> {
    state.sender.send(Message::KillSpeech(payload.uuid)).await?;
    Ok(())
}
