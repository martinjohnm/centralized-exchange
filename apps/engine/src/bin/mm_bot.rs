

use std::time::Duration;

use rust_decimal_macros::dec;
use tokio;

pub mod exchange_proto {
    include!(concat!(env!("OUT_DIR"), "/exchange.rs"));
}
use prost::Message; // This trait provides the .encode() method

use tokio::time::{interval, MissedTickBehavior};
// Assuming your proto is generated here
use crate::exchange_proto::ExchangeRequest; 

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Establish a single Multiplexed Connection (Thread-safe)
    let client = redis::Client::open("redis://127.0.0.1:6379/")?;
    let multiplex_conn = client.get_multiplexed_async_connection().await?;

    println!("Starting Swarm: 200 bots @ 50 orders/sec (Goal: 10k TPS)");

    let btc_market : &'static str = "trades:btc_usdt";

    for bot_id in 1..=200 {
        let mut bot_conn = multiplex_conn.clone();
        
        tokio::spawn(async move {
            let mut market_price = 65000.0;
            let mut timer = interval(Duration::from_millis(50));
            timer.set_missed_tick_behavior(MissedTickBehavior::Delay);

            loop {
                // 1. Await the timer (No non-Send types exist here)
                timer.tick().await; 

                // 2. Open a temporary scope for the RNG
                let payload = {
                    let mut rng = rand::thread_rng();
                    let order = generate_random_order(
                        bot_id, 
                        &mut rng, 
                        &mut market_price
                    );

                    let mut buf = Vec::with_capacity(order.encoded_len());
                    order.encode(&mut buf).unwrap();

                    buf // Return the bytes the RNG dies here 
                };

                // 3. Now we can await redis (The RNG is died)

                let _: () = redis::cmd("LPUSH")
                    .arg(btc_market)
                    .arg(payload)
                    .query_async(&mut bot_conn)
                    .await
                    .unwrap();
            }
        });
    }

    // Keep the main process alive
    tokio::signal::ctrl_c().await?;
    println!("Swarm shut down.");
    Ok(())
}
use rand::Rng;

// Pass the rng and the last known price to create a "Trend"
fn generate_random_order(
    total_count: u64, 
    rng: &mut impl Rng, 
    current_market_price: &mut f64
) -> exchange_proto::ExchangeRequest {
    
    // 1. Move the price slightly (Random Walk)
    let drift = rng.gen_range(-5.0..5.0);
    *current_market_price += drift;

    let is_buy = rng.gen_bool(0.5);
    
    // 2. Bids should be slightly BELOW market, Asks slightly ABOVE
    let price_offset = if is_buy { -1.0 } else { 1.0 };
    let order_price = *current_market_price + price_offset;

    exchange_proto::ExchangeRequest {
        user_id: rng.gen_range(1..500),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(), // Use real timestamps!
        action: Some(exchange_proto::exchange_request::Action::Create(
            exchange_proto::CreateOrder {
                market: 1,
                side: if is_buy { 0 } else { 1 },
                price: format!("{:.2}", order_price),
                quantity: rng.gen_range(1..101).to_string(),
                order_type: 0, 
                client_id: total_count,
            }
        )),
    }
}