use std::thread;

use crate::utils::load_markets;


mod model;
mod worker;
mod bank;
mod orderbook;
mod utils;

fn main() {

    let markets = load_markets();
    let redis_url = "redis://127.0.0.1:6379";

    for (symbol, config) in markets {
        let market_config = config.clone();
        thread::spawn(move || {
            println!("{} Initializing",symbol );
            let queue = market_config.get_redis_key();
            // Create a new market worker here 
        });
    }
}