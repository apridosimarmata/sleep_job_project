use std::collections::HashMap;
use std::{sync::Arc, thread::sleep, time::Duration};
use tokio::sync::Mutex;

use crate::{
    domain::{
        model::job_model::UpdateJobStatusModel,
        repository::{job::JobRepository, repositories::RepositoriesWrapper},
        usecase::job::JobWorkerUsecase,
    },
    infrastructure::messaging::messaging::{Messaging, MessagingError, MessagingI},
};
use chrono::{DateTime, Utc};
use common_lib::constants::*;
use common_lib::error::Err;
use common_lib::message::message::{JobCreationRequest, JobUpdate};

pub struct JobWorkerUsecaseImpl {
    pub repositories: Arc<RepositoriesWrapper>,
    pub messaging: Arc<Messaging>,
    pub job_progesses: Mutex<HashMap<i64, i64>>,
}

pub async fn do_sleep(n: i32) {
    let mut handles = Vec::new();
    for _ in 0..n {
        let handle = tokio::spawn(async move {
            let _ = sleep(Duration::from_secs(2));
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }
}

#[async_trait::async_trait]
impl JobWorkerUsecase for JobWorkerUsecaseImpl {
    async fn consume_job_request(&mut self, req: JobCreationRequest) -> Result<(), MessagingError> {
        let mut progress_map = self.job_progesses.lock().await;
        let progress = match progress_map.get(&req.job_id) {
            Some(val) => *val,
            None => 0,
        };

        if progress == 0 {
            sleep(Duration::from_secs(2));
            let job = self
                .repositories
                .job_repository
                .get_job_by_id(req.job_id)
                .await
                .map_or_else(
                    |e| {
                        return Err(Err {
                            message: e.to_string(),
                        });
                    },
                    |res| Ok(res),
                );

            let job_unwrap = job.unwrap();

            /* notify main service, that the job is started */
            let req = JobUpdate {
                job_id: req.job_id,
                no_of_progress:0 as i32,
                status: JOB_PROCESSING.to_string(),
                no_of_sleep: job_unwrap.n,
                created_by: job_unwrap.email.clone(),
                created_at: job_unwrap.created_at,
                finished_at: 0,
                actual_time: 0,
            };
            let payload = serde_json::to_string(&req)
                .map_err(|err| return MessagingError::Other(err.to_string()))
                .unwrap();

            let _ = self
                .messaging
                .publish(&payload, JOBS_EXCHANGE, JOB_PROGRESS_TOPIC)
                .await
                .map_err(|err| dbg!(err))
                .unwrap();
            println!("notified progress");
        }

        do_sleep(req.n).await;
        println!("done");

        println!("{}", (progress + 10) as i32);
        let completed = (progress + 10) as i32 >= req.target;
        let notification_type = if completed {
            JOB_COMPLETED
        } else {
            JOB_PROGRESSING
        };

        let job = self
            .repositories
            .job_repository
            .get_job_by_id(req.job_id)
            .await
            .map_or_else(
                |e| {
                    return Err(Err {
                        message: e.to_string(),
                    });
                },
                |res| Ok(res),
            );

        let job_unwrap = job.unwrap();
        let mut tx = match self.repositories.job_repository.get_tx().await {
            Ok(tx) => tx,
            Err(err) => return Err(MessagingError::ConsumeError(err.message)),
        };

        let now_utc: DateTime<Utc> = Utc::now();
        let now_second = now_utc.timestamp();


        /* updating to completed */
        if completed{
            let _ = self
            .repositories
            .job_repository
            .update_job(
                &mut tx,
                UpdateJobStatusModel {
                    id: req.job_id,
                    finishes_at: now_second,
                    status: notification_type.to_string(),
                },
            )
            .await
            .map_or_else(|err| Err(MessagingError::ConsumeError(err.to_string())), Ok);

        let _ = tx
            .commit()
            .await
            .map_or_else(|err| Err(MessagingError::ConsumeError(err.to_string())), Ok);
        }

        /* notify main service */
        let update = JobUpdate {
            job_id: req.job_id,
            status: notification_type.to_string(),
            no_of_sleep:req.target,
            no_of_progress: progress as i32+req.n,
            created_by: job_unwrap.email,
            created_at: job_unwrap.created_at,
            finished_at: now_second,
            actual_time: now_second - job_unwrap.created_at,
        };
        let payload = serde_json::to_string(&update)
            .map_err(|err| return MessagingError::Other(err.to_string()))
            .unwrap();

        let _ = self
            .messaging
            .publish(&payload, JOBS_EXCHANGE, JOB_PROGRESS_TOPIC)
            .await
            .map_err(|err| dbg!(err))
            .unwrap();

        progress_map.insert(req.job_id, progress + req.n as i64);
        drop(progress_map);

        Ok(())
    }
}
