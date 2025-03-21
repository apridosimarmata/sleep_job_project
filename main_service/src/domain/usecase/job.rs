use std::sync::Arc;

use crate::domain::repository::repositories::RepositoriesWrapper;
use common_lib::http_response::HTTPResponder;
use crate::domain::dto::job_dto::*;
use crate::infrastructure::messaging::messaging::Messaging;

pub trait JobUsecase {
    async fn create_job(&self, req:JobRequestDTO) -> HTTPResponder<JobResponseDTO>;
}

#[derive(Clone)]
pub struct JobUsecaseImpl{
    pub repositories: RepositoriesWrapper,
    pub messaging:  Arc<Messaging>,
}