use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::fmt;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct CountRequest {
    pub name: String,
}

#[derive(Clone, PartialEq)]
pub enum BreakdownType {
    Party,
    Gender,
}

impl FromStr for BreakdownType {
    type Err = ();
    fn from_str(input: &str) -> Result<BreakdownType, Self::Err> {
        match input.to_lowercase().as_str() {
            "party" => Ok(BreakdownType::Party),
            "gender" => Ok(BreakdownType::Gender),
            _ => Err(())
        }
    }
}

impl fmt::Display for BreakdownType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BreakdownType::Party => write!(f, "party"),
            BreakdownType::Gender => write!(f, "gender"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct BreakdownResponse {
    pub name: String,
    pub colour: String,
    pub count: i32,
    pub score: f32,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct SpeakerResponse {
    pub first_name: String,
    pub last_name: String,
    pub colour: String,
    pub count: i64
}
