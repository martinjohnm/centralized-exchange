use prost::Message;
use redis::Commands;
use std::time::{Instant, Duration};
use std::env;
use std::thread;

pub mod exchange_proto {
    include!(concat!(env!("OUT_DIR"), "/exchange.rs"));
}

/// A simple helper to handle high-precision timing
struct Throttler {
    start_time: Instant,
    interval: Duration,
    target_ops: f64,
}

impl Throttler {
    fn new(ops: f64) -> Self {
        Self {
            start_time: Instant::now(),
            interval: Duration::from_secs_f64(1.0 / ops),
            target_ops: ops,
        }
    }

    fn wait(&self, iteration: u64) {
        let target_time = self.start_time + self.interval * (iteration as u32);
        let now = Instant::now();
        if target_time > now {
            thread::sleep(target_time - now);
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let target_ops: f64 = args.get(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(100000.0);

    let client = redis::Client::open("redis://127.0.0.1:6379").expect("Invalid Redis URL");
    let mut con = client.get_connection().expect("Failed to connect to Redis");
    let queue_key = "orders:BTC_USDT";

    let throttler = Throttler::new(target_ops);
    let mut total_count: u64 = 0;
    
    // Performance Tracking
    let mut last_report = Instant::now();
    let mut last_count = 0;

    // println!("🚀 Load Generator Started | Target: {} TPS | Key: {}", target_ops, queue_key);
    // println!("----------------------------------------------------------");

    loop {
        total_count += 1;

        // 1. Precise Throttling: Calculate wait based on total elapsed time 
        // to prevent drift over long periods.
        throttler.wait(total_count);

        // 2. Generate Protobuf Message
        let request = exchange_proto::ExchangeRequest {
            user_id: (total_count % 500) as u64,
            timestamp: 1711000000 + total_count,
            action: Some(exchange_proto::exchange_request::Action::Create(
                exchange_proto::CreateOrder {
                    symbol: "BTC_USDT".to_string(),
                    side: if total_count % 2 == 0 { 0 } else { 1 },
                    price: "65000.00".to_string(),
                    quantity: "0.05".to_string(),
                    order_type: 0,
                    client_id: total_count,
                }
            )),
        };

        // 3. Serialize and Push
        let mut buf = Vec::with_capacity(request.encoded_len());
        request.encode(&mut buf).unwrap();
        let _: () = con.lpush(queue_key, buf).expect("Redis LPUSH failed");

        // 4. Performance Reporting (Every 1 second)
        if last_report.elapsed() >= Duration::from_secs(1) {
            let now = Instant::now();
            let elapsed = now.duration_since(last_report).as_secs_f64();
            let current_tps = (total_count - last_count) as f64 / elapsed;
            
            // print!("\r[Live] Sent: {:<8} | Actual TPS: {:<8.2} | Target: {:.0}", 
            //     total_count, current_tps, target_ops
            // );

        }}
}