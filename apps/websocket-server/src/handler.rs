use std::sync::Arc;

use tokio::sync::broadcast;
use tokio_stream::StreamMap;
use crate::AppState;
use axum::{extract::{ws::{Message as WsMessage, WebSocket}}};

pub async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {

    let mut subscriptions = StreamMap::new();
    let mut heartbeat = tokio::time::interval(tokio::time::Duration::from_secs(10));

    loop {
        tokio::select! {
            // Leg 1. Receive json from frontend
            // Leg 2. Receive Data from the subscribed market
            // Leg 3. Heartbeat
        }
        
    }
}
