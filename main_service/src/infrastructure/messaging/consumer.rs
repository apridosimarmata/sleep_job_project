use lapin::ConsumerDelegate;
use lapin::{
    Queue, options::BasicAckOptions
};
use tokio_amqp::*;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use std::future;
use std::pin::Pin;
use std::sync::Arc;
use lapin::message::{Delivery, DeliveryResult};

pub struct Consumer {
}

#[async_trait::async_trait]
impl ConsumerDelegate for Consumer {
    fn on_new_delivery(&self, delivery_result: DeliveryResult) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        match delivery_result {
            Ok(delivery) => {

                match delivery {
                    Some(d) => {
                        let payload = String::from_utf8_lossy(&d.data);
                        println!("Received message: {}", payload);
                        Box::pin(async move {
                            d.ack(BasicAckOptions::default()).await.expect("ack");
                        })
                    },
                    None => {Box::pin(future::ready(()))}
                }
            }
            Err(e) => {
                println!("Delivery error: {:?}", e);
                Box::pin(future::ready(())) //return a ready future.
            }
        }
    }

    fn drop_prefetched_messages(&self) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        Box::pin(async move {})
    }
}