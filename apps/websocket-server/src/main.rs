use std::sync::Arc;

use axum::{Router, extract::{State, WebSocketUpgrade, ws::{Message as WsMessage, WebSocket}}, response::Response, routing::get};
use futures_util::StreamExt;
use prost::Message as ProtoMessage;
use tokio::sync::broadcast;

use crate::{candle::Candle, handler::handle_socket, model::{InternalTrade, exchange_proto::Trade}};




mod model;
mod state;
mod handler;
mod candle;
struct AppState {
    tx : broadcast::Sender<Vec<u8>>
}

#[tokio::main]
async fn main() {

    // 2. create a broadcast channel 
    let (broadcast_tx, _rx) = broadcast::channel::<Vec<u8>>(1024);

    // internal channel : Redis pub sub -> Aggregator
    let (agg_tx, agg_rx) = tokio::sync::mpsc::channel::<Vec<u8>>(10000);

    let state = Arc::new(AppState { tx : broadcast_tx.clone() });
    

    // Prolly do for all markets here did for only btc_usdt 
    // using config and all markets will do later!!!

    tokio::spawn(async move {
        let client = redis::Client::open("redis://127.0.0.1:6379/").expect("Redis connection failed");
        let mut pubsub = client
            .get_async_pubsub()
            .await
            .expect("Pubsub connection failed");

        pubsub.subscribe("trades:btcusdt").await.expect("trades:btcusdt connection failed");

        let mut stream = pubsub.on_message();

        while let Some(msg) = stream.next().await {
            let payload: Vec<u8> = msg.get_payload().expect("Payload error");

            if let Err(e) = agg_tx.send(payload).await {
                eprintln!("Aggregator channel closed:{}", e);
                break;
            }
        }

    });

    // create another green thread for to implement the aggregator task

    let candle_broadcast_tx = broadcast_tx.clone();
    
    tokio::spawn(async move {
        let mut internal_rx = agg_rx;
        let mut current_candle = Candle::default();

        // The pulse : Fires every 1000 ms
        let mut ticker = tokio::time::interval(tokio::time::Duration::from_secs(1));
        const MICROS_PER_MINUTE: u64 = 60_000_000;

        loop {
            tokio::select! {

                // Branch for NEw trades arriving
                Some(payload) = internal_rx.recv() => {
                    if let Ok(proto) = Trade::decode(&payload[..]) {
                        let trade = InternalTrade::from_proto(proto);
                        
                        // Minute boundary logic
                        let bucket_ts = (trade.timestamp / MICROS_PER_MINUTE) * MICROS_PER_MINUTE;
                        
                        if current_candle.timestamp != 0 && bucket_ts > current_candle.timestamp {
                            current_candle = Candle::default();
                        }
                        
                        current_candle.timestamp = bucket_ts;
                        current_candle.update(trade.price, trade.quantity, trade.timestamp);
                        
                    }
                }

                // Branch for 1 second clock tick
                _ = ticker.tick() => {
                    if current_candle.open > 0.0 {
                        if let Ok(bytes) = serde_json::to_vec(&current_candle) {
                            // Pushes to handle_socket's rx.recv()
                            let _ = candle_broadcast_tx.send(bytes);
                        }
                    }
                }
            }
        }
        
    });


    // 1. simple Axum router
    let app = Router::new()
        .route("/ws", get(handler))
        .with_state(state);

    // 2. create a tcp listener
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();

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