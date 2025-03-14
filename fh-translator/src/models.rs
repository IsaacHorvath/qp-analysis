use diesel::prelude::*;
use time::PrimitiveDateTime;

#[derive(Queryable, Selectable, Default)]
#[diesel(table_name = db::speaker)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Speaker {
    pub id: i32,
    // pub first_name: String,
    // pub last_name: String,
    // pub party: i32,
    // pub age: i32,
    // pub gender: i32,
    // pub riding: i32,
}

#[derive(Insertable)]
#[diesel(table_name = db::speech)]
pub struct NewSpeech<'a> {
    pub speaker: &'a i32,
    pub transcript: &'a i32,
    pub text: &'a str,
    pub start: &'a PrimitiveDateTime,
    pub end: &'a PrimitiveDateTime
}

#[derive(Insertable)]
#[diesel(table_name = db::speech_clean)]
pub struct NewSpeechClean<'a> {
    pub speech: &'a i32,
    pub text: &'a str,
}

#[derive(Insertable)]
#[diesel(table_name = db::transcript)]
pub struct NewTranscript<'a> {
    pub link: &'a str,
}
