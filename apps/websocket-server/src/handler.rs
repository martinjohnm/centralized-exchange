use std::sync::Arc;

use tokio::sync::broadcast;
use tokio_stream::StreamMap;
use crate::AppState;
use axum::{extract::{ws::{Message as WsMessage, WebSocket}}};

pub async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {

    let mut subscriptions = StreamMap::new();
    
    loop {
        
    }
}
