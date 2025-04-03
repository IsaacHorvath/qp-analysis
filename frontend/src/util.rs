use gloo_net::http::{Request, Response};
use anyhow::bail;
use anyhow::Result;

/// A speaker as it is stored on the frontend.

#[derive(Clone, PartialEq, Debug)]
pub struct Speaker {
    pub first_name: String,
    pub last_name: String,
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
        
    if !resp.ok() {
        bail!("request failed");
    };
    
    Ok(resp)
}
