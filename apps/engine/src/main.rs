use std::thread;

use crate::{engine::Engine, utils::load_markets, worker::Worker};


mod model;
mod worker;
mod bank;
mod orderbook;
mod utils;
mod engine;

fn main() {

    let markets = load_markets();
    let redis_url = "redis://127.0.0.1:6379";

    for (symbol, config) in markets {
        let market_config = config.clone();
        thread::spawn(move || {
            println!("{} Initializing",symbol );
            let queue = market_config.get_redis_key();
            let symbol = market_config.get_symbol();
            let engine = Engine::new(symbol);
            // Create a new market worker here 
            let worker = Worker::new(queue, symbol, engine);
            
        });
    }
}