use std::{num::NonZeroUsize};

use prost::Message;
use redis::{Commands, Connection};
use tokio::sync::mpsc::{Sender, Receiver};
use crate::{engine::Engine, model::{DepthResponse, InternalTrade, OrderRequest, exchange_proto::{ExchangeRequest, ExecutionReport, MarketId, OrderUpdate}}, utils::MarketConfig};




pub struct Worker {
    pub connection: Connection,
    pub market_id : MarketId,
    pub engine : Engine,
    pub config : MarketConfig,
    pub depth_producer : Sender<DepthResponse>
}

impl Worker {
    pub fn new(market_id : MarketId, config : MarketConfig, redis_url : &str,  trade_tx_clone : Sender<InternalTrade>, depth_tx_clone: Sender<DepthResponse>, report_tx_clone: Sender<ExecutionReport>, orders_tx_clone: Sender<OrderUpdate>) -> Self {

        let queue_key = config.redis_key;
        let symbol = market_id;
        let redis_client = redis::Client::open(redis_url).unwrap();

        let trade_tx_clone = trade_tx_clone.clone();
        let engine = Engine::new(config, trade_tx_clone, report_tx_clone, orders_tx_clone);
        let connection = redis_client.get_connection().expect("failed to connect to redis");

        Self { 
            market_id,
            connection,
            engine,
            config,
            depth_producer : depth_tx_clone
        }
    }

    pub fn run_worker(&mut self) {
        // 1. Poll redis (Blocking call)
      
        let mut last_snapshot = std::time::Instant::now();
        let snapshot_interval = std::time::Duration::from_secs(1);

        loop {
            // 1. BLOCK for the first item (Parks the thread until data exists)
            let result: redis::RedisResult<Option<Vec<Vec<u8>>>> = 
                self.connection.brpop(&self.config.redis_key, 1.0);

            match result {
                Ok(Some(data)) => {
                    let mut batch = vec![data[1].clone()];
                    let count = NonZeroUsize::new(99);
                    if let Ok(extra_items) = self.connection.rpop::<&str, Vec<Vec<u8>>>(&self.config.redis_key, count) {
                        batch.extend(extra_items);
                    }

                    for binary_payload in batch {
                        if let Ok(proto) = ExchangeRequest::decode(&binary_payload[..]) {
                            // Transform the order to clean "OrderRequest"
                            match OrderRequest::try_from(proto) {
                                Ok(clean_order) => {
                                    self.engine.process_request(clean_order);
                                }
                                Err(e) => {
                                    eprintln!("Firewall rejected the order")
                                }
                            }
                        }
                    }
                }

                Ok(None) => {
                    // Timeout reached
                }

                Err(e) => {
                    eprintln!("Redis error: {}", e)

                }
            }

            // We are sending the depth every second to a transmitter as depth snapshot
            if last_snapshot.elapsed() >= snapshot_interval {

                // geting the top 50 depth response 

                // Try to send the depth here 
                let depth = self.engine.get_market_depth(50);
                if let Err(e) = self.depth_producer.try_send(depth) {
                        eprintln!("trade is not sent");
                    }
                

                // update the last_snapshot
                last_snapshot = std::time::Instant::now();
            }
        }

    }
}