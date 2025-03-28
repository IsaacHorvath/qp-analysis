use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::fmt;
use time::PrimitiveDateTime;

#[derive(Clone, PartialEq)]
pub enum BreakdownType {
    Party,
    Gender,
    Province,
    Speaker,
}

pub struct BreakdownTypeParseError;

impl FromStr for BreakdownType {
    type Err = BreakdownTypeParseError;
    fn from_str(input: &str) -> Result<BreakdownType, Self::Err> {
        match input.to_lowercase().as_str() {
            "party" => Ok(BreakdownType::Party),
            "gender" => Ok(BreakdownType::Gender),
            "province" => Ok(BreakdownType::Province),
            "speaker" => Ok(BreakdownType::Speaker),
            _ => Err(BreakdownTypeParseError)
        }
    }
}

impl fmt::Display for BreakdownType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BreakdownType::Party => write!(f, "party"),
            BreakdownType::Gender => write!(f, "gender"),
            BreakdownType::Province => write!(f, "province"),
            BreakdownType::Speaker => write!(f, "speaker"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct DataRequest {
    pub search: String,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Speaker {
    pub first_name: String,
    pub last_name: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct SpeakerResponse {
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
}

pub type SpeakerRow = (i32, String, String);

impl From<SpeakerRow> for SpeakerResponse {
    fn from(row: SpeakerRow) -> SpeakerResponse {
        SpeakerResponse {
            id: row.0,
            first_name: row.1,
            last_name: row.2,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct BreakdownResponse {
    pub id: i32,
    pub name: String,
    pub colour: String,
    pub count: i64,
    pub score: f64,
}

pub type BreakdownRow = (i32, String, String, Option<i64>, Option<f64>);

pub fn to_breakdown_response(row: BreakdownRow) -> Option<BreakdownResponse> {
    Some(BreakdownResponse {
        id: row.0,
        name: row.1,
        colour: row.2,
        count: if row.3? > 0 { row.3? } else { None? },
        score: row.4?
    })
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct PopulationResponse {
    pub id: i32,
    pub name: String,
    pub population: i32,
    pub area: f64,
    pub colour: String,
    pub count: i64,
    pub score: f64,
}

pub type PopulationRow = (i32, String, i32, f64, String, Option<i64>, Option<f64>);

pub fn to_population_response(row: PopulationRow) -> Option<PopulationResponse> {
    Some(PopulationResponse {
        id: row.0,
        name: row.1,
        population: row.2,
        area: row.3,
        colour: row.4,
        count: row.5?,
        score: row.6?,
    })
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct SpeechResponse {
    pub speaker: i32,
    pub text: String,
    pub link: String,
    pub start: PrimitiveDateTime,
    pub end: PrimitiveDateTime,
}

pub type SpeechRow = (i32, String, String, PrimitiveDateTime, PrimitiveDateTime);

impl From<SpeechRow> for SpeechResponse {
    fn from(row: SpeechRow) -> SpeechResponse {
        SpeechResponse {
            speaker: row.0,
            text: row.1,
            link: row.2,
            start: row.3,
            end: row.4,
        }
    }
}
