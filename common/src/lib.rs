use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::fmt;
use time::PrimitiveDateTime;


#[derive(Clone, PartialEq)]
pub enum BreakdownType {
    Party,
    Gender,
    Speaker,
}

impl FromStr for BreakdownType {
    type Err = ();
    fn from_str(input: &str) -> Result<BreakdownType, Self::Err> {
        match input.to_lowercase().as_str() {
            "party" => Ok(BreakdownType::Party),
            "gender" => Ok(BreakdownType::Gender),
            "speaker" => Ok(BreakdownType::Speaker),
            _ => Err(())
        }
    }
}

impl fmt::Display for BreakdownType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BreakdownType::Party => write!(f, "party"),
            BreakdownType::Gender => write!(f, "gender"),
            BreakdownType::Speaker => write!(f, "speaker"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct DataRequest {
    pub search: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct BreakdownResponse {
    pub id: i32,
    pub name: String,
    pub colour: String,
    pub count: i32,
    pub score: f32,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct SpeechResponse {
    pub text: String,
    pub link: String,
    pub start: PrimitiveDateTime,
    pub end: PrimitiveDateTime,
}
