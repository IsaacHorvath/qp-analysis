use reqwest;
use std::time::Duration;
use regex::Regex;
use scraper::{Html, Selector};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut url = "https://www.ola.org/en/legislative-business/house-documents/parliament-43/session-1/2022-08-08/hansard".to_string();
    let mut volume = 'A';
    
    loop {
        let reg = Regex::new("/(\\d{4}-\\d{2}-\\d{2})/").unwrap();
        let Some(capture) = reg.captures(&url) else {
            println!("Couldn't read date from {}", url);
            return Ok(())
        };
        let date = &capture[1];
        
        let response = reqwest::get(&url).await?;
        let html = response.text().await?;
        
        let file_name = format!("/home/isaac/.rust/qp-scraper/downloads/{}_{}.txt", date, volume);
        let mut buffer = File::create(file_name.clone()).await?;
        buffer.write_all(html.as_bytes()).await?;
        println!("Wrote {}", file_name);
        
        volume = ((volume as u8) + 1) as char;
        if let Some(next_volume_url) = get_next_volume_url(volume, &html) {
            url = next_volume_url;
            println!("another volume found");
        }
        else {
            println!("moving to next day");
            volume = 'A';
            // todo migrate to scraper
            let reg = Regex::new("href=\"([\\/A-z0-9-\\.:]+)\">\\s*Next").unwrap();
            let Some(capture) = reg.captures(&html) else {
                println!("Couldn't read link for next page on {}", date);
                return Ok(())        
            };
            url = (&capture[1]).to_string();
        
        }
        
        if url.chars().nth(0).unwrap() == '/' {
            url = format!("https://www.ola.org{}", url);
        }
        
        println!("Next url in 3: {}", url);
        tokio::time::sleep(Duration::new(3, 0)).await;
    }
}

pub fn get_next_volume_url(volume: char, html: &str) -> Option<String> {
    let fragment = Html::parse_fragment(&html);
    let fl_selector = Selector::parse(".field-content.nav-link").unwrap();
    let a_selector = Selector::parse("a").unwrap();
    let vr_selector = Selector::parse(".volume-wrapper").unwrap();
    for field_link in fragment.select(&fl_selector) {
        let a = field_link.select(&a_selector).next().unwrap();
        let vr = field_link.select(&vr_selector).next().unwrap();
        
        let vt_collect = vr.text().collect::<Vec<_>>();
        if vt_collect.len() < 1 { return None }
        let volume_text = vt_collect[0];
        if volume_text.len() < 6 { return None }
        let volume_char = volume_text.chars().nth(5).unwrap();
        
        println!("{} {}", volume, volume_char);
        
        if volume_char == volume {
            return Some(a.value().attr("href").unwrap().to_string());
        }
        // else {
        //     if target_volume == volume_char {
        //         return Some(a.value().attr("href").unwrap().to_string());
        //     }
        //     target_volume = ((target_volume as u8) + 1) as char;
        //     println!("Volume: {}", target_volume);
        // }
    }
    None
}
