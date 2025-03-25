use std::convert::Infallible;
use std::sync::Arc;

use crate::domain::misc::broadcast_channel::JobProggressBroadcaster;
use crate::domain::repository::repositories::RepositoriesWrapper;
use actix_web_lab::sse::Event;
use common_lib::error::Err;
use common_lib::http_response::HTTPResponder;
use common_lib::message::message::JobUpdate;
use tokio::sync::mpsc::{Receiver, Sender};
use crate::domain::dto::job_dto::*;
use crate::infrastructure::messaging::messaging::{Messaging, MessagingError};

pub trait JobUsecase <'a>{
    async fn create_job(&self, req:JobRequestDTO) -> HTTPResponder<JobResponseDTO>;
    async fn stream_jobs(&self, tx : &Sender< Result<Event, Infallible>>);
    async fn consume_job_update(&self, req: JobUpdate) -> Result<(), MessagingError>;
}

#[derive(Clone)]
pub struct JobUsecaseImpl{
    pub repositories: RepositoriesWrapper,
    pub messaging:  Arc<Messaging>,
    pub progress_channel: Arc<JobProggressBroadcaster>
}