use std::sync::Arc;

use axum::{Router, extract::{State, WebSocketUpgrade, ws::{Message as WsMessage, WebSocket}}, response::Response, routing::get};
use futures_util::StreamExt;
use prost::Message as ProtoMessage;
use tokio::sync::broadcast;

use crate::{candle::Candle, model::{InternalTrade, exchange_proto::Trade}};




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
    // 3. spawn a background green thread for redis litening incoming messages
    let redis_tx = state.tx.clone();

    tokio::spawn(async move {
        let client = redis::Client::open("redis://127.0.0.1:6379/").expect("Redis connection failed");
        let mut conn = client.get_multiplexed_async_connection().await.expect("Redis connetcon failde");
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

        while let Some(payload) = internal_rx.recv().await {
            // Now that 'Message' is in scope, .decode() will be found
            match Trade::decode(&payload[..]) {
                Ok(proto_trade) => {
                    // Convert to your internal Decimal/f64 struct for math
                    let internal = InternalTrade::from_proto(proto_trade);
                    current_candle.update(internal.price, internal.quantity, internal.timestamp);
                    println!("Current candle {:?}", current_candle);
                }
                Err(e) => {
                    eprintln!("❌ Protobuf decode error: {:?}", e);
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

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    println!("Connection establishes");
    let mut rx = state.tx.subscribe();

    // 5. THE REAL-TIME LOOP
    // We wait for either a trade from Redis OR a 10s Heartbeat
    let mut heartbeat_interval = tokio::time::interval(tokio::time::Duration::from_secs(10));
    // Test loop
    
    let mut ticker = tokio::time::interval(tokio::time::Duration::from_secs(1));
    
    
    loop {
        tokio::select! {
            // 🚀 Capture the result of the receive
            res = rx.recv() => {
                match res {
                    Ok(bytes) => {
                        if socket.send(WsMessage::Binary(bytes.into())).await.is_err() {
                            println!("🔌 Client disconnected");
                            break; 
                        }
                    }
                    // ⚠️ This happens if the engine is too fast for the websocket
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        eprintln!("⚠️ Client lagged by {} messages", n);
                        // We don't break, but we acknowledge the lag
                    }
                    Err(_) => break, // Channel closed
                }
            }
            
            _ = heartbeat_interval.tick() => {
                if socket.send(WsMessage::Ping(vec![].into())).await.is_err() {
                    break;
                }
            }
        }
    }
}

// This function waits for you to press Ctrl-C
async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C handler");
    
    println!("🛑 Ctrl-C received: Closing all WebSocket tasks and Redis connections...");
    // When this returns, Axum will stop accepting new finishing current ones
}