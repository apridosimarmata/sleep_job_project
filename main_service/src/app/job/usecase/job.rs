use common_lib::http_response::HTTPResponder;
use common_lib::message::message::JobCreationRequest;
use serde::Serialize;

use crate::domain::dto::job_dto::*;

use crate::domain::model::job_model::JobModel;
use crate::domain::usecase::job::{JobUsecaseImpl, JobUsecase};
use crate::domain::repository::job::JobRepository;
use crate::infrastructure::messaging::messaging::MessagingI;

impl JobUsecase for JobUsecaseImpl {
    async fn create_job(&self, req: JobRequestDTO) -> HTTPResponder<JobResponseDTO> {
        /* write the job to the database without committing it */
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
                status: "PROCESSING".to_string(),
                created_at: 1,
                finishes_at: 1,
            },
        ).await {
            Ok(id) => id,
            Err(err) => {
                let _ = tx.rollback().await;
                return HTTPResponder::InternalServerError(err.message)
            },
        };

        /* publish job request to worker services */
        let request = JobCreationRequest{
            job_id:job_id,
            n: req.n
        };
        if let Err(err) = self.messaging.publish(&serde_json::to_string(&request).unwrap(), "jobs_exchange", "jobs.created").await {
            let _ = tx.rollback().await;
            return HTTPResponder::InternalServerError(err.to_string());
        }

        /* commit job to db */
        match tx.commit().await {
            Ok(_) => HTTPResponder::Ok(JobResponseDTO {
                message: Some(job_id.to_string()),
                success: true,
            }),
            Err(err) => HTTPResponder::BadRequest(err.to_string()),
        }

    }
}
