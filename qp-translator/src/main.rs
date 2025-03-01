use std::fs;
use diesel::MysqlConnection;
use time::{PrimitiveDateTime, Time, Date};
use time::macros::format_description;
use regex::Regex;
use aho_corasick::AhoCorasick;
use scraper::{Html, Selector};
use models::*;
use crate::db::*;

mod db;
mod models;

pub fn main() {
    let mut files = fs::read_dir("/home/isaac/.rust/qp-scraper/downloads/").unwrap()
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, _>>().unwrap();
        
    files.sort();
    
    for file in files {
        let file_name = format!("{}", file.display());
        println!("{}", file_name);
        
        let reg = Regex::new(r#"/(\d{4}-\d{2}-\d{2})_"#).unwrap();
        let file_date = &reg.captures(&file_name).unwrap()[1];
        let format = format_description!("[year]-[month]-[day]");
        let date = Date::parse(file_date, &format).unwrap();
        
        // if date < Date::from_calendar_date(2024, time::Month::October, 23).unwrap() {
        //     continue;
        // }
        
        let html = fs::read_to_string(file).expect("couldn't read file")
            .replace("M<sup>me</sup>", "Mme.")
            .replace("M<span id=\"P197_11242\"/>.", "M.")
            .replace("> <strong>Hon. Sam Oosterhoff", "><strong>Hon. Sam Oosterhoff");
        let fragment = Html::parse_fragment(&html);
        
        let ts_selector = Selector::parse("div[id=\"transcript\"]").unwrap();
        let p_selector = Selector::parse("p").unwrap();
        let ts = fragment.select(&ts_selector).next().unwrap();
        
        let connection = &mut establish_connection();
        
        let head_link = fragment.select(&Selector::parse("link").unwrap()).next().expect("couldn't find head link");
        let transcript_id = post_transcript(connection, NewTranscript { link: head_link.value().attr("href").unwrap() });
        println!("transcript id {}", transcript_id);
        
        let mut time = Time::MIDNIGHT;
        let mut speaker: Speaker = Default::default();
        let mut speech = "".to_string();
        let mut start_time = time;
        for element in ts.select(&p_selector) {
            let class_attr = element.value().attr("class");
            let text = element.text().collect::<Vec<_>>();
            match class_attr {
                Some("procedure") => if time == Time::MIDNIGHT { time = procedure_time(text, time) },
                Some("timeStamp") => time = parse_time(text[0], time),
                Some("speakerStart") => {
                    // not performant code, do something else here than .replace.replace
                    let name = text[0]
                            .replace("Wai Lam (William)", "William")
                            .replace("Jennifer (Jennie) Stevens", "Jennifer Stevens");
                    
                    if speaker.id != 0 {
                        //println!("{} - {}: {} {} ({}): {}", start_time, time, speaker.first_name, speaker.last_name, speaker.id, speech);
                        post(connection, &speaker.id, &transcript_id, &mut speech, &(PrimitiveDateTime::new(date, start_time)), &(PrimitiveDateTime::new(date, time)));
                    }
                    
                    if name.contains("Interjection") || name.contains("Une voix") || name.contains("Des voix") {
                        speaker = Default::default();
                    }
                    else {
                        speaker = parse_speaker(connection, &name);
                        speech = text[1..].join("");
                        start_time = time;
                    }
                },
                _ => {
                    speech.push_str(" ");
                    speech.push_str(&(text.join("")));
                },
            }
        }
        post(connection, &speaker.id, &transcript_id, &mut speech, &(PrimitiveDateTime::new(date, start_time)), &(PrimitiveDateTime::new(date, time)));
    }
}

pub fn post(conn: &mut MysqlConnection, speaker: &i32, transcript: &i32, text: &mut String, start: &PrimitiveDateTime, end: &PrimitiveDateTime) {
    let text = text.trim();
    let clean_text = clean(text);
    let new_speech = NewSpeech { speaker, text: text, transcript, start, end };
    post_speech(conn, new_speech, clean_text);
}

pub fn procedure_time(text: Vec<&str>, time: Time) -> Time {
    let reg = Regex::new("\\W(\\d{4})\\W").unwrap();
    for line in text {
        if let Some(capture) = reg.captures(line) {
            return parse_time(&capture[1], time);
        }
    }
    Time::MIDNIGHT
}

pub fn clean(text: &str) -> String {
    let patterns = &[".", ",", ";", ":", "!", "?", "\'", "\"", "”", "“", "’", "‘", "(", ")", "[", "]", "{", "}", "«", "»"];
    let ac = AhoCorasick::builder().build(patterns).unwrap();
    let mut clean = String::new();
    ac.replace_all_with(text, &mut clean, |_, _, dst| {
        dst.push_str("");
        true
    });
    
    clean = clean.to_lowercase().replace("—", " ");
    clean.insert(0, ' ');
    clean.push(' ');
    clean
}

pub fn parse_time(text: &str, time: Time) -> Time {
    println!("New time: {}", text);
    let new_time = Time::from_hms(text[0..2].parse().unwrap(), text[2..4].parse().unwrap(), 0).unwrap();
    if new_time < time { panic!("time can't go backwards") };
    new_time
}

pub fn parse_speaker(connection: &mut MysqlConnection, name: &str) -> Speaker {
    let reg = Regex::new(r#"(Hon\.|L’hon\.|Mr\.|Ms\.|Mrs\.|Mme\.?|M\.|Miss|Mister|Missus|Madame|Monsieur|MPP)\s+([\w\.-]+)\s([\w\.-]+\s)?([\w\.-]+)"#).unwrap();
    let Some(captures) = reg.captures(name) else { panic!("{}", name) };
    let speaker = get_speaker_by_name(connection, &captures[2], &captures[captures.len()-1]);
    speaker
}
