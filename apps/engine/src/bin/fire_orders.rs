use std::thread;
use std::time::Duration;
use engine::trade::model::{MarketConfig, load_markets};
use prost::Message;
use redis::Commands;
// Assuming your crate name in Cargo.toml is 'engine'
use engine::trade::model::exchange_proto::OrderRequestProto;

fn main() {
    // Configuration
    let redis_url = "redis://127.0.0.1/";
    let orders_per_second = 3000; 
    let delay = Duration::from_millis(1000 / orders_per_second);

    println!("Spawning fire thread at {} orders/sec", orders_per_second);

    let markets = load_markets();

    
    // Spawn the worker thread
    let handle = thread::spawn(move || {
        let client = redis::Client::open(redis_url).expect("Invalid Redis URL");
        let mut con = client.get_connection().expect("Failed to connect to Redis");

        let btc_config = markets.get("BTC_USDT")
            .cloned() 
            .expect("Error: BTC_USDT not found in config file");
        loop {

            // 1. Create the Proto message
            let order = OrderRequestProto {
                user_id: 1,
                symbol: btc_config.get_symbol(),
                price: "50000.00".to_string(),
                quantity: "0.1".to_string(),
                side: 0, // Buy
            };

            // 2. Encode to binary
            let mut buf = Vec::new();
            order.encode(&mut buf).unwrap();

            // 3. Push to Redis
            let _: () = con.lpush(btc_config.get_redis_key(), buf).unwrap();

        

            // 4. Wait to maintain the custom speed
            thread::sleep(delay);
        }
    });

    // Keep the main thread alive
    handle.join().unwrap();
}