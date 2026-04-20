use std::{i64, str::FromStr, time::{Duration, Instant}};

use prost::Message;
use redis::AsyncCommands;
use rust_decimal::Decimal;
use sqlx::QueryBuilder;

use crate::exchange_proto::{OrderUpdate, Side};



pub async fn start_order_worker(pool : sqlx::Pool<sqlx::Postgres>, redis_client: redis::Client) -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = redis_client.get_multiplexed_async_connection().await?;
    let mut order_buffer = Vec::with_capacity(1000);
    let mut last_flush = Instant::now();

    let max_batch_size = 500;
    let max_wait_time = Duration::from_millis(100);

    loop {
        // Drain redis for order updates
        while let Ok(Some(bytes)) = conn.rpop::<_, Option<Vec<u8>>>("db_order_processor", None).await  {
            if let Ok(update) = OrderUpdate::decode(&bytes[..]) {
                order_buffer.push(update);
            }
            if order_buffer.len() >= max_batch_size {break;}
        }

        // 2. THE LOGIC FIX: Check if we SHOULD flush
        let should_flush = !order_buffer.is_empty() && 
            (order_buffer.len() >= max_batch_size || last_flush.elapsed() >= max_wait_time);

        // Batch Update/Upsert

        if should_flush {
            
            // 1. Create a map to keep only the latest update for each order_id 
            let mut unique_updates = std::collections::BTreeMap::new();

            for ord in order_buffer.drain(..) {
                unique_updates.insert(ord.engine_id, ord);
            }



            let mut query_builder = QueryBuilder::new(
                "INSERT INTO open_orders (order_id, user_id, symbol, side, price, quantity, filled) "
            );

            query_builder.push_values(unique_updates.values(), |mut b, ord| {
                let price = Decimal::from_str(&ord.price).unwrap_or(Decimal::ZERO);
                let quantity = Decimal::from_str(&ord.quantity).unwrap_or(Decimal::ZERO);
                let filled = Decimal::from_str(&ord.filled_quantity).unwrap_or(Decimal::ZERO);
                let side_str = if ord.side == Side::Buy as i32 { "BUY" } else { "SELL" };
                let symbol = format!("{:?}", ord.market);

                b.push_bind(ord.engine_id as i64)
                .push_bind(ord.user_id as i64)
                .push_bind(symbol)
                .push_bind(side_str)
                .push_bind(price)
                .push_bind(quantity)
                .push_bind(filled);
            });

            // THE ESSENTIAL ADDITION:
            query_builder.push(
                " ON CONFLICT (order_id) DO UPDATE SET "
            );
            query_builder.push("filled = EXCLUDED.filled"); 

            // Optional: If you track status in this table, update it too
            // query_builder.push(", status = EXCLUDED.status"); 

            let query = query_builder.build();
            // ... execute ...
            if let Err(e) = query.execute(&pool).await {
                eprintln!("Database insert error: {}", e);
            }
            order_buffer.clear();
            last_flush = Instant::now();
        }

        // 3. ADAPTIVE SLEEP: Only sleep if there's nothing to do
        if order_buffer.is_empty() {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }
}