use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug)]
pub struct JobCreationRequest {
    pub job_id : i64,
    pub n: i32,
    pub target: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JobUpdate {
    pub job_id : i64,
    pub no_of_sleep: i32,
    pub no_of_progress:i32,
    pub status : String,
    pub created_by: String,
    pub created_at: i64,
    pub finished_at: i64,
    pub actual_time: i64,
}