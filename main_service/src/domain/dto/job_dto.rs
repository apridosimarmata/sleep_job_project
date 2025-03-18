use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct JobRequestDTO {
    pub n: i32,
    pub email: String,
}

#[derive(Serialize, Deserialize)]
pub struct JobResponseDTO {
    pub success: bool,
    pub message: Option<String>,
} 