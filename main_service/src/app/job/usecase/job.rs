use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use actix_web_lab::sse::{self, Event};
use chrono::{DateTime, Utc};
use common_lib::http_response::HTTPResponder;
use common_lib::message::message::{JobCreationRequest, JobUpdate};
use tokio::sync::mpsc::Sender;
use crate::domain::dto::job_dto::*;
use crate::domain::model::job_model::JobModel;
use crate::domain::usecase::job::{JobUsecaseImpl, JobUsecase};
use crate::domain::repository::job::JobRepository;
use crate::infrastructure::messaging::messaging::{MessagingError, MessagingI};

impl <'a> JobUsecase<'a> for JobUsecaseImpl{
    async fn consume_job_update(&self, req: JobUpdate) -> Result<(), MessagingError>{
        println!("usecase {:?}", req);


        let _ =self.progress_channel.tx.send(JobUpdate { job_id: req.job_id, status: req.status });
        Ok(())
    }

    async fn stream_jobs(&self, tx : &Sender< Result<Event, Infallible>>){
        let clone = Arc::new(tx.clone());
        let mut progress_subscriber = self.progress_channel.rx.resubscribe();
        tokio::spawn( async move {
            loop {
              match progress_subscriber.recv().await {
                    Ok(val) => {
                        let data = Event::Data(
                            sse::Data::new(format!("job {} updated, status {}", val.job_id, val.status).as_str()),
                        );
                        let x = Ok::<_, Infallible>(data);
                        match clone.send(x).await {
                            Ok(_) => {},
                            Err(_) =>{}
                        } ;
                    },
                    Err(err) => {
                        println!("got error on job progress subscriber: {}", err.to_string())
                    }
                };
                 
                tokio::time::sleep(Duration::from_secs(2)).await;

            }
        });
    }

    async fn create_job(&self, req: JobRequestDTO) -> HTTPResponder<JobResponseDTO> {
        /* write the job to the database without committing it */
        let mut tx = match self.repositories.job_repository.get_tx().await {
            Ok(tx) => tx,
            Err(err) => return HTTPResponder::BadRequest(err.message),
        };

        let now_utc: DateTime<Utc> = Utc::now();
        let now_second = now_utc.timestamp();

        let job_id = match self.repositories.job_repository.create_job(
            &mut tx,
            JobModel {
                email: req.email.clone(),
                n: req.n,
                id: 0,
                status: "PROCESSING".to_string(),
                created_at: now_second,
                finishes_at: 0,
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
        if let Err(err) = self.messaging.publish(&serde_json::to_string(&request).unwrap(), "jobs_exchange", "jobs.create").await {
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
