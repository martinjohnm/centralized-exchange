mod engine;
mod model;
mod utils;
mod worker;
mod orderbook;
mod ledger;
use crate::{utils::{initialize_registry, load_markets}, worker::Worker};
use std::thread;

fn main() {
    // 1. Load configuration once at the top level
    let markets = load_markets();
    let redis_url = "redis://127.0.0.1:6379";

    println!("Starting Exchange Engine...");

    println!("{:?}", initialize_registry());
    // 2. Spawn sharded market threads
    for (_, config) in markets {
        // Prepare local copies for the thread move
        let redis_url = redis_url.to_string();
        let queue = config.get_redis_key().to_string();
        let symbol = config.get_symbol().to_string();

        thread::spawn(move || {
            println!("[{}] Initializing market thread...", symbol);

            // Engine and Worker are created inside the thread to ensure
            // they are owned by the thread's stack (Shared-Nothing).
            let mut worker = Worker::new(&queue, &symbol, &redis_url);
            
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