use std::sync::{Arc, Mutex};

use crate::trade::{bank::Bank, worker::MarketWorker};


mod trade;

fn main() {
    println!("Hello, world!");

    
    // 1. Initialize the single Source of truth
    let shared_bank = Arc::new(Mutex::new(Bank::new()));
    let redis_url = "redis://127.0.0.1:6379";
    // 2. spawn BTC/USDT Worker
    let btc_worker = MarketWorker::new(
        Arc::clone(&shared_bank), 
        redis_url, 
        "ETH/USDT"
    );
    btc_worker.spawn();

    // 3. Spawn ETH/USDT worker
    let eth_worker = MarketWorker::new(
        Arc::clone(&shared_bank), 
        redis_url, 
        "ETH/USDT"
    );

    eth_worker.spawn();

    println!("Exchanges are running. BTC and ETH workers alive");

    // Keep main thread alive
    loop {
        std::thread::park();
    }
    
}
