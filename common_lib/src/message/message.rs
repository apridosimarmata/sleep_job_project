use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug)]
pub struct JobCreationRequest {
    pub job_id : i64,
    pub n: i32
}