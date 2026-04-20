use std::{i64, str::FromStr, time::{Duration, Instant}};

use chrono::{TimeZone, Utc};
use prost::Message;
use redis::AsyncCommands;
use rust_decimal::Decimal;
use sqlx::QueryBuilder;

use crate::exchange_proto::{Side, Trade};





pub async fn start_trade_worker(pool : sqlx::Pool<sqlx::Postgres>, client: redis::Client) -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = client.get_multiplexed_async_connection().await?;
    println!("Trade DB worker online: Connected to redis and timescaledb");

    // ------------------------------BATCHING CONFIG---------------------
    let mut trade_buffer = Vec::with_capacity(1000);
    let mut last_flush = Instant::now();
    let max_batch_size = 500;
    let max_wait_time = Duration::from_millis(100);

    loop {
        // 1. NON-BLOCKING DRAIN: Pull as many items as possible from Redis quickly
        // This keeps the Redis queue empty and memory-resident
        
        while let Ok(Some(bytes)) = conn.rpop::<_, Option<Vec<u8>>>("db_processor", None).await {
            if let Ok(trade) = Trade::decode(&bytes[..]) {
                trade_buffer.push(trade);
            }

            // loop breaking logic
            if trade_buffer.len() >= max_batch_size {break;}
        }



        // 2. THE LOGIC FIX: Check if we SHOULD flush
        let should_flush = !trade_buffer.is_empty() && 
            (trade_buffer.len() >= max_batch_size || last_flush.elapsed() >= max_wait_time);

        
        if should_flush {
            let mut query_builder = QueryBuilder::new(
            "INSERT INTO trade_history (time, symbol, price, volume, taker_user_id, maker_user_id, taker_order_id, maker_order_id, taker_side) "
            );

            query_builder.push_values(trade_buffer.iter(), |mut b, trade| {
                let datetime = Utc.timestamp_micros(trade.timestamp as i64).unwrap();
                let price = Decimal::from_str(&trade.price).unwrap_or(Decimal::ZERO);
                let quantity = Decimal::from_str(&trade.quantity).unwrap_or(Decimal::ZERO);
                let side_str = if trade.taker_side == Side::Buy as i32 { "BUY" } else { "SELL" };
                let symbol = format!("{:?}_{:?}", trade.base, trade.quote);

                b.push_bind(datetime)
                 .push_bind(symbol)
                 .push_bind(price)
                 .push_bind(quantity)
                 .push_bind(trade.taker_id as i64) // 5. taker_user_id (from your new proto fields)
                 .push_bind(trade.maker_id as i64) // 6. maker_user_id
                 .push_bind(trade.taker_order_id as i64) // 7. taker_order_id
                 .push_bind(trade.maker_order_id as i64) // 8. maker_order_id
                 .push_bind(side_str);
            });

            let query = query_builder.build();
            if let Err(e) = query.execute(&pool).await {
                eprintln!("Database insert error: {}", e);
            }

            trade_buffer.clear();
            last_flush = Instant::now();
        }

        // 3. ADAPTIVE SLEEP: Only sleep if there's nothing to do
        if trade_buffer.is_empty() {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }
}