use prost::Message;
use redis::Commands;
use std::time::{Instant, Duration};
use exchange_proto::{ExchangeRequest, exchange_request::Action, CreateOrder};

pub mod exchange_proto {
    include!(concat!(env!("OUT_DIR"), "/exchange.rs"));
}

fn main() {
    let client = redis::Client::open("redis://127.0.0.1:6379").unwrap();
    let mut con = client.get_connection().expect("Redis down");
    
    let queue_key = "orders:BTC_USDT";

    let mut count = 0;
    let start = Instant::now();

    println!("🔥 Continuous Firing Started. Press Ctrl+C to stop.");

    loop {
        count += 1;
        
        // 1. Alternate between Buy and Sell to force matches
        let side = if count % 2 == 0 { 0 } else { 1 };
        let price = if side == 0 { "50000.00" } else { "49999.00" };

        let request = ExchangeRequest {
            user_id: (count % 1000) as u64, // Rotate through 1000 users
            timestamp: 1711000000 + count,
            action: Some(Action::Create(CreateOrder {
                symbol: "BTC_USDT".to_string(),
                side,
                price: price.to_string(),
                quantity: "0.1".to_string(),
                order_type: 0,
                client_id: count,
            })),
        };

        // 2. Serialize
        let mut buf = Vec::new();
        request.encode(&mut buf).unwrap();

        // 3. Fire
        let _: () = con.lpush(queue_key, buf).unwrap();

        // 4. Progress Report every 5000 orders
        if count % 5000 == 0 {
            let tps = count as f64 / start.elapsed().as_secs_f64();
            println!("🚀 Sent {} orders | Avg TPS: {:.0}", count, tps);
        }
        
        // Optional: Add a tiny sleep if you want to throttle it
        // std::thread::sleep(Duration::from_micros(10)); 
    }
}