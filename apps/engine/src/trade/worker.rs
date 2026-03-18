use std::sync::{Arc, Mutex};

use prost::Message;
use redis::Commands;
use serde::de::value::Error;

use crate::trade::{bank::Bank, engine::MatchingEngine, model::{OrderRequest, exchange_proto::OrderRequestProto}};




pub struct MarketWorker {
    bank : Arc<Mutex<Bank>>,
    redis_client: redis::Client,
    pair : String
}

impl MarketWorker {
    pub fn new(bank : Arc<Mutex<Bank>>, redis_url: &str, pair: &str) -> Self {
        Self { 
            bank,
            redis_client: redis::Client::open(redis_url).unwrap(),
            pair: pair.to_string()
        }
    }

    pub fn spawn(self) {
        let pair = self.pair.clone();
        let bank = self.bank;
        let client = self.redis_client;

        std::thread::spawn(move || {
            let mut con = client.get_connection().expect("Redis connection failed");
            let mut engine = MatchingEngine::new(pair.clone());

            let queu_key = format!("Orders:{}", pair);
            loop {
                // 1. BRPOP : wait for the order
                let data : Vec<Vec<u8>> = con.brpop(&queu_key, 0.0).unwrap();
                let binary_payload = &data[1];

                let order : OrderRequest = match OrderRequestProto::decode(&binary_payload[..]) {
                    Ok(proto) => match OrderRequest::try_from(proto) {
                        Ok(clean_order) => clean_order,
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

                println!("{:?}", order);

                // // 2. PRE-MATCH LOCK (Check and lock funds)
                // {
                //     let mut bank_guard = bank.lock().unwrap();
                //     if let Err(_) = bank_guard.lock_funds(order.user_id,"BTC/USDT", order.quantity) {
                //         continue;
                //     }
                // }

                // // 3. MATCH - (NO lock - Pure logic)
                // let trades = engine.submit_order(order);


                // // 4. POST MATCH LOCK (Settle balances)
                
                // if !trades.is_empty() {
                //     // settle the trades here
                //     let mut bank_guard = bank.lock().unwrap();

                //     for trade in trades {
                //         bank_guard.settle_trade(
                //             trade.maker_id, 
                //             trade.taker_id, 
                //             trade.quantity, 
                //             trade.price * trade.quantity, 
                //             "BTC", 
                //             "USDT", 
                //             trade.taker_side
                //         );
                //         // Publish the trade event
                //         let _ : () = con.publish("trade_updates", serde_json::to_string(&trade).unwrap()).unwrap();
                //     }

                // }


            }
        });
    }
}