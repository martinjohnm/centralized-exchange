use tokio::{net::unix::pipe::Receiver, sync::mpsc};

use crate::model::InternalTrade;




pub struct RedisPublisher {
    pub receiver : mpsc::Receiver<InternalTrade>,
    pub redis_url : String
}

impl RedisPublisher {
    pub fn new(receiver :mpsc::Receiver<InternalTrade>, redis_url: String) -> Self {
        Self { 
            receiver,
            redis_url 
        }
    }
}