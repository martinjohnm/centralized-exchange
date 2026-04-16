use std::{env, sync::Arc, vec};

use axum::{Router, extract::{State, WebSocketUpgrade}, response::Response, routing::get};

use tokio::sync::broadcast;

use crate::{aggregator::start_aggregator, handler::handle_socket, redis::{start_depth_redis_listener, start_trades_redis_listener}, state::AppState};




mod model;
mod state;
mod handler;
mod candle;
mod aggregator;
mod redis;
#[tokio::main]
async fn main() {


    let redis_url = env::var("REDIS_URL")
        .expect("REDIS_URL must be set in .env or system environment");
    // 1. state initialization
    let state = Arc::new(AppState::new(redis_url));

    // 2. Define which markets supports
    let markets = vec!["btcusdt".to_string(), "ethusdt".to_string()];

    let intervals = vec![("candles_1m", 60_000_000 as u64), ("candles_5m", 300_000_000 as u64), ("candles_15m", 900_000_000 as u64)];

    for market_symbol in markets {

            // 1. The Main Ingress from Redis
        let (redis_tx, mut redis_rx) = tokio::sync::mpsc::channel::<Vec<u8>>(10000);
        
        // 2. A list to keep track of all timeframe senders for this market
        let mut timeframe_senders = Vec::new();

        for interval in &intervals {
            let room_id = format!("{}:{}", market_symbol, interval.0);
            let (broadcast_tx,_) = broadcast::channel(1024);
            let (agg_tx, agg_rx) = tokio::sync::mpsc::channel(10000);

            // Save the broadcast sender so that the websocket cand find the "room"
            state.market_map.write().await.insert(room_id, broadcast_tx.clone());

            // spawn the specific aggregator for this timeframe
            tokio::spawn(start_aggregator(interval.clone(), agg_rx, broadcast_tx));
            timeframe_senders.push(agg_tx);
        }
        

        // Distributor task
        // takes 1 trade from redis and send it to 1m 5m and 15m aggregators
        tokio::spawn(async move {
            while let Some(payload) = redis_rx.recv().await {
                for tx in &timeframe_senders {
                    let _ = tx.send(payload.clone()).await;
                }
            }
        });

        // start the redis listenre for this market
        tokio::spawn(start_trades_redis_listener(market_symbol, redis_tx));
    }

    let markets = vec!["btcusdt".to_string(), "ethusdt".to_string()];

    for market_symbol in markets {
        // setup the dpeth room 
        let depth_room_id = format!("{}:depth", market_symbol);
        let (depth_broadcast_tx,_) = broadcast::channel(1024);

        state.market_map.write().await.insert(depth_room_id, depth_broadcast_tx.clone());

        // 2. Start a dedicated Redis listener for Depth
        // This listens to "depth:btcusdt" instead of "trades:btcusdt"
        let depth_channel_name = format!("depth:{}", market_symbol); 
        tokio::spawn(start_depth_redis_listener(market_symbol, depth_broadcast_tx));
    }

    // 1. simple Axum router
    let app = Router::new()
        .route("/ws", get(handler))
        .with_state(state);

    // 2. create a tcp listener
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("Failed to bind to 0.0.0.0:8080");

    println!("Axum websocket listening on 8080");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn handler(
    ws : WebSocketUpgrade, 
    State(state): State<Arc<AppState>>
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

// This function waits for you to press Ctrl-C
async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C handler");
    
    println!("🛑 Ctrl-C received: Closing all WebSocket tasks and Redis connections...");
    // When this returns, Axum will stop accepting new finishing current ones
}