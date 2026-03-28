use tokio::{net::unix::pipe::Receiver, sync::mpsc};

use crate::model::Trade;



pub struct RedisPublisher {
    pub receiver : mpsc::Receiver<Trade>,
    pub redis_url : String
}

impl RedisPublisher {
    pub fn new(receiver :mpsc::Receiver<Trade>, redis_url: String) -> Self {
        Self { 
            receiver,
            redis_url 
        }
    }
}