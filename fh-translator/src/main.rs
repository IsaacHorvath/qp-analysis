use std::fs;
use diesel::MysqlConnection;
use time::{PrimitiveDateTime, Time, Date};
use time::macros::format_description;
use regex::{Regex, RegexBuilder};
use aho_corasick::AhoCorasick;
use models::*;
use crate::db::*;

mod db;
mod models;

pub fn main() {
    let connection = &mut establish_connection();
        
    let mut files = fs::read_dir("./fh-scraper/downloads/").unwrap()
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, _>>().unwrap();
        
    files.sort();
    
    for file in files {
        let file_name = format!("{}", file.display());
        println!("{}", file_name);
        
        let xml = fs::read_to_string(file).expect("couldn't read file");
        let doc = roxmltree::Document::parse(&xml).unwrap();
        
        let date_text = doc.descendants().find(|n| n.has_tag_name("ExtractedItem") && n.attribute("Name") == Some("Date")).unwrap().text().unwrap();
        let format = format_description!("[weekday repr:long], [month repr:long] [day padding:none], [year]");
        let date = Date::parse(date_text, &format).unwrap();
        
        let house_speaker = doc.descendants().find(|n| n.has_tag_name("ExtractedItem") && n.attribute("Name") == Some("SpeakerName")).unwrap().text().unwrap();
        
        // if date < Date::from_calendar_date(2021, time::Month::December, 01).unwrap() {
        //     continue;
        // }
        let reg = Regex::new(r#"/(\d\d\d)\.xml"#).unwrap();
        let Some(captures) = reg.captures(&file_name) else { panic!("couldn't get sitting in {}", file_name) };
        let link = &format!("https://www.ourcommons.ca/documentviewer/en/44-1/house/sitting-{}/hansard", captures[1].parse::<u16>().unwrap());
        let transcript_id = post_transcript(connection, NewTranscript { link });
        println!("transcript id {}", transcript_id);
        
        //if captures[1].parse::<u16>().unwrap() < 206 { continue; }
        
        let mut time = PrimitiveDateTime::new(date, Time::MIDNIGHT);
        let mut speaker: Speaker = Default::default();
        let mut speech = "".to_string();
        let mut start_time = time;
        let time_format = format_description!("([hour][minute])");
        
        for sb in doc.descendants().filter(|n| n.has_tag_name("SubjectOfBusiness")) {
            if let Some(timestamp) = sb.children().find(|n| n.has_tag_name("Timestamp")) {
                time = parse_time(date, timestamp.text().unwrap());
            }    
            if let Some(sbc) = sb.children().find(|n| n.has_tag_name("SubjectOfBusinessContent")) {
                for sbc_child in sbc.children() {
                    match sbc_child.tag_name().name() {
                        "Timestamp" => time = parse_time(date, sbc_child.text().unwrap()),
                        "Intervention" => {
                            for int_child in sbc_child.children().filter(|n| n.is_element()) {
                                match int_child.tag_name().name() {
                                    "PersonSpeaking" => {
                                        
                                        let name = int_child.children()
                                            .find(|n| n.has_tag_name("Affiliation")).unwrap()
                                            .text().unwrap();
                                        
                                        new_speaker(connection, transcript_id, &mut speaker, &mut speech, &mut start_time, &time, name, house_speaker);
                                            
                                    }
                                    "Content" => {
                                        for c_child in int_child.children().filter(|n| n.is_element()) {
                                            match c_child.tag_name().name() {
                                                "Timestamp" => time = parse_time(date, c_child.text().unwrap()),
                                                "ParaText" => {
                                                    if c_child.text().unwrap().starts_with("[") { continue; }
                                                    speech.push(' ');
                                                    
                                                    recursive_parse(&mut speech, c_child, &mut |sp, name| {new_speaker(connection, transcript_id, &mut speaker, sp, &mut start_time, &time, name, house_speaker)}, false);
                                                },
                                                "ProceduralText" => (),
                                                "FloorLanguage" => (),
                                                _ => println!("warning, found <{}> in <Content>", c_child.tag_name().name())
                                            };
                                        }
                                    },
                                    _ => println!("warning, found <{:?}> in <Intervention>", int_child)
                                };
                            }
                        }
                        _ => (),
                    };
                }
            }
        }
        
        post(connection, &speaker.id, &transcript_id, &mut speech, &start_time, &time);
    }
}

fn post(conn: &mut MysqlConnection, speaker: &i32, transcript: &i32, text: &mut String, start: &PrimitiveDateTime, end: &PrimitiveDateTime) {
    let text = text.trim();
    let clean_text = clean(text);
    let new_speech = NewSpeech { speaker, text: text, transcript, start, end };
    post_speech(conn, new_speech, clean_text);
}

fn clean(text: &str) -> String {
    let patterns = &[".", ",", ";", ":", "!", "?", "\'", "\"", "”", "“", "’", "‘", "(", ")", "[", "]", "{", "}", "«", "»"];
    let ac = AhoCorasick::builder().build(patterns).unwrap();
    let mut clean = String::new();
    ac.replace_all_with(text, &mut clean, |_, _, dst| {
        dst.push_str("");
        true
    });
    
    clean = Regex::new(r#"\s+"#).unwrap().replace_all(&clean, " ").to_lowercase().replace("—", " ");
    clean.insert(0, ' ');
    clean.push(' ');
    clean
}

fn recursive_parse<F: FnMut(&mut String, &str)>(speech: &mut String, node: roxmltree::Node, speaker_change: &mut F, tmp: bool) {
    let boring_tags = ["affiliation", "document", "quote", "quotepara", "poetry", "verse", "line", "i", "b", "", "sup", "sub", "committeequote", "query", "legislationquote", "forcecolumnbreak"];
    //let cancels = ["some hon. members:", "hon. members:", "some hon. member", "an hon. member:"];
    for child in node.children() {
        if child.is_text() {
            if tmp {
                print!("{}", child.text().unwrap());
            }
            speech.push_str(child.text().unwrap());
        }
        else {
            let tag = child.tag_name().name().to_lowercase();
            if let Some(text) = child.text() {
                if tag == "b" {
                    let tl = text.to_lowercase();
                    if tl.contains("hon. member") || tl.contains("member:") || tl.contains("members:") || tl.contains("voices:") {
                        speaker_change(speech, "");
                    }
                    else if !tl.contains("fluor corporation") {
                        speaker_change(speech, &text);
                    }
                }
            }
            if !boring_tags.contains(&tag.as_str()) {
                print!("<{}>", tag);
                recursive_parse(speech, child, speaker_change, true);
                println!("</{}>", tag);
            }
            else {
                recursive_parse(speech, child, speaker_change, false);
            }
        }
    }
}

fn new_speaker(connection: &mut MysqlConnection, transcript_id: i32, speaker: &mut Speaker, speech: &mut String, start_time: &mut PrimitiveDateTime, time: &PrimitiveDateTime, name: &str, house_speaker: &str) {
    if speaker.id != 0 {
        //println!("{} - {}: {} {} ({}): {}", start_time, time, speaker.first_name, speaker.last_name, speaker.id, speech.trim());
        post(connection, &speaker.id, &transcript_id, speech, start_time, time);
    }
     
    if name == "" {
        *speaker = Speaker::default();
    }
    else {
        //print!("{name}\t");
        *speaker = parse_speaker(connection, name, house_speaker);
        //println!("{} {}", speaker.first_name, speaker.last_name);
    }
    *speech = "".to_string();
    *start_time = *time;
}

fn parse_speaker(connection: &mut MysqlConnection, name: &str, house_speaker: &str) -> Speaker {
    let titles = r#"(Hon\.|L’hon\.|Mr\.?|Ms\.?|Mrs\.?|Mme\.?|M\.|Miss|Mister|Missus|Madame|Monsieur|MP|H\.E\.|Her Excellency|His Excellency|The Honourable)"#;
    let names = r#"([\w\.\-'‑]+)\s([\w\.\-'‑]+\s)?([\w\.\-'‑]+)"#;
    
    if name.to_lowercase().contains("ursula von der leyen") {
        return get_speaker_by_name(connection, "Ursula", "von der Leyen");
    }
    
    let name = name
        .replace("The Speaker", house_speaker)
        .replace("Mr. Speaker", house_speaker)
        .replace("Speaker Rota", "Anthony Rota")
        .replace("Robert Morrissey", "Bobby Morrissey")
        .replace("Robert Oliphant", "Rob Oliphant")
        .replace("Mr. Doherty", "Mr. Todd Doherty")
        .replace("Peter Julien", "Peter Julian")
        .replace("Steven McKinnon", "Steven MacKinnon")
        .replace("Gabriel-Marie-Marie", "Gabriel Ste-Marie")
        .replace("‑", "-");
    let title_reg = RegexBuilder::new(titles).case_insensitive(true).build().unwrap();
    let mut speaker = Speaker::default();
    if let Some(title) = title_reg.captures(&name) {
        let reg = RegexBuilder::new(&(titles.to_owned() + r#"\s+"# + names)).case_insensitive(true).build().unwrap();
        let Some(captures) = reg.captures(&name) else { panic!("{}", name) };
        speaker = get_speaker_by_name(connection, &captures[2], &captures[captures.len()-1]);
    }
    else {
        let name = name
            .replace("The Clerk of the House", "Charles Robert")
            .replace("The Assistant Deputy Chair", "Alexandra Mendès")
            .replace("The Assistant Deputy Speaker", "Alexandra Mendès")
            .replace("The Deputy Chair", "Carol Hughes")
            .replace("The Deputy Speaker", "Chris d'Entremont")
            .replace("The Chair", "Chris d'Entremont");
        let reg = Regex::new(names).unwrap();
        let Some(captures) = reg.captures(&name) else { panic!("{}", name) };
        speaker = get_speaker_by_name(connection, &captures[1], &captures[captures.len()-1]);
    }
    speaker
}

fn parse_time(date: Date, text: &str) -> PrimitiveDateTime {
    let mut hour = text[1..3].parse::<u8>().unwrap();
    let mut date = date;
    if hour >= 24 {
        hour -= 24;
        date = date.next_day().unwrap();
    }
    let new_time = PrimitiveDateTime::new(date, Time::from_hms(hour, text[3..5].parse().unwrap(), 0).unwrap());
    //println!("New time: {}", new_time);
    new_time
}
