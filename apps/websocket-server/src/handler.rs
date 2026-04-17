use std::sync::Arc;

use tokio_stream::{StreamExt, StreamMap, wrappers::BroadcastStream};
use crate::{ model::{WsRequest, exchange_proto::{ExecutionReport, StreamType, WsOutMessage, ws_out_message::Data}}, state::AppState};
use axum::{extract::{ws::{Message as WsMessage, WebSocket}}};

use futures::stream::{Stream, BoxStream}; // Ensure these are imported
use prost::Message;
// 1. Define the type alias to make it readable
type RedisStream = BoxStream<'static, Vec<u8>>;

pub async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {

    let mut subscriptions = StreamMap::new();
    let mut heartbeat = tokio::time::interval(tokio::time::Duration::from_secs(10));
    
    // Use a specific stream for User Updates so it doesn't get dropped
    let mut user_updates_stream: Option<RedisStream> = None;
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
                            WsRequest::UserUpdates { user_id } => {
                                // Only subscribe if we aren't already subscribed
                                if user_updates_stream.is_none() {
                                    let channel = format!("user:{}", user_id);
                                    println!("channel : {}", channel);
                                    if let Ok(mut pubsub) = state.redis_client.get_async_pubsub().await {
                                        if pubsub.subscribe(&channel).await.is_ok() {
                                            let stream = pubsub.into_on_message().map(|m| {
                                                m.get_payload::<Vec<u8>>().unwrap_or_default()
                                            });
                                            user_updates_stream = Some(Box::pin(stream));
                                            println!("✅ Persistent Redis Sub: {}", channel);
                                        }
                                    }
                                }
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
            // Leg 3: User Updates (Private) - FIXED BRANCH
            // Using a match guard to ensure we only poll if the stream exists
            Some(raw_bytes) = async {
                match user_updates_stream.as_mut() {
                    Some(s) => s.next().await,
                    None => None,
                }
            }, if user_updates_stream.is_some() => {
                if let Ok(report) = ExecutionReport::decode(&raw_bytes[..]) {
                    let out_msg = WsOutMessage {
                        stream: StreamType::UserUpdates as i32,
                        data: Some(Data::ExecutionReport(report)),
                    };
                    
                    let mut buf = Vec::new();
                    if out_msg.encode(&mut buf).is_ok() {
                        if socket.send(WsMessage::Binary(buf.into())).await.is_err() { break; }
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
