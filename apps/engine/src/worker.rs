use std::num::NonZeroUsize;

use prost::Message;
use redis::Commands;

use crate::{engine::Engine, model::exchange_proto::ExchangeRequest};




pub struct Worker {
    pub redis_client: redis::Client,
    pub queue_key : String,
    pub symbol : String,
}

impl Worker {
    pub fn new(queue_key : &str, symbol: &str, redis_url : &str) -> Self {

        let queue_key = queue_key.to_string();
        let symbol = symbol.to_string();
        let redis_client = redis::Client::open(redis_url).unwrap();

        Self { 
            queue_key,
            symbol ,
            redis_client
        }
    }

    pub fn run_worker(&mut self) {
        // 1. Poll redis (Blocking call)
        let engine = Engine::new(self.symbol.clone());
        let mut conn = self.redis_client.get_connection().expect("failed to connect to redis");

        // 1. INITIALIZE TIMER OUTSIDE THE LOOP
        let mut last_log_time = std::time::Instant::now();
        let log_interval = std::time::Duration::from_millis(500);


        loop {
            // 1. BLOCK for the first item (Parks the thread until data exists)
            let first_time : Vec<Vec<u8>> = conn.brpop(&self.queue_key, 0.0).expect("Redis connection lost");
            let mut batch = vec![first_time[1].clone()];

            // 2. Greedy drain: Grab up to 99 more items immediately.
            let count = NonZeroUsize::new(99);
            if let Ok(extra_items) = conn.rpop::<&str, Vec<Vec<u8>>>(&self.queue_key, count) {
                batch.extend(extra_items)
            }

            // 3. Process the batch
            for binary_payload in batch {
                // 1. Decode the proto
                let proto = match ExchangeRequest::decode(&binary_payload[..]) {
                    Ok(p) => p,
                    Err(e) => {
                        eprintln!("Corrupt protobuf: {}", e);
                        continue;
                    }
                };
            }

        }

    }
}