use common_lib::message::message::JobCreationRequest;

use crate::domain::repository::repositories::RepositoriesWrapper;
use crate::infrastructure::messaging::messaging::MessagingError;
use crate::domain::dto::job_dto::*;

#[async_trait::async_trait]
pub trait JobWorkerUsecase {
    async fn consume_job_request(&self, req:JobCreationRequest) -> Result<(), MessagingError>;
}
