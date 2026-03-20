use std::{num::NonZeroUsize, sync::{Arc, Mutex}};

use prost::Message;
use redis::Commands;
use serde::de::value::Error;

use crate::trade::{bank::Bank, engine::MatchingEngine, model::{OrderRequest, exchange_proto::OrderRequestProto, load_markets}};




pub struct MarketWorker {
    bank : Arc<Mutex<Bank>>,
    redis_client: redis::Client,
    pair : String,
    queue_key: String
}

impl MarketWorker {
    pub fn new(bank : Arc<Mutex<Bank>>, redis_url: &str, pair: &str, queue_key: &str) -> Self {
        Self { 
            bank,
            redis_client: redis::Client::open(redis_url).unwrap(),
            pair: pair.to_string(),
            queue_key: queue_key.to_string()
        }
    }

    pub fn spawn(self) {
        let pair = self.pair.clone();
        let bank = self.bank;
        let client = self.redis_client;

        std::thread::spawn(move || {
            let mut con = client.get_connection().expect("Redis connection failed");
            let mut engine = MatchingEngine::new(pair.clone());
            println!("{}", self.queue_key);

            loop {
                // 1. BLOCK for the first item (prevents 100% CPU usage when idle)
                // BRPOP returns [key, value]
                let first_item: Vec<Vec<u8>> = con.brpop(&self.queue_key, 0.0).expect("Redis fail");
                let mut batch = vec![first_item[1].clone()];

                // 2. GREEDY DRAIN: Grab up to 99 more items immediately (Non-blocking)
                // RPOP with a count is MUCH faster than BRPOP in a loop
                let count = NonZeroUsize::new(99);
                let extra_items: Vec<Vec<u8>> = con.rpop(&self.queue_key, count).unwrap_or_default();
                batch.extend(extra_items);

                // 3. PROCESS THE BATCH
                // Now you have 100 orders in memory with only 2 Redis calls!
                for binary_payload in batch {
                    // println!("{:?}", binary_payload);
                    // Decode and Match logic goes here...
                    let order: OrderRequest = match OrderRequestProto::decode(&binary_payload[..]) {
                        Ok(proto) => match OrderRequest::try_from(proto) {
                            Ok(clean_order) => {
                                clean_order
                            },
                            Err(e) => {
                                eprintln!("Validation Failed: {}", e);
                                continue;
                            }
                        },
                        Err(e) => {
                            eprintln!("Protobuf Decode Failed: {}", e);
                        continue;
                        }
                    };
                    
                    // 2. PRE - MATCH LOCK (CHECK and LOCK FUNDS)

                    // 3. MATCH - (NO LOCK - MATCHING THE ORDERS)
                    let trades = engine.submit_order(order);

                    // 4. POST MATCH - (THE SETTLEMENT AND FUND SWAPPING)


                }

                engine.orderbook.get_order_book_stats();
            }
            // loop {
            //     

            //     // // 2. PRE-MATCH LOCK (Check and lock funds)
            //     // {
            //     //     let mut bank_guard = bank.lock().unwrap();
            //     //     if let Err(_) = bank_guard.lock_funds(order.user_id,"BTC/USDT", order.quantity) {
            //     //         continue;
            //     //     }
            //     // }

            //     // // 3. MATCH - (NO lock - Pure logic)
            //     // let trades = engine.submit_order(order);


            //     // // 4. POST MATCH LOCK (Settle balances)
                
            //     // if !trades.is_empty() {
            //     //     // settle the trades here
            //     //     let mut bank_guard = bank.lock().unwrap();

            //     //     for trade in trades {
            //     //         bank_guard.settle_trade(
            //     //             trade.maker_id, 
            //     //             trade.taker_id, 
            //     //             trade.quantity, 
            //     //             trade.price * trade.quantity, 
            //     //             "BTC", 
            //     //             "USDT", 
            //     //             trade.taker_side
            //     //         );
            //     //         // Publish the trade event
            //     //         let _ : () = con.publish("trade_updates", serde_json::to_string(&trade).unwrap()).unwrap();
            //     //     }

            //     // }


            // }
        });
    }
}