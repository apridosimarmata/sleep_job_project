use actix_web::{HttpResponse, Responder};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct CustomHTTPResponse<T: Serialize> {
    pub data: T,
}

#[derive(Serialize, Deserialize)]
pub struct CustomHTTPError {
    pub error: String,
}

#[derive(Serialize, Deserialize)]
pub enum HTTPResponder<T: Serialize> {
    Ok(T),
    BadRequest(String),
}

impl<T: Serialize> Responder for HTTPResponder<T> {
    type Body = actix_web::body::BoxBody;

    fn respond_to(self, _req: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
        match self {
            HTTPResponder::Ok(data) => HttpResponse::Ok().json(CustomHTTPResponse { data }),
            HTTPResponder::BadRequest(msg) => HttpResponse::BadRequest().json(CustomHTTPError { error: msg }),
        }
    }
}
