use std::{sync::Arc, thread::sleep, time::{ Duration, SystemTime, UNIX_EPOCH}};

use chrono::{DateTime, Utc};
use common_lib::message::message::{JobCreationRequest, JobUpdate};
use tokio::sync::Mutex;

use crate::{domain::{model::job_model::UpdateJobStatusModel, repository::{job::JobRepository, repositories::RepositoriesWrapper}, usecase::job::JobWorkerUsecase}, infrastructure::messaging::messaging::{Messaging, MessagingError, MessagingI}};

#[derive(Clone)]
pub struct JobWorkerUsecaseImpl{
    pub repositories: Arc<RepositoriesWrapper>,
    pub messaging: Arc<Messaging>
}

#[async_trait::async_trait]
impl JobWorkerUsecase for JobWorkerUsecaseImpl {
    async fn consume_job_request(&self, req:JobCreationRequest) -> Result<(), MessagingError>{
    let mut handles = Vec::new();

    for _ in 0..req.n {
        let handle = tokio::spawn(async move {
            let _= sleep(Duration::from_secs(2));
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let mut tx = match self.repositories.job_repository.get_tx().await {
        Ok(tx) => tx,
        Err(err) => return Err(MessagingError::ConsumeError(err.message)),
    };


    let now_utc: DateTime<Utc> = Utc::now();
    let now_second = now_utc.timestamp();

    let _ = self.repositories.job_repository.update_job(&mut tx, UpdateJobStatusModel{
        id: req.job_id,
        finishes_at: now_second,
        status: "COMPLETED".to_string(),
    }).await.map_or_else(|err|{
        Err(MessagingError::ConsumeError(err.to_string()))
    }, Ok);

    let _ = tx.commit().await.map_or_else(|err|{
        Err(MessagingError::ConsumeError(err.to_string()))
    }, Ok);

    dbg!(1);
    /* notify main service */
    let req = JobUpdate{
        job_id: req.job_id,
        status: &"completed"
    };
    let payload = serde_json::to_string(&req).map_err(|err| {
        return MessagingError::Other(err.to_string())
    }).unwrap();

    dbg!(req);

    let res = self.messaging.publish(&payload, "jobs_exchange", "jobs.status").await.map_err(|err|{
        dbg!(err)
    }).unwrap();

    Ok(())

    }
}