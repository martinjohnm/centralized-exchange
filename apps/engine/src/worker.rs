use std::{num::NonZeroUsize};

use prost::Message;
use redis::{Commands, Connection};
use tokio::sync::mpsc::{Sender, Receiver};
use crate::{engine::Engine, model::{InternalTrade, OrderRequest, exchange_proto::{ExchangeRequest, MarketId}}, utils::MarketConfig};




pub struct Worker {
    pub connection: Connection,
    pub market_id : MarketId,
    pub engine : Engine,
    pub config : MarketConfig
}

impl Worker {
    pub fn new(market_id : MarketId, config : MarketConfig, redis_url : &str,  tx_clone : Sender<InternalTrade>) -> Self {

        let queue_key = config.redis_key;
        let symbol = market_id;
        let redis_client = redis::Client::open(redis_url).unwrap();

        let tx_clone = tx_clone.clone();
        let engine = Engine::new(config, tx_clone);
        let connection = redis_client.get_connection().expect("failed to connect to redis");

        Self { 
            market_id,
            connection,
            engine,
            config
        }
    }

    pub fn run_worker(&mut self) {
        // 1. Poll redis (Blocking call)
      
        // 1. INITIALIZE TIMER OUTSIDE THE LOOP
        let mut _last_log_time = std::time::Instant::now();
        let _log_interval = std::time::Duration::from_millis(500);


        loop {
            // 1. BLOCK for the first item (Parks the thread until data exists)
            let first_time : Vec<Vec<u8>> = self.connection.brpop(&self.config.redis_key, 0.0).expect("Redis connection lost");
            let mut batch = vec![first_time[1].clone()];

            // 2. Greedy drain: Grab up to 99 more items immediately.
            let count = NonZeroUsize::new(99);
            if let Ok(extra_items) = self.connection.rpop::<&str, Vec<Vec<u8>>>(&self.config.redis_key, count) {
                batch.extend(extra_items)
            }

            // 3. Process the batch
            // Inside the batch loop
            for binary_payload in batch {
                // 1. Decode the raw bytes into the Protobuf "ExchangeRequest"
                if let Ok(proto) = ExchangeRequest::decode(&binary_payload[..]) {
                    
                    // 2. Transform the Proto into your Clean "OrderRequest"
                    // This uses the TryFrom logic we wrote earlier!
                    match OrderRequest::try_from(proto) {
                        Ok(clean_order) => {
                            // 3. Pass the CLEAN order to the engine
                            self.engine.process_request(clean_order);
                        }
                        Err(e) => eprintln!("Firewall rejected order: {}", e),
                    }
                }
            }

        }

    }
}