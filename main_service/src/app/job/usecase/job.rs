use common_lib::http_response::HTTPResponder;

use crate::domain::dto::job_dto::*;

use crate::domain::model::job_model::JobModel;
use crate::domain::usecase::job::{JobUsecaseImpl, JobUsecase};
use crate::domain::repository::job::JobRepository;
use crate::infrastructure::messaging::messaging::MessagingI;

impl JobUsecase for JobUsecaseImpl {
    async fn create_job(&self, req: JobRequestDTO) -> HTTPResponder<JobResponseDTO> {
        let mut tx = match self.repositories.job_repository.get_tx().await {
            Ok(tx) => tx,
            Err(err) => return HTTPResponder::BadRequest(err.message),
        };

        let job_id = match self.repositories.job_repository.create_job(
            &mut tx,
            JobModel {
                email: req.email.clone(),
                n: req.n,
                id: 0,
                status: "PENDING".to_string(),
                created_at: 1,
                finishes_at: 1,
            },
        ).await {
            Ok(id) => id,
            Err(err) => return HTTPResponder::InternalServerError(err.message),
        };

        // publish job
        let result = self.messaging.publish("from_usecase", "jobs_exchange", "jobs.created")
        .await.map_err(|err| err.to_string());
        match  result {
            Ok(_) => {},
            Err(err) => return HTTPResponder::InternalServerError(err), 
            
        }

        let _ = match self.repositories.job_repository.update_job(
            &mut tx,
            JobModel {
                email: req.email,
                n: req.n,
                id: job_id,
                status: "PROCESSING".to_string(),
                created_at: 1,
                finishes_at: 1,
            },
        ).await {
            Ok(id) => id,
            Err(err) => return HTTPResponder::InternalServerError(err.message),
        };

        match tx.commit().await {
            Ok(_) => HTTPResponder::Ok(JobResponseDTO {
                message: Some(job_id.to_string()),
                success: true,
            }),
            Err(err) => HTTPResponder::BadRequest(err.to_string()),
        }

    }
}
