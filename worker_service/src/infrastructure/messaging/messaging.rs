use std::sync::Arc;
use tokio::sync::Mutex;
use async_trait::async_trait;
use thiserror::Error;
use lapin::{Connection, ConnectionProperties, options::*, types::FieldTable, BasicProperties, ExchangeKind};
use tokio_executor_trait::Tokio;
use crate::infrastructure::messaging::channel_pool::ChannelPool;

use super::channel_pool::ChannelPoolI;

#[derive(Error, Debug, Clone)]
pub enum MessagingError {
    #[error("Connection error: {0}")]
    ConnectionError(String),
    #[error("Channel error: {0}")]
    ChannelError(String),   
    #[error("Publish error: {0}")]
    PublishError(String),
    #[error("Consume error: {0}")]
    ConsumeError(String),
    #[error("Other error: {0}")]
    Other(String),
    #[error(transparent)]
    LapinError(#[from] lapin::Error),
}

#[async_trait]
pub trait MessagingI {
    async fn publish(&self, message: &str, exchange_name: &str, routing_key: &str) -> Result<(), MessagingError>;
    async fn create_exchange(&self, name: &str) -> Result<(), MessagingError>;
    async fn create_queue(&self, name: &str, bind_to: &str, routing_key: &str ) -> Result<(), MessagingError>;
}

pub struct Messaging {
    channel_pool: ChannelPool,
    pub conn : Arc<Mutex<Connection>>,
}

impl Messaging {
    pub async fn new(connection_string: String) -> Self {
        let conn = Connection::connect(&connection_string, ConnectionProperties::with_executor(ConnectionProperties::default(), Tokio::current()))
            .await
            .map_err(MessagingError::LapinError);
        let channel_pool = ChannelPool::new(5).await;

        Messaging {
            conn: Arc::new(Mutex::new(conn.unwrap())),
            channel_pool: channel_pool,
        }
    }
}

#[async_trait]
impl MessagingI for Messaging {
    async fn create_exchange(&self, name: &str) -> Result<(), MessagingError> {  
        let channel: lapin::Channel = self
        .channel_pool
        .borrow_channel(&mut self.conn.clone()).await.map_err(|err| {err})?;
    
        let _ = channel
        .exchange_declare(
            &name,
            ExchangeKind::Topic,
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        )
        .await
        .map_err(MessagingError::LapinError)?;

        self.channel_pool.return_channel(channel).await?;
        Ok(())
    }

    async fn create_queue(&self, name: &str, bind_to: &str, routing_key: &str ) -> Result<(), MessagingError>{
        let channel: lapin::Channel = self.channel_pool.borrow_channel(&mut self.conn.clone()).await.map_err(|err| {err})?;
        let _ = channel.queue_declare(&name, QueueDeclareOptions::default(), FieldTable::default()).await.map_err(|err| {err});

        let _ =channel
        .queue_bind(
            &name,
            &bind_to,
            &routing_key,
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await
        .map_err(MessagingError::LapinError)?;

        self.channel_pool.return_channel(channel).await?;
        Ok(())
    }


     async fn publish(&self, message: &str, exchange_name: &str, routing_key: &str) -> Result<(), MessagingError> {
        let channel = self.channel_pool.borrow_channel(&self.conn).await.unwrap();
        channel.clone().basic_publish(
                &exchange_name,
                &routing_key,
                BasicPublishOptions::default(),
                message.as_bytes(),
                BasicProperties::default(),
            )
            .await
            .map_err(MessagingError::LapinError)?;
        
        self.channel_pool.return_channel(channel).await?;
        Ok(())
    }
}
