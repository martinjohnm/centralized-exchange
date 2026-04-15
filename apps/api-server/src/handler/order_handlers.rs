use std::{os::linux::raw::stat, sync::Arc, time::Duration};

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use prost::Message;
use redis::AsyncCommands;
use tokio::time::timeout;
use crate::model::{AppState, exchange_proto::{ExchangeRequest, ExecutionReport, exchange_request::Action}};
use futures_util::StreamExt;


pub async fn create_order(
    State(state): State<AppState>,
    Json(exchange_req) : Json<ExchangeRequest>
) -> impl IntoResponse {
// 1. Extract the action and ensure it's a Create action
    let create_payload = match &exchange_req.action {
        Some(Action::Create(c)) => c,
        Some(_) => return (StatusCode::BAD_REQUEST, "Only CreateOrder allowed here").into_response(),
        None => return (StatusCode::BAD_REQUEST, "No action provided").into_response(),
    };

    // 2. Resolve the Market and its corresponding Queue
    let market = create_payload.market(); // Proto-generated helper
    let market_config = match state.markets.get(&market) {
        Some(config) => config,
        None => return (StatusCode::BAD_REQUEST, "Market not supported").into_response(),
    };

    let client_id = create_payload.client_id.to_string();

    let conn_a = state.redis.get().await.unwrap();
    let mut pubsub = deadpool_redis::Connection::take(conn_a).into_pubsub();

    if let Err(_) = pubsub.subscribe(&client_id).await {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    let mut stream = pubsub.into_on_message();


    
    // 3. Serialization
    let mut buf = Vec::new();
    if let Err(_) = exchange_req.encode(&mut buf) {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Encoding failed").into_response();
    }

    // 4. Redis Ingress (WAL)
    let mut conn = match state.redis.get().await {
        Ok(c) => c,
        Err(_) => return (StatusCode::SERVICE_UNAVAILABLE, "Redis Busy").into_response(),
    };

    // LPUSH to the specific market queue (e.g., "orders:btc_usdt")
    let _: () = conn.lpush(market_config.redis_key, buf).await.unwrap();

    tokio::pin!(stream);

    // 5. Await Response (The final evaluation)
    match timeout(Duration::from_secs(2), stream.next()).await {
        Ok(Some(msg)) => {
        // 1. Get the raw bytes from Redis
        let payload: Vec<u8> = msg.get_payload().unwrap();
        
        // 2. Decode the bytes into your Protobuf ExecutionReport struct
        // Replace 'ExecutionReport' with the actual name in your proto file
        match ExecutionReport::decode(&*payload) {
            Ok(report) => {
                // 3. Return as JSON (Axum handles the serialization to JSON string)
                (StatusCode::OK, Json(report)).into_response()
            }
            Err(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to decode engine response").into_response()
            }
        }
    }
    Ok(None) => (StatusCode::INTERNAL_SERVER_ERROR, "Connection Lost").into_response(),
    Err(_) => (StatusCode::GATEWAY_TIMEOUT, "Engine Timeout").into_response(),
    }
}

pub async fn cancel_order() {

}