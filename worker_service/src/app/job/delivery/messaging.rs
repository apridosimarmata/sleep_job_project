use std::{pin::Pin, sync::{Arc}};

use lapin::{message::DeliveryResult, options::BasicConsumeOptions, types::FieldTable, Connection, ConsumerDelegate};
use tokio::sync::Mutex;
use crate::{app::job::usecase::job::JobWorkerUsecaseImpl, domain::usecase::job::JobWorkerUsecase, infrastructure::messaging::messaging::MessagingError};
use common_lib::message::message::JobCreationRequest;


pub trait JobMessagingHandler  {
    async fn listen(&self) ->  Result<(), MessagingError>;
}

pub struct JobMessagingHandlerImpl {
    conn_arc: Arc<Mutex<Connection>>,
    job_worker_usecase: Arc<JobWorkerUsecaseImpl>,
}

impl JobMessagingHandlerImpl {
    pub fn new(conn_arc: Arc<Mutex<Connection>>, usecase: Arc<JobWorkerUsecaseImpl>) -> Self {
        JobMessagingHandlerImpl { conn_arc: conn_arc, job_worker_usecase: usecase}
    }
}

impl JobMessagingHandler for JobMessagingHandlerImpl {
    async fn listen(&self) -> Result<(), MessagingError> {
        let conn = self.conn_arc.lock().await;

        let channel = conn.create_channel().await.map_err(MessagingError::LapinError).unwrap();
        drop(conn);
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
                    dbg!(payload.clone());
                    let req:Result<JobCreationRequest, serde_json::Error> = serde_json::from_str(format!("{}", payload).as_str());

                    let result = usecase_clone.consume_job_request(req.unwrap()).await;
                    match result {
                        Ok(_) => delivery.ack(lapin::options::BasicAckOptions::default()).await.expect("ack"),
                        Err(_) => delivery.nack(lapin::options::BasicNackOptions::default()).await.expect("nack")
                    }                    
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