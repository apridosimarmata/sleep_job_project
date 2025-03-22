use std::sync::{Arc};

use common_lib::message::message::JobCreationRequest;
use tokio::sync::{Mutex};

use crate::{domain::{repository::repositories::RepositoriesWrapper, usecase::job::JobWorkerUsecase}, infrastructure::messaging::{consumer::WorkerUsecase, messaging::MessagingError}};

#[derive(Clone)]
pub struct JobWorkerUsecaseImpl{
    pub repositories: Arc<Mutex<RepositoriesWrapper>>,
}

#[async_trait::async_trait]
impl JobWorkerUsecase for JobWorkerUsecaseImpl {
    async fn consume_job_request(&self, req:JobCreationRequest) -> Result<(), MessagingError>{
        dbg!(req);
        Ok(())
    }
}