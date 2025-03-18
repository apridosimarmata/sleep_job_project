use std::sync::Arc;
use lapin::{Channel, Connection};
use crate::infrastructure::messaging::messaging::MessagingError;
use tokio::sync::{Mutex, Semaphore};


pub trait ChannelPoolI {
    async fn borrow_channel(&self, conn: &Arc<Mutex<Connection>>, ivkr: &str) -> Result<Channel, MessagingError>;
    async fn return_channel(&self, chan: Channel) -> Result<(), MessagingError>;
}

pub struct ChannelPool {
    channels : Box<Mutex<Vec<Channel>>>,
    max_channel: usize,
    semaphore: Semaphore,
}

impl ChannelPool {
    pub async fn new(no_of_channel: usize) -> Self {
        let semaphore = Semaphore::new(no_of_channel); 
        ChannelPool{
            channels: Box::new(Mutex::new(Vec::new())),
            max_channel: no_of_channel,
            semaphore: semaphore,
        }
    }
}

impl ChannelPoolI for ChannelPool {
    async fn borrow_channel(&self, conn: &Arc<Mutex<Connection>>, ivkr: &str) -> Result<Channel, MessagingError>{
        // this semaphore makes sure there are only max n threads are using n channel
        // where n is max no. of channel.
        let _ = self.semaphore.acquire().await.unwrap();
        let mut chan = self.channels.lock().await;

        if let Some(channel) = chan.pop(){
            Ok(channel)
        }else if chan.len() < self.max_channel {
            println!("creating channel: {}", ivkr);
            // drop chan to allow other thread to modify it.
            drop(chan);
            // when channel is not max yet.
            // no longer needed since we creating a new channel
            let clone_conn = conn.clone();
            let new_channel = clone_conn.lock().await.create_channel().await.unwrap();
            Ok(new_channel)
        }else{
            loop {
                if let Some(channel) = chan.pop() {
                    return Ok(channel);
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }
    }

    async fn return_channel(&self, chan: Channel) -> Result<(), MessagingError>{
        // put back the channel
        self.channels.lock().await.push(chan);

        // indicates a channel is returned and allows waiting thread (if any) to use it.
        self.semaphore.add_permits(1);
        Ok(())
    }

}