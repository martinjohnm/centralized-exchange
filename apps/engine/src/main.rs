use std::{sync::{Arc, Mutex}, thread};

use engine::trade::model::load_markets;

use crate::trade::{bank::Bank, worker::MarketWorker};


mod trade;

fn main() {
    println!("Hello, world!");

    let markets = load_markets();
    // 1. Initialize the single Source of truth
    let shared_bank = Arc::new(Mutex::new(Bank::new()));
    let redis_url = "redis://127.0.0.1:6379";
    for (symbol, config) in markets {
        let market_config = config.clone();

        let bank_handle = Arc::clone(&shared_bank);
        thread::spawn(move || {
            println!("{} initializing", symbol);
            let queue_key = market_config.get_redis_key();
            let worker = MarketWorker::new(
                Arc::clone(&bank_handle),
                redis_url.clone(),
                &symbol,
                &queue_key
            );

            worker.spawn();

        });
    }
    
    // // 2. spawn BTC_USDT Worker
    // let pair_btc = "BTC_USDT";
    // let btc_worker = MarketWorker::new(
    //     Arc::clone(&shared_bank), 
    //     redis_url, 
    //     pair_btc,
        
    // );
    // btc_worker.spawn();

    // // 3. Spawn ETH_USDT worker
    // let eth_worker = MarketWorker::new(
    //     Arc::clone(&shared_bank), 
    //     redis_url, 
    //     "ETH_USDT"
    // );

    // eth_worker.spawn();

    println!("Exchanges are running. BTC and ETH workers alive");

    // Keep main thread alive
    loop {
        std::thread::park();
    }
    
}
