use common_lib::message::message::JobUpdate;
use tokio::sync::broadcast::{channel, Receiver, Sender};


pub struct JobProggressBroadcaster {
   pub tx: Sender<JobUpdate>,
   pub rx: Receiver<JobUpdate>,
}

impl <'a> JobProggressBroadcaster{
    pub fn new() -> Self {
        let (tx, rx) : (Sender<JobUpdate>, Receiver<JobUpdate>)  = channel(100);
        JobProggressBroadcaster { 
            tx:tx,
            rx:rx,
        }
    }
}