use common::models::BreakdownType;
use gloo_net::http::{Request, Response};
use anyhow::Result;

// todo think about moving structs to a models file

/// A speaker as it is stored on the frontend.

#[derive(Clone, PartialEq, Debug)]
pub struct Speaker {
    pub first_name: String,
    pub last_name: String,
}

/// An overlay selection, used to determine what set of speeches we are looking at
/// in the speech overlay.

#[derive(PartialEq, Clone)]
pub struct OverlaySelection {
    pub breakdown_type: BreakdownType,
    pub id: i32,
    pub heading: String,
}

/// Put a request to the given uri.
///
/// This helper function exists mostly to coalesce errors.

pub async fn put<T>(uri: &str, body: T) -> Result<Response>
    where T: serde::Serialize
{
    let req = Request::put(uri)
        .header("Content-Type", "application/json")
        .json(&body)?;
        
    let resp = req.send().await?;
    
    Ok(resp)
}
