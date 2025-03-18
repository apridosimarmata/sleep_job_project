use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct JobRequestMessage {
    pub message: String,
    pub code: String,
}