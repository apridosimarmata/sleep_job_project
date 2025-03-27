use std::{pin::Pin, sync::Arc};

use common_lib::{constants::{JOB_PROGRESS_QUEUE, JOB_PROGRESS_TOPIC}, message::message::JobUpdate};
use lapin::{message::DeliveryResult, options::BasicConsumeOptions, types::FieldTable, Connection, ConsumerDelegate};
use tokio::sync::Mutex;

use crate::{domain::usecase::{job::{JobUsecase, JobUsecaseImpl}, usecases::UsecasesWrapper}, infrastructure::messaging::messaging::MessagingError};

pub trait JobMessagingHandler  {
    async fn listen(&self) -> Result<(), MessagingError>;
}

pub struct JobMessagingHandlerImpl {
    conn_arc: Arc<Mutex<Connection>>,
    usecases:Arc<UsecasesWrapper>,
}

impl JobMessagingHandlerImpl {
    pub fn new(conn_arc: Arc<Mutex<Connection>>, usecases :Arc<UsecasesWrapper>) -> Self {
        JobMessagingHandlerImpl { conn_arc: conn_arc, usecases:usecases}
    }
}

impl  JobMessagingHandler for JobMessagingHandlerImpl {
    async fn listen(&self) -> Result<(), MessagingError> {
        let conn = self.conn_arc.lock().await;

        let channel = conn.create_channel().await.map_err(MessagingError::LapinError).unwrap();
        drop(conn);
        let res = channel.basic_consume(
            JOB_PROGRESS_QUEUE,
            "test",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await.map_err(|err| {
            return MessagingError::LapinError(err)
        });

        let c = res.unwrap();
        let delegation = JobConsumer{
            usecase: Arc::new(self.usecases.job_usecases.clone())
        };
        c.set_delegate(delegation);

        Ok(())
    }
}

pub struct JobConsumer {
    usecase:Arc<JobUsecaseImpl>,
}

#[async_trait::async_trait]
impl  ConsumerDelegate for JobConsumer{
    fn on_new_delivery(&self, delivery_result: DeliveryResult) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        let usecase_clone = self.usecase.clone();

        Box::pin(async move {
            match delivery_result {
                Ok(Some(delivery)) => {
                    let payload = String::from_utf8_lossy(&delivery.data);
                    let string_payload = format!("{}", payload);
                    let req:Result<JobUpdate, serde_json::Error> = serde_json::from_str(string_payload.as_str());
                    let x = req.is_err();

                    let _ = usecase_clone.consume_job_update(req.unwrap()).await;
                    delivery.ack(lapin::options::BasicAckOptions::default()).await.expect("ack");                 
                }
                Ok(None) => println!("Consumer ended"),
                Err(e) => eprintln!("Delivery error: {}", e),
            }
        })
    }

    fn drop_prefetched_messages(&self) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        Box::pin(async move {})
    }
}