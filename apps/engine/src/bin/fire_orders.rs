use prost::Message;
use redis::Commands;
use std::time::{Instant, Duration};
use std::env;
use std::thread;
use dotenvy::dotenv;

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
        .unwrap_or(100.0);

    let redis_url = env::var("REDIS_URL")
        .expect("REDIS_URL must be set in .env or system environment");


    let client = redis::Client::open(redis_url).expect("Invalid Redis URL");
    let mut con = client.get_connection().expect("Failed to connect to Redis");
    let queue_key: &'static str = "trades:btc_usdt";

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
        let request = generate_random_order(2);

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


use rand::Rng;

fn generate_random_order(total_count: u64) -> exchange_proto::ExchangeRequest {
    let mut rng = rand::thread_rng();

    // 1. Randomize the Side (50/50 chance)
    let is_buy = rng.gen_bool(0.5);
    let side = if is_buy { 0 } else { 1 };

    // 2. Randomize the Price (Random Walk around 65,000)
    // We add/subtract up to $100 from the base price
    let offset: f64 = rng.gen_range(-100.0..100.0);
    let price_val = 65000.0 + offset;
    let price_str = format!("{:.2}", price_val);

    // 3. Randomize Quantity (0.01 to 0.50)
    let qty_val: u64 = rng.gen_range(1..101); 
    let qty_str = qty_val.to_string(); // Simple string conversion

    exchange_proto::ExchangeRequest {
        user_id: (rng.gen_range(1..500)) as u64, // Random user from a pool of 500
        timestamp: 123456,
        action: Some(exchange_proto::exchange_request::Action::Create(
            exchange_proto::CreateOrder {
                market: 1,
                side,
                price: price_str,
                quantity: qty_str,
                order_type: 0, // Limit Order
                client_id: total_count,
            }
        )),
    }
}