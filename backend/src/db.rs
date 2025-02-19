use diesel::prelude::*;
use diesel::dsl::sum;
use dotenvy::dotenv;
use std::env;
use common::{BreakdownType, BreakdownResponse};
use crate::schema::speech_clean::dsl::{speech_clean, speaker as s_id, text};
use crate::schema::speaker::dsl::{speaker, first_name, last_name, total_words as speaker_total_words};
use crate::schema::party::dsl::{party, name as party_name, colour as party_colour, total_words as party_total_words};
use crate::schema::gender::dsl::{gender, name as gender_name, colour as gender_colour, total_words as gender_total_words};
use crate::schema::{count_words, concat};

pub fn establish_connection() -> MysqlConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    MysqlConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn get_breakdown_word_count(connection: &mut MysqlConnection, breakdown_type: BreakdownType, word: &str) -> Vec<BreakdownResponse> {
    let filtered = speech_clean.filter(s_id.ne(117));
    let loaded = match breakdown_type {
        BreakdownType::Party => filtered
            .inner_join(speaker.inner_join(party))
            .group_by((party_name, party_colour, party_total_words))
            .select((
                party_name,
                party_colour,
                sum(count_words(text, word)),
                party_total_words,
            ))
            .load::<(String, String, Option<i64>, i32)>(connection),
        BreakdownType::Gender => filtered
            .inner_join(speaker.inner_join(gender))
            .group_by((gender_name, gender_colour, gender_total_words))
            .select((
                gender_name,
                gender_colour,
                sum(count_words(text, word)),
                gender_total_words,
            ))
            .load::<(String, String, Option<i64>, i32)>(connection),
        BreakdownType::Speaker => filtered
            .inner_join(speaker.inner_join(party))
            .group_by((first_name, last_name, party_colour, speaker_total_words))
            .select((
                concat(first_name, " ", last_name),
                party_colour,
                sum(count_words(text, word)),
                speaker_total_words,
            ))
            .load::<(String, String, Option<i64>, i32)>(connection),
    };
    
    loaded    
        .expect(format!("error loading {} word count", breakdown_type).as_str())
        .into_iter()
        .map(|row| { BreakdownResponse {
            name: row.0,
            colour: row.1,
            count: row.2.unwrap() as i32,
            score: 100000.0/(row.3 as f32)*(row.2.unwrap() as f32), // should this be in sql?
        } })
        .collect()
}
