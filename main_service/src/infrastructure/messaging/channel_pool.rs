use std::{borrow::Borrow, rc::Rc, sync::Arc};
use lapin::{Channel, Connection};
use crate::infrastructure::messaging::messaging::MessagingError;
use tokio::sync::{Mutex, Semaphore};


pub trait ChannelPoolI {
    async fn borrow_channel(&self, conn: &Arc<Mutex<Connection>>) -> Result<Channel, MessagingError>;
    async fn return_channel(&self, chan: Channel) -> Result<(), MessagingError>;
}

pub struct ChannelPool {
    channels : Arc<Mutex<Vec<Channel>>>,
    max_channel: usize,
    borrower_mutex: Semaphore,
    no_of_channels_created:Mutex<usize>,

}

impl ChannelPool {
    pub async fn new(no_of_channel: usize) -> Self {
        let borrower_mutex = Semaphore::new(1); 

        ChannelPool{
            max_channel: no_of_channel,
            borrower_mutex: borrower_mutex,
            no_of_channels_created: Mutex::new(0),
            channels: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl ChannelPoolI for ChannelPool {
    async fn borrow_channel(&self, conn: &Arc<Mutex<Connection>>) -> Result<Channel, MessagingError>{  
        let _permit = self.borrower_mutex.acquire().await;

        let mut channels_guard = self.channels.lock().await; // wait for channels lock
        let mut created_channel_count =  self.no_of_channels_created.lock().await; // get latest channel count

        if channels_guard.len() > 0 {
            let mut channel = channels_guard.pop().unwrap();
            if channel.status().closing(){
                channel = conn.clone().lock().await.create_channel().await.unwrap();
            }
            Ok(channel)
        }else if channels_guard.clone().len() < self.max_channel &&  *created_channel_count < self.max_channel {
            /* allows another thread to return channel */
            drop(channels_guard);
            
            let new_channel = conn.clone().lock().await.create_channel().await.unwrap();
            *created_channel_count +=1;
            Ok(new_channel)
        }else{
            /* allows another thread to return channel */
            drop(channels_guard);

            /* wait for channel availability */
            loop {
                let channels_guard = self.channels.lock().await;
                if channels_guard.len() > 0  {
                    break;
                }
            }

            /* take channel */
            let mut channels_guard = self.channels.lock().await;
            let mut channel = channels_guard.pop().unwrap();

            if channel.status().closing(){
                channel = conn.clone().lock().await.create_channel().await.unwrap();
            }
            Ok(channel)
        }
    }

    async fn return_channel(&self, chan: Channel) -> Result<(), MessagingError> {
        let mut channels_guard = self.channels.lock().await;
        channels_guard.push(chan);
        Ok(())
    }

}