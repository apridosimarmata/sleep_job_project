use lapin::ConsumerDelegate;
use lapin::options::BasicAckOptions;
use std::future;
use std::pin::Pin;
use std::sync::{Arc};
use tokio::sync::Mutex;
use lapin::message::DeliveryResult;
use super::messaging::MessagingError;

#[async_trait::async_trait]
pub trait WorkerUsecase: Send + Sync {
    async fn consume(&self, msg: &str) -> Result<(), MessagingError>;
}