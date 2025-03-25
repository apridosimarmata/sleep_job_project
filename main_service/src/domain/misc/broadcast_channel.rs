use common_lib::message::message::{JobCreationRequest, JobUpdate};
use futures::sink::Send;
use tokio::sync::broadcast::{channel, Receiver, Sender};


pub struct JobProggressBroadcaster {
   pub tx: Sender<JobUpdate>,
   pub rx: Receiver<JobUpdate>,
}

impl <'a> JobProggressBroadcaster{
    pub fn new() -> Self {
        let (tx, mut rx) : (Sender<JobUpdate>, Receiver<JobUpdate>)  = channel(100);
        JobProggressBroadcaster { 
            tx:tx,
            rx:rx,
        }
    }
}