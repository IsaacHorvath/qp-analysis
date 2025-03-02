use reqwest;
use std::time::Duration;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    for sitting in 1..391 {
        let response = reqwest::get(&format!("https://www.ourcommons.ca/Content/House/441/Debates/{:03}/HAN{:03}-E.XML", sitting, sitting)).await?;
        let html = response.text().await?;
        
        let file_name = format!("fh-scraper/downloads/{:03}.xml", sitting);
        let mut buffer = File::create(file_name.clone()).await?;
        buffer.write_all(html.as_bytes()).await?;
        println!("Wrote {}", file_name);

        tokio::time::sleep(Duration::new(3, 0)).await;
    }
    
    Ok(())
}
