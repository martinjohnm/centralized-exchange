use std::sync::Arc;

use axum::{Router, extract::{State, WebSocketUpgrade, ws::{Message, WebSocket}}, response::Response, routing::get};
use futures_util::StreamExt;
use tokio::sync::broadcast;



mod model;
mod state;
mod handler;

struct AppState {
    tx : broadcast::Sender<Vec<u8>>
}

#[tokio::main]
async fn main() {

    // 2. create a broadcast channel 
    let (tx, _rx) = broadcast::channel::<Vec<u8>>(1024);
    let state = Arc::new(AppState { tx });
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

    axum::serve(listener, app).await.unwrap();
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
    
    println!("{:?}", 34);
    loop {
        tokio::select! {
            // Receive trade from the broadcast channel
            Ok(bytes) = rx.recv() => {


                if socket.send(Message::Binary(bytes.into())).await.is_err() {
                    break; 
                }

            }
            // Send Heartbeat Ping to keep connection alive
            _ = heartbeat_interval.tick() => {
                if socket.send(Message::Ping(vec![].into())).await.is_err() {
                    break;
                }
            }
        }
    }
}