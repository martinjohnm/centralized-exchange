use std::sync::Arc;

use tokio::sync::broadcast;
use crate::AppState;
use axum::{extract::{ws::{Message as WsMessage, WebSocket}}};

pub async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
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
