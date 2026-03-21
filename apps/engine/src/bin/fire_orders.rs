use std::thread;
use std::time::Duration;
use engine::trade::model::{Action, MarketConfig, OrderType, load_markets};
use prost::Message;
use redis::Commands;
// Assuming your crate name in Cargo.toml is 'engine'
fn main() {
    // Configuration
    let redis_url = "redis://127.0.0.1/";
    let orders_per_second = 10000; 
    let delay = Duration::from_millis(1000 / orders_per_second);

    println!("Spawning fire thread at {} orders/sec", orders_per_second);

    let markets = load_markets();

    
    // // Spawn the worker thread
    // let handle = thread::spawn(move || {
    //     let client = redis::Client::open(redis_url).expect("Invalid Redis URL");
    //     let mut con = client.get_connection().expect("Failed to connect to Redis");

    //     let btc_config = markets.get("BTC_USDT")
    //         .cloned() 
    //         .expect("Error: BTC_USDT not found in config file");
    //     loop {

    //         // 1. Create the Proto message
    //         let order = create_realistic_order(&btc_config, 50000);

    //         // 2. Encode to binary
    //         let mut buf = Vec::new();
    //         order.encode(&mut buf).unwrap();

    //         // 3. Push to Redis
    //         let _: () = con.lpush(btc_config.get_redis_key(), buf).unwrap();

        

    //         // 4. Wait to maintain the custom speed
    //         thread::sleep(delay);
    //     }
    // });

    // // Keep the main thread alive
    // handle.join().unwrap();
}


// use rand::{thread_rng, Rng}; // Correct imports
// fn create_realistic_order(config: &MarketConfig, current_market_price: u64) -> OrderRequestProto {
//     let mut rng = thread_rng();

//     // 1. Decide Side: 0 for Buy, 1 for Sell
//     let side = rng.gen_range(0..2);

//     // 2. Decide Strategy: 75% Maker, 25% Taker
//     let is_taker = rng.gen_bool(0.25); 
    
//     // 3. Decide if this is a "Whale" (High Quantity)
//     // 5% of all orders are "Whales" that clear the book
//     let is_whale = rng.gen_bool(0.05);

//     let final_price = if side == 0 { // BUY SIDE
//         if is_taker {
//             // Aggressive: Price slightly ABOVE market to guarantee a match
//             current_market_price + rng.gen_range(1..3)
//         } else {
//             // Passive: Price BELOW market to add depth
//             current_market_price - rng.gen_range(1..15)
//         }
//     } else { // SELL SIDE
//         if is_taker {
//             // Aggressive: Price slightly BELOW market to guarantee a match
//             current_market_price - rng.gen_range(1..3)
//         } else {
//             // Passive: Price ABOVE market to add depth
//             current_market_price + rng.gen_range(1..15)
//         }
//     };

//     // 4. Quantity Logic: Makers are small, Takers are medium, Whales are huge.
//     let qty: f64 = if is_whale {
//         // A Whale clears 50.0 to 200.0 units (Wipes out many small orders)
//         rng.gen_range(50.0..200.0)
//     } else if is_taker {
//         // Normal Taker: 5.0 to 15.0 units
//         rng.gen_range(5.0..10.0)
//     } else {
//         // Normal Maker: 0.1 to 2.0 units (Adds small liquidity)
//         rng.gen_range(0.1..2.0)
//     };

//     OrderRequestProto {
//         user_id: rng.gen_range(1..1000),
//         symbol: config.get_symbol(),
//         price: format!("{}.00", final_price),
//         quantity: format!("{:.2}", qty), // 2 decimals for precision
//         side,
//         action : Action::Create as i32,
//         order_type: OrderType::Limit as i32,
//         // Client id should be incremental from for each new order
//         client_id : 1,  // used for the market makers to reduce the round trip time
//         engine_id: 0    // used by the retail users (default 0 because the engine module uses 1 as the order start id)
//     }
// }