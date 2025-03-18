use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct JobModel {
    pub id: i64,
    pub n: i32,
    pub email: String,
    pub created_at: i64,
    pub status: String,
    pub finishes_at: i64,
}