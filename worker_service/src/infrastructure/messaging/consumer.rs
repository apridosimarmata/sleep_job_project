use super::messaging::MessagingError;

#[async_trait::async_trait]
pub trait WorkerUsecase: Send + Sync {
    async fn consume(&self, msg: &str) -> Result<(), MessagingError>;
}