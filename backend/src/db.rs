use diesel::prelude::*;
use diesel::dsl::sum;
use dotenvy::dotenv;
use std::env;
use time::PrimitiveDateTime;
use common::models::*;
use db::speech::dsl::{speech, speaker as speech_speaker, text, start, end};
use db::speech_clean::dsl::{speech_clean, text as clean_text};
use db::speaker::dsl::{speaker, id as speaker_id, first_name, last_name, total_words as speaker_total_words};
use db::party::dsl::{party, id as party_id, name as party_name, colour as party_colour, total_words as party_total_words};
use db::gender::dsl::{gender, id as gender_id, name as gender_name, colour as gender_colour, total_words as gender_total_words};
use db::province::dsl::{province, id as province_id, name as province_name, colour as province_colour, total_words as province_total_words};
use db::transcript::dsl::{transcript, link};
use db::{count_words, concat};

pub fn establish_connection() -> MysqlConnection {
    dotenv().ok();

    let database_url = format!("{}{}",
        env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
        env::var("DATA_SOURCE").expect("DATA_SOURCE must be set")
    );
    MysqlConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}


pub fn get_speakers(connection: &mut MysqlConnection) -> Vec<SpeakerResponse> {
    speaker
        .select((
            speaker_id,
            first_name,
            last_name,
        ))
        
        .load::<(i32, String, String)>(connection)    
        .expect(format!("error loading speakers").as_str())
        .into_iter()
        .map(|row| { SpeakerResponse {
            id: row.0,
            first_name: row.1,
            last_name: row.2,
        } })
        .collect()
}

pub fn get_breakdown_word_count(connection: &mut MysqlConnection, breakdown_type: BreakdownType, word: &str) -> Option<Vec<BreakdownResponse>> {
    if breakdown_type == BreakdownType::Province && env::var("DATA_SOURCE").unwrap() != "federal_house" {
        return None;
    }
    
    let loaded = match breakdown_type {
        BreakdownType::Party => speech
            .inner_join(speaker.inner_join(party))
            .group_by((party_id, party_name, party_colour, party_total_words))
            .select((
                party_id,
                party_name,
                party_colour,
                sum(count_words(text, word)),
                party_total_words,
            ))
            .load::<(i32, String, String, Option<i64>, i32)>(connection),
        BreakdownType::Gender => speech
            .inner_join(speaker.inner_join(gender))
            .group_by((gender_id, gender_name, gender_colour, gender_total_words))
            .select((
                gender_id,
                gender_name,
                gender_colour,
                sum(count_words(text, word)),
                gender_total_words,
            ))
            .load::<(i32, String, String, Option<i64>, i32)>(connection),
        BreakdownType::Province => speech
            .filter(province_total_words.gt(0))
            .inner_join(speaker.inner_join(province))
            .group_by((province_id, province_name, province_colour, province_total_words))
            .select((
                province_id,
                province_name,
                province_colour,
                sum(count_words(text, word)),
                province_total_words,
            ))
            .load::<(i32, String, String, Option<i64>, i32)>(connection),
        BreakdownType::Speaker => speech
            .filter(speaker_total_words.gt(0))
            .inner_join(speaker.inner_join(party))
            .group_by((speaker_id, first_name, last_name, party_colour, speaker_total_words))
            .select((
                speaker_id,
                concat(first_name, " ", last_name),
                party_colour,
                sum(count_words(text, word)),
                speaker_total_words,
            ))
            .load::<(i32, String, String, Option<i64>, i32)>(connection),
    };
    
    Some(loaded    
        .expect(format!("error loading {} word count", breakdown_type).as_str())
        .into_iter()
        .filter(|row| row.3.unwrap() > 0)
        .map(|row| { BreakdownResponse {
            id: row.0,
            name: row.1,
            colour: row.2,
            count: row.3.unwrap() as i32,
            score: 100000.0/(row.4 as f32)*(row.3.unwrap() as f32), // todo: this should be in sql
        } })
        .collect())
}

pub fn get_speeches(connection: &mut MysqlConnection, breakdown_type: BreakdownType, id: i32, word: &str) -> Vec<SpeechResponse> {
    let loaded = match breakdown_type {
        BreakdownType::Party => speech
            .inner_join(speech_clean)
            .inner_join(speaker.inner_join(party))
            .filter(party_id.eq(id).and(clean_text.like(format!("%{}%", word))))
            .inner_join(transcript)
            .select((
                speech_speaker,
                text,
                link,
                start,
                end,
            ))
            .limit(100)
            .load::<(i32, String, String, PrimitiveDateTime, PrimitiveDateTime)>(connection),
        BreakdownType::Gender => speech
            .inner_join(speech_clean)
            .inner_join(speaker.inner_join(gender))
            .filter(gender_id.eq(id).and(clean_text.like(format!("%{}%", word))))
            .inner_join(transcript)
            .select((
                speech_speaker,
                text,
                link,
                start,
                end,
            ))
            .limit(100)
            .load::<(i32, String, String, PrimitiveDateTime, PrimitiveDateTime)>(connection),
        BreakdownType::Province => speech
            .inner_join(speech_clean)
            .inner_join(speaker.inner_join(province))
            .filter(province_id.eq(id).and(clean_text.like(format!("%{}%", word))))
            .inner_join(transcript)
            .select((
                speech_speaker,
                text,
                link,
                start,
                end,
            ))
            .limit(100)
            .load::<(i32, String, String, PrimitiveDateTime, PrimitiveDateTime)>(connection),
        BreakdownType::Speaker => speech
            .inner_join(speech_clean)
            .filter(speech_speaker.eq(id).and(clean_text.like(format!("%{}%", word))))
            .inner_join(transcript)
            .select((
                speech_speaker,
                text,
                link,
                start,
                end,
            ))
            .limit(100)
            .load::<(i32, String, String, PrimitiveDateTime, PrimitiveDateTime)>(connection),
    };
    
    loaded
        .expect(format!("error loading speeches for {} {}, {}", breakdown_type, id, word).as_str())
        .into_iter()
        .map(|row| { SpeechResponse { // todo: this should be modelled select instead of mapping it, database types go into common
            speaker: row.0,
            text: row.1,
            link: row.2,
            start: row.3,
            end: row.4,
        } })
        .collect()
}
