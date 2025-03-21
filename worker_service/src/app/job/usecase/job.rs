use common_lib::http_response::HTTPResponder;

use crate::domain::dto::job_dto::*;

use crate::domain::model::job_model::JobModel;
use crate::domain::usecase::job::{JobWorkerUsecaseImpl, JobWorkerUsecase};
use crate::domain::repository::job::JobRepository;

impl JobWorkerUsecase for JobWorkerUsecaseImpl {
    async fn consume_job_request(&self, req:JobRequestDTO) -> HTTPResponder<JobResponseDTO>{
        HTTPResponder::Ok(JobResponseDTO { success: true, message: Some("ok".to_string()) })
    }

}
