use diesel::prelude::*;
use dotenvy::dotenv;
use std::env;
use crate::{Speaker, NewSpeech, NewSpeechClean, NewTranscript};
use db::speaker::dsl::*;
use db::speech;
use db::speech_clean;
use db::transcript;
use db::last_insert_id;

use std::io::stdin;

pub fn establish_connection() -> MysqlConnection {
    dotenv().ok();

    let database_url = format!("{}federal_house", env::var("DATABASE_URL").expect("DATABASE_URL must be set"));
    MysqlConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn get_speaker_by_name(connection: &mut MysqlConnection, first: &str, last: &str) -> Speaker {
    let mut results = speaker
        .filter(first_name.like("%".to_string() + first + "%"))
        .filter(last_name.like("%".to_string() + last + "%"))
        .select(Speaker::as_select())
        .load(connection)
        .expect("Error loading speakers");
    
    if results.len() < 1 {
        println!("{} {}: no speakers found", first, last);
        
        let mut s = String::new();
        stdin().read_line(&mut s).expect("Did not enter a correct string");
        
        results = speaker
            .filter(first_name.like("%".to_string() + first + "%"))
            .filter(last_name.like("%".to_string() + last + "%"))
            .select(Speaker::as_select())
            .load(connection)
            .expect("Error loading speakers");
        
        if results.len() < 1 { panic!("{} {}: no speakers found", first, last) };
    };
    if results.len() > 1 { panic!("{} {}: multiple speakers found", first, last) };
    
    results.pop().unwrap()
}

pub fn post_speech(conn: &mut MysqlConnection, new_speech: NewSpeech, clean_text: String) {
    conn.transaction(|conn| {
        diesel::insert_into(speech::table)
            .values(&new_speech)
            .execute(conn)
            .expect("couldn't post speech");
        
        let speech_id = speech::dsl::speech
            .filter(speech::dsl::id.eq(last_insert_id()))
            .select(speech::id)
            .load::<i32>(conn)
            .expect("couldn't select last speech insert id");
            
        diesel::insert_into(speech_clean::table)
            .values(NewSpeechClean { speech: &speech_id.first().unwrap(), text: &clean_text })
            .execute(conn)
            .expect("couldn't post clean speech");
            
        Ok::<(), diesel::result::Error>(())
    }).unwrap();
}

pub fn post_transcript(conn: &mut MysqlConnection, new_transcript: NewTranscript) -> i32 {
    conn.transaction(|conn| {
        diesel::insert_into(transcript::table)
            .values(&new_transcript)
            .execute(conn)
            .expect("couldn't post transcript");
        
        let res = transcript::dsl::transcript
            .filter(transcript::id.eq(last_insert_id()))
            .select(transcript::id)
            .load::<i32>(conn)
            .expect("couldn't select last transcript insert id");
            
        Ok::<i32, diesel::result::Error>(*res.first().unwrap())
    }).unwrap()
}
