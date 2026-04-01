use std::sync::Arc;

use axum::{Router, extract::{State, WebSocketUpgrade, ws::{Message as WsMessage, WebSocket}}, response::Response, routing::get};
use futures_util::StreamExt;
use prost::Message as ProtoMessage;
use tokio::sync::broadcast;

use crate::{aggregator::start_aggregator, candle::Candle, handler::handle_socket, model::{InternalTrade, exchange_proto::Trade}, redis::start_redis_listener, state::AppState};




mod model;
mod state;
mod handler;
mod candle;
mod aggregator;
mod redis;
#[tokio::main]
async fn main() {

    // 1. state initialization
    let state = Arc::new(AppState::new());

    // 2. Define which markets supports
    let markets = vec!["btcusdt".to_string(), "ethusdt".to_string()];

    for market_symbol in markets {

        let (broadcast_tx, _) = broadcast::channel(1024);
        let (agg_tx, agg_rx) = tokio::sync::mpsc::channel(10000);

        state.market_map.write().await.insert(market_symbol.clone(), broadcast_tx.clone());

        tokio::spawn(start_aggregator(agg_rx, broadcast_tx));

        tokio::spawn(start_redis_listener(market_symbol, agg_tx));
    }

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