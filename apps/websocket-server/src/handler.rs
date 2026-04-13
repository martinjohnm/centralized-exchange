use std::sync::Arc;

use tokio_stream::{StreamExt, StreamMap, wrappers::BroadcastStream};
use crate::{ model::WsRequest, state::AppState};
use axum::{extract::{ws::{Message as WsMessage, WebSocket}}};

pub async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {

    let mut subscriptions = StreamMap::new();
    let mut heartbeat = tokio::time::interval(tokio::time::Duration::from_secs(10));
    
    loop {
        tokio::select! {
            // Leg 1. Receive json COMMANDS (SUBSCRIBE || UNSUBSCRIBE) from frontend
            Some(msg) = socket.recv() => {
                let msg = match msg { Ok(m) => m, Err(_) => break };
                if let Ok(text) = msg.to_text() {
                    if let Ok(req) = serde_json::from_str::<WsRequest>(text) {
                        match req {
                            WsRequest::Subscribe { market, stream } => {
                                // 1. Create the unique Room ID (e.g., "btcusdt:depth")
                                let room_id = format!("{}:{}", market, stream);
                                println!("{}", room_id);

                                // 2. Fetch the broadcast sender from your AppState
                                if let Some(tx) = state.market_map.read().await.get(&room_id) {
                                    let stream_handle = BroadcastStream::new(tx.subscribe());
                                    // Use the room_id as the key so the user can have multiple subs
                                    subscriptions.insert(room_id, stream_handle);
                                }
                            }
                            WsRequest::Unsubscribe { market, stream } => {
                                let room_id = format!("{}:{}", market, stream);
                                subscriptions.remove(&room_id);
                            }
                        }
                    }
                }
            }
            // Leg 2. Receive Data from the subscribed market and forwarding to client
            Some((_market, broadcast_result)) = subscriptions.next() => {
                match broadcast_result {
                    Ok(bytes) => {
                        if socket.send(WsMessage::Binary(bytes.into())).await.is_err() {
                            break; 
                        }
                    }
                    Err(e) => {
                        // This happens if the consumer is too slow for the producer
                        eprintln!("⚠️ Client lagged on market {}: {}", _market, e);
                    }
                }
            }
            // Leg 3. Heartbeat
            _ = heartbeat.tick() => {
                let _ = socket.send(WsMessage::Ping(vec![].into())).await;
            }
        }
        
    }
}
