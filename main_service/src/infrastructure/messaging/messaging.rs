use std::env::consts::ARCH;
use std::sync::Arc;
use amq_protocol::protocol::channel;
use lapin::Queue;
use tokio::sync::Mutex;

use async_trait::async_trait;
use thiserror::Error;
use lapin::{Connection, ConnectionProperties, options::*, types::FieldTable, BasicProperties, ExchangeKind};
use tokio_executor_trait::Tokio;
use crate::infrastructure::messaging::channel_pool::ChannelPool;
use crate::infrastructure::messaging::consumer::Consumer;

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
    async fn publish(&self, message: &String) -> Result<(), MessagingError>;
    async fn consume(&self, queue: &str) -> Result<(), MessagingError>;
    async fn create_exchange(&self, name: &str) -> Result<(), MessagingError>;
}

pub struct Messaging {
    channel_pool: ChannelPool,
    conn : Arc<Mutex<Connection>>,
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
        // acquiring channel from pool      
        let get_channel = self.channel_pool.borrow_channel(&self.conn, "create_exchange").await;
        let _ = get_channel.as_ref().map_err(|err| {err});
        let channel = get_channel.unwrap();

        let _ = channel
        .exchange_declare(
            &name,
            ExchangeKind::Topic,
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        )
        .await.map_err(|err| MessagingError::Other(err.to_string()));

        let _ = channel.queue_declare("my_queue", QueueDeclareOptions::default(), FieldTable::default()).await.map_err(|err| MessagingError::Other(err.to_string()));
        channel
        .queue_bind(
            "my_queue",
            &name,
            "my_routing_key", // Use a specific routing key
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await
        .map_err(MessagingError::LapinError)?;

        Ok(())
    }


     async fn publish(&self, message: &String) -> Result<(), MessagingError> {
        let channel = self.channel_pool.borrow_channel(&self.conn, "publish").await.unwrap();
        channel.clone().basic_publish(
                "logs_exchange",
                "my_routing_key",
                BasicPublishOptions::default(),
                message.as_bytes(),
                BasicProperties::default(),
            )
            .await
            .map_err(MessagingError::LapinError)?;
        self.channel_pool.return_channel(channel).await?;
        Ok(())
    }

    async fn consume(&self, queue: &str) -> Result<(), MessagingError> {
        let channel = self.conn.lock().await.create_channel().await.map_err(MessagingError::LapinError).unwrap();
        channel.basic_consume("my_queue", "test", BasicConsumeOptions::default(), FieldTable::default()).await?.set_delegate(Consumer{});     
        Ok(())
    }
}
