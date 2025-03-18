use crate::domain::repository::repositories::RepositoriesWrapper;
use common_lib::http_response::HTTPResponder;
use crate::domain::dto::job_dto::*;

pub trait JobUsecase {
    async fn create_job(&self, req:JobRequestDTO) -> HTTPResponder<JobResponseDTO>;
}

#[derive(Clone)] // ✅ Derive Clone instead of Copy
pub struct JobUsecaseImpl{
    pub repositories: RepositoriesWrapper,
}