use futures_util::StreamExt;
use dotenvy::dotenv;
use tokio::sync::broadcast;
use std::env;

use crate::model::{ exchange_proto::{DepthUpdate, StreamType, WsOutMessage, ws_out_message::Data}};
use prost::Message as ProtoMessage;


pub async fn start_trades_redis_listener(
    market: String, 
    agg_tx: tokio::sync::mpsc::Sender<Vec<u8>>
) {
    let redis_url = env::var("REDIS_URL")
        .expect("REDIS_URL must be set in .env or system environment");
    println!("{:?}", redis_url);
    // 1. Establish connection
    let client = redis::Client::open(redis_url)
        .expect("Failed to create Redis client");
        
    let mut pubsub = client
        .get_async_pubsub()
        .await
        .expect("Failed to get async pubsub");

    // 2. Subscribe to the specific market channel
    // Example: "trades:btc_usdt"
    let channel_name = format!("trades:{}", market);
    pubsub.subscribe(&channel_name)
        .await
        .expect(&format!("Failed to subscribe to {}", channel_name));

    println!("📡 Redis Listener started for: {}", channel_name);

    let mut stream = pubsub.on_message();

    // 3. Listen and forward to Aggregator
    while let Some(msg) = stream.next().await {
        match msg.get_payload::<Vec<u8>>() {
            Ok(payload) => {
                // We send the raw Protobuf bytes to the Aggregator
                if let Err(e) = agg_tx.send(payload).await {
                    eprintln!("❌ Aggregator for {} closed: {}", market, e);
                    break;
                }
            }
            Err(e) => eprintln!("❌ Redis payload error on {}: {}", market, e),
        }
    }
}


pub async fn start_depth_redis_listener(
    market: String, 
    broadcast_tx: broadcast::Sender<Vec<u8>>
) {
    let redis_url = env::var("REDIS_URL")
        .expect("REDIS_URL must be set in .env or system environment");
    println!("{:?}", redis_url);
    // 1. Establish connection
    let client = redis::Client::open(redis_url)
        .expect("Failed to create Redis client");
        
    let mut pubsub = client
        .get_async_pubsub()
        .await
        .expect("Failed to get async pubsub");

    // 2. Subscribe to the specific market channel
    // Example: "trades:btc_usdt"
    let channel_name = format!("depth:{}", market);
    pubsub.subscribe(&channel_name)
        .await
        .expect(&format!("Failed to subscribe to {}", channel_name));

    println!("📡 Redis Listener started for: {}", channel_name);

    let mut stream = pubsub.on_message();

    // 3. Listen and forward to Broadcastor
    while let Some(msg) = stream.next().await {
        match msg.get_payload::<Vec<u8>>() {
            Ok(payload) => {
                // We send the raw Protobuf bytes to the Broadcastor
                if let Ok(proto) = DepthUpdate::decode(&payload[..]) {
                    let out_message = WsOutMessage {
                        stream : StreamType::Depth as i32,
                        data : Some(Data::Depth(proto))
                    };
                    let mut payload = Vec::new();
                    if out_message.encode(&mut payload).is_ok() {
                        let _ = broadcast_tx.send(payload);
                    }
                }
                
            }
            Err(e) => eprintln!("❌ Redis payload error on {}: {}", market, e),
        }
    }
}