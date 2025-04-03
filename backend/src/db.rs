use crate::error::AppError;
use common::models::*;
use db::gender::dsl::{
    colour as gender_colour, gender, id as gender_id, name as gender_name,
    total_words as gender_total_words,
};
use db::party::dsl::{
    colour as party_colour, id as party_id, name as party_name, party,
    total_words as party_total_words,
};
use db::province::dsl::{
    colour as province_colour, id as province_id, name as province_name, province,
    total_words as province_total_words,
};
use db::riding::dsl::{area, name as riding_name, population, riding};
use db::speaker::dsl::{
    first_name, id as speaker_id, last_name, speaker, total_words as speaker_total_words,
};
use db::speech::dsl::{end, speaker as speech_speaker, speech, start, text};
use db::speech_clean::dsl::{speech_clean, text as clean_text};
use db::transcript::dsl::{link, transcript};
use db::{concat, count_words, score};
use diesel::dsl::sum;
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::Integer;
use diesel_async::{
    pooled_connection::{bb8::Pool, AsyncDieselConnectionManager},
    AsyncMysqlConnection, RunQueryDsl,
};
use dotenvy::dotenv;
use std::env;

#[derive(QueryableByName)]
struct ConnectionId {
    #[diesel(sql_type = Integer)]
    id: i32
}

/// Returns an initialized bb8 async connection pool for the database.
///
/// The connection pool will retain 10 idle connections and will open at max
/// 50 connections.
///
/// Panics on error, since there's no point in trying to run without database
/// connections.

pub async fn get_connection_pool() -> Pool<AsyncMysqlConnection> {
    dotenv().ok();

    let database_url = format!(
        "{}{}",
        env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
        env::var("DATA_SOURCE").expect("DATA_SOURCE must be set")
    );
    let manager = AsyncDieselConnectionManager::<AsyncMysqlConnection>::new(database_url);

    Pool::builder()
        .max_size(50) // todo find a sane default number here
        .min_idle(Some(10)) // todo same
        .build(manager)
        .await
        .expect("Could not build connection pool")
}

/// Returns the MariaDB connection id, as an i32, that corresponds to the connection
/// parameter. See the [mariadb CONNECTION_ID docs](https://mariadb.com/kb/en/connection_id/).

pub async fn get_connection_id(connection: &mut AsyncMysqlConnection) -> Result<i32, AppError> {
    Ok(sql_query("SELECT CONNECTION_ID() AS id;")
        .load::<ConnectionId>(connection)
        .await?
        .pop()
        .ok_or(AppError::GenericError)?
        .id)
}

/// Kills the query associated with the gived connection id. See the
/// [mariadb KILL docs](https://mariadb.com/kb/en/kill/).

pub async fn kill_connection_id(connection: &mut AsyncMysqlConnection, id: &i32) -> Result<(), AppError> {
    let _ = sql_query(format!("KILL QUERY {};", id))
        .execute(connection)
        .await?;
        
    Ok(())
}

/// Returns all the speakers in the database, as SpeakerResponse objects.

pub async fn get_speakers(connection: &mut AsyncMysqlConnection) -> Result<Vec<SpeakerResponse>, AppError> {
    Ok(speaker
        .select((speaker_id, first_name, last_name))
        .load::<(i32, String, String)>(connection)
        .await?
        .into_iter()
        .map(|row| row.into())
        .collect())
}

/// Returns breakdown data of the given type (party, gender, etc.) for the given word.
///
/// The parameters are:
/// - connnection: the async MySQL connection
/// - breakdown_type: the breakdown type  
/// - word: the word to search for
///
/// The return type is BreakdownResponse, which contains the id of the breakdown (e.g. party
/// id, gender id,  etc.), the breakdown name, the breakdown colour, the sum of all times
/// that the word is mentioned for that breakdown, and that sum adjusted to the number of
/// words spoken in total.

pub async fn get_breakdown_word_count(
    connection: &mut AsyncMysqlConnection,
    breakdown_type: BreakdownType,
    word: &str,
) -> Result<Vec<BreakdownResponse>, AppError> {
    let loaded = match breakdown_type {
        BreakdownType::Party => speech
            .filter(party_total_words.gt(0))
            .inner_join(speaker.inner_join(party))
            .group_by((party_id, party_name, party_colour, party_total_words))
            .select((
                party_id,
                party_name,
                party_colour,
                sum(count_words(text, word)),
                score(party_total_words, sum(count_words(text, word))),
            ))
            .load::<BreakdownRow>(connection),
        BreakdownType::Gender => speech
            .filter(gender_total_words.gt(0))
            .inner_join(speaker.inner_join(gender))
            .group_by((gender_id, gender_name, gender_colour, gender_total_words))
            .select((
                gender_id,
                gender_name,
                gender_colour,
                sum(count_words(text, word)),
                score(gender_total_words, sum(count_words(text, word))),
            ))
            .load::<BreakdownRow>(connection),
        BreakdownType::Province => speech
            .filter(province_total_words.gt(0))
            .inner_join(speaker.inner_join(province))
            .group_by((
                province_id,
                province_name,
                province_colour,
                province_total_words,
            ))
            .select((
                province_id,
                province_name,
                province_colour,
                sum(count_words(text, word)),
                score(province_total_words, sum(count_words(text, word))),
            ))
            .load::<BreakdownRow>(connection),
        BreakdownType::Speaker => speech
            .filter(speaker_total_words.gt(0))
            .inner_join(speaker.inner_join(party))
            .group_by((
                speaker_id,
                first_name,
                last_name,
                party_colour,
                speaker_total_words,
            ))
            .select((
                speaker_id,
                concat(first_name, " ", last_name),
                party_colour,
                sum(count_words(text, word)),
                score(speaker_total_words, sum(count_words(text, word))),
            ))
            .order(sum(count_words(text, word)).desc())
            .limit(10)
            .load::<BreakdownRow>(connection),
    };

    Ok(loaded
        .await?
        .into_iter()
        .filter_map(|row| to_breakdown_response(row))
        .collect())
}

/// Returns riding population density data for the given word.
///
/// The return type is PopulationResponse, which contains the id of the speaker, the
/// riding name, the riding population, the riding area, the party colour, the sum of
/// all times that the word is mentioned for that breakdown, and that sum adjusted to
/// the number of words spoken in total.

pub async fn get_population_word_count(
    connection: &mut AsyncMysqlConnection,
    word: &str,
) -> Result<Vec<PopulationResponse>, AppError> {
    Ok(speech
        .filter(speaker_total_words.gt(0))
        .inner_join(speaker.inner_join(party).inner_join(riding))
        .group_by((
            speaker_id,
            riding_name,
            population,
            area,
            party_colour,
            speaker_total_words,
        ))
        .select((
            speaker_id,
            riding_name,
            population,
            area,
            party_colour,
            sum(count_words(text, word)),
            score(speaker_total_words, sum(count_words(text, word))),
        ))
        .load::<PopulationRow>(connection)
        .await?
        .into_iter()
        .filter_map(|row| to_population_response(row))
        .collect())
}

/// Returns all speeches matching the breakdown that contain the requested word.
///
/// The parameters are:
/// - connnection: the async MySQL connection
/// - breakdown_type: the breakdown type
/// - id: the breakdown id
/// - word: the word to search for
///
/// The return type is SpeechResponse, which contains the speaker id, the text of the
/// speech, a link to the original transcript, the start time, and the end time.

pub async fn get_speeches(
    connection: &mut AsyncMysqlConnection,
    breakdown_type: BreakdownType,
    id: i32,
    word: &str,
) -> Result<Vec<SpeechResponse>, AppError> {
    let loaded = match breakdown_type {
        BreakdownType::Party => speech
            .inner_join(speech_clean)
            .inner_join(speaker.inner_join(party))
            .filter(party_id.eq(id).and(clean_text.like(format!("%{}%", word))))
            .inner_join(transcript)
            .select((speech_speaker, text, link, start, end))
            .limit(100)
            .load::<SpeechRow>(connection),
        BreakdownType::Gender => speech
            .inner_join(speech_clean)
            .inner_join(speaker.inner_join(gender))
            .filter(gender_id.eq(id).and(clean_text.like(format!("%{}%", word))))
            .inner_join(transcript)
            .select((speech_speaker, text, link, start, end))
            .limit(100)
            .load::<SpeechRow>(connection),
        BreakdownType::Province => speech
            .inner_join(speech_clean)
            .inner_join(speaker.inner_join(province))
            .filter(
                province_id
                    .eq(id)
                    .and(clean_text.like(format!("%{}%", word))),
            )
            .inner_join(transcript)
            .select((speech_speaker, text, link, start, end))
            .limit(100)
            .load::<SpeechRow>(connection),
        BreakdownType::Speaker => speech
            .inner_join(speech_clean)
            .filter(
                speech_speaker
                    .eq(id)
                    .and(clean_text.like(format!("%{}%", word))),
            )
            .inner_join(transcript)
            .select((speech_speaker, text, link, start, end))
            .limit(100)
            .load::<SpeechRow>(connection),
    };

    Ok(loaded
        .await?
        .into_iter()
        .map(|row| row.into())
        .collect())
}
