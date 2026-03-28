use tokio::sync::mpsc;

use crate::model::Trade;



pub struct RedisPublisher {
    pub receiver : mpsc::Receiver<Trade>,
    pub redis_url : String
}

impl RedisPublisher {
    
}