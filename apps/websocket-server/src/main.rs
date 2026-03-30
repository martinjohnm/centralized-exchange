use std::sync::Arc;

use axum::{Router, extract::{WebSocketUpgrade, ws::WebSocket}, response::Response, routing::get};
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
        let client = redis::Client::open("redis:://127.0.0.1/").expect("Redis connection failed");
        let mut conn = client.get_multiplexed_async_connection().await.expect("Redis connetcon failde");
        let mut pubsub = client
            .get_async_pubsub()
            .await
            .expect("Pubsub connection failed");

        pubsub.subscribe("trades:btcusdt").await.unwrap();

        
    });

    // 1. simple Axum router
    let app = Router::new()
        .route("/ws", get(handler));

    // 2. create a tcp listener
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();

    println!("Axum websocket listening on 8080");

    axum::serve(listener, app).await.unwrap();
}

async fn handler(ws : WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    println!("Connection establishes");


    // Test loop
    loop {
        let heartbeat = vec![1,2,3,4,5];

        if socket.send(axum::extract::ws::Message::Binary(heartbeat.into())).await.is_err() {
            break;
        }

        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
}