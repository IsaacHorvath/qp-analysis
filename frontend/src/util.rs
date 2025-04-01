use gloo_net::http::{Request, Response};
use anyhow::bail;
use anyhow::Result;

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
