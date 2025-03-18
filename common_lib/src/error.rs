use std::error::Error;
use std::fmt;
use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize)]
pub struct Err {
    pub message: String,
}

impl fmt::Display for Err {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Err: {}", self.message)
    }
}

impl Error for Err {}

#[derive(Serialize, Deserialize)]
pub struct HTTPError {
    pub error: String,
    pub code: String,
}

