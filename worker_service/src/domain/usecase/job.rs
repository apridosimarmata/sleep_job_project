use common_lib::message::message::JobCreationRequest;

use crate::infrastructure::messaging::messaging::MessagingError;

#[async_trait::async_trait]
pub trait JobWorkerUsecase {
    async fn consume_job_request(&mut self, req:JobCreationRequest) -> Result<(), MessagingError>;
}
