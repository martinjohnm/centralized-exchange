mod engine;
mod model;
mod utils;
mod worker;
mod orderbook;
mod ledger;
mod publisher;

use tokio::sync::mpsc;
use crate::{model::InternalTrade, publisher::RedisPublisher, utils::{MarketConfig, load_markets_from_proto}, worker::Worker};
use std::thread;
use dotenvy::dotenv;
use std::env;

#[tokio::main]
async fn main() {
    // 1. Load configuration once at the top level
    let markets = load_markets_from_proto();
    println!("DEBUG: Current REDIS_URL is: {:?}", std::env::var("REDIS_URL"));
    let redis_url = env::var("REDIS_URL")
        .expect("REDIS_URL must be set in .env or system environment");
    println!("{:?}", redis_url);

    println!("Starting Exchange Engine...");


    // 1. Create the central "Trade pipe" 
    let (trade_tx, trade_rx) = mpsc::channel::<InternalTrade>(10000);

    // 2. Create a green thread for broadcastor (which holds the single receiver and listens and broadcasts 
    //    events from various markets in which each markets clones a copy of the transmitter to send events)
    let redis_url_to_redis_publisher = redis_url.to_string().clone();
    tokio::spawn(async move {
        let publisher = RedisPublisher::new(trade_rx, redis_url_to_redis_publisher);
        publisher.run().await;
    });

    // 2. Spawn sharded market threads
    for (market_id, config) in markets {
        // Prepare local copies for the thread move
        let redis_url = redis_url.to_string();
        
        // create a transmitter clone before the thread creation
        let trade_tx_clone = trade_tx.clone();

        

        thread::spawn(move || {
            println!("[] Initializing market thread...");

            // Engine and Worker are created inside the thread to ensure
            // they are owned by the thread's stack (Shared-Nothing).
            let mut worker = Worker::new(market_id, config, &redis_url, trade_tx_clone);
            
            worker.run_worker();
        });
    }

    // 3. Keep the main thread alive properly
    // 'park' is better than 'loop {}' because it puts the thread to sleep 
    // without consuming 100% CPU.
    loop {
        thread::park();
    }
}