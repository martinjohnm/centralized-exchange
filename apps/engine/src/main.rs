mod engine;
mod model;
mod utils;
mod worker;
mod orderbook;
mod ledger;
use crate::{model::AssetRegistry, utils::{initialize_registry, load_markets}, worker::Worker};
use std::{sync::Arc, thread};

fn main() {
    // 1. Load the Registry ONCE and wrap in Arc for cross-thread sharing
    let asset_registry = Arc::new(initialize_registry());
    let redis_url = "redis://127.0.0.1:6379";

    println!("🚀 Exchange Engine Booting...");

    // 2. Iterate through the markets
    // We use .values() because we only need the config, not the symbol string key

    for config in load_markets().values() {
        
        // Clone the ARC (this just increments a counter)
        let registry_handle = Arc::clone(&asset_registry);

        let redis_url = redis_url.to_string();
        let queue = config.get_redis_key().to_string();
        let symbol = config.get_symbol().to_string();
        
        // get the specific market_id for this thread
        thread::spawn(move || {
            let m_id = registry_handle.get_id_by_symbol(symbol)
                .expect(&format!("CRITICAL: Market symbol {} not found in Registry", symbol));
            println!("[{}] Thread Started (ID: {})", symbol, m_id);

            // Engine and Worker are created inside the thread to ensure
            // they are owned by the thread's stack (Shared-Nothing).
            let mut worker = Worker::new(
                m_id, 
                registry_handle, 
                queue, 
                redis_url
            );
            
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