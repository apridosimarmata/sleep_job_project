

use std::{future, ops::Deref, pin::Pin, sync::Arc};

use lapin::{message::DeliveryResult, options::{BasicAckOptions, BasicConsumeOptions}, types::FieldTable, Connection, ConsumerDelegate};
use crate::{app::job::usecase::job::JobWorkerUsecaseImpl, domain::usecase::job::JobWorkerUsecase, infrastructure::messaging::messaging::MessagingError};
use common_lib::message::message::JobCreationRequest;
pub trait JobMessagingHandler  {
    async fn listen(&self) ->  Result<(), MessagingError>;
}

pub struct JobMessagingHandlerImpl<'conn> {
    conn: &'conn Connection,
    job_worker_usecase: Arc<JobWorkerUsecaseImpl>,
}

impl<'conn> JobMessagingHandlerImpl <'conn>{
    pub fn new(conn: &'conn Connection, usecase: Arc<JobWorkerUsecaseImpl>) -> Self {
        JobMessagingHandlerImpl { conn: conn, job_worker_usecase: usecase }
    }
}

impl <'conn> JobMessagingHandler for JobMessagingHandlerImpl<'conn> {
    async fn listen(&self) -> Result<(), MessagingError> {
        let channel = self.conn.create_channel().await.map_err(MessagingError::LapinError).unwrap();
        let res = channel.basic_consume(
            "jobs_queue",
            "test",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await.map_err(|err| {
            return MessagingError::LapinError(err)
        });

        let c = res.unwrap();
        let delegation = JobConsumer{
            usecase: self.job_worker_usecase.clone()
        };
        c.set_delegate(delegation);

        Ok(())
    }
}


pub struct JobConsumer{
    usecase:Arc<JobWorkerUsecaseImpl>,
}

#[async_trait::async_trait]
impl <'a> ConsumerDelegate for JobConsumer{
    fn on_new_delivery(&self, delivery_result: DeliveryResult) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        let usecase_clone = self.usecase.clone();
        Box::pin(async move {
            match delivery_result {
                Ok(Some(delivery)) => {
                    let payload = String::from_utf8_lossy(&delivery.data);
                    let req:Result<JobCreationRequest, serde_json::Error> = serde_json::from_str(format!("{}", payload).as_str());
                    
                    usecase_clone.consume_job_request(req.unwrap()).await;
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