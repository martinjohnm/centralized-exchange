use prost::Message;
use redis::Commands;
use std::time::{Instant, Duration};
use std::env; // To take CLI arguments

pub mod exchange_proto {
    include!(concat!(env!("OUT_DIR"), "/exchange.rs"));
}

fn main() {
    // 1. Get OPS from command line args, default to 1 if not provided
    let args: Vec<String> = env::args().collect();
    let ops: f64 = args.get(1)
        .map(|val| val.parse().unwrap_or(1.0))
        .unwrap_or(1.0);

    let client = redis::Client::open("redis://127.0.0.1:6379").unwrap();
    let mut con = client.get_connection().expect("Redis down");
    let queue_key = "orders:BTC_USDT";

    let interval = Duration::from_secs_f64(1.0 / ops);
    let mut count = 0;
    
    println!("🔥 Firing at {} orders per second. Ctrl+C to stop.", ops);

    loop {
        let start_time = Instant::now();
        count += 1;

        // --- Create & Serialize Request ---
        let side = if count % 2 == 0 { 0 } else { 1 };
        let request = exchange_proto::ExchangeRequest {
            user_id: (count % 1000) as u64,
            timestamp: 1711000000 + count,
            action: Some(exchange_proto::exchange_request::Action::Create(
                exchange_proto::CreateOrder {
                    symbol: "BTC_USDT".to_string(),
                    side,
                    price: "50000.00".to_string(),
                    quantity: "0.1".to_string(),
                    order_type: 0,
                    client_id: count,
                }
            )),
        };

        let mut buf = Vec::new();
        request.encode(&mut buf).unwrap();
        let _: () = con.lpush(queue_key, buf).unwrap();

        // --- Precise Throttling Logic ---
        let elapsed = start_time.elapsed();
        if elapsed < interval {
            std::thread::sleep(interval - elapsed);
        }

        if count % (ops as u64).max(1) == 0 {
            println!("🚀 Total sent: {}", count);
        }
    }
}