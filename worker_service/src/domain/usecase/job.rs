use crate::domain::repository::repositories::RepositoriesWrapper;
use common_lib::http_response::HTTPResponder;
use crate::domain::dto::job_dto::*;

pub trait JobWorkerUsecase {
    async fn consume_job_request(&self, req:JobRequestDTO) -> HTTPResponder<JobResponseDTO>;
}

#[derive(Clone)]
pub struct JobWorkerUsecaseImpl{
    pub repositories: RepositoriesWrapper,
}