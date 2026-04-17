use std::{os::linux::raw::stat, sync::Arc, time::Duration};

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use prost::Message;
use redis::AsyncCommands;
use serde_json::json;
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
// 2. Encode to Protobuf
    let mut buf = Vec::new();
    if let Err(_) = exchange_req.encode(&mut buf) {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Encoding failed").into_response();
    }

    // 3. Push to Redis (The Single Source of Truth)
    let mut conn = match state.redis.get().await {
        Ok(c) => c,
        Err(_) => return (StatusCode::SERVICE_UNAVAILABLE, "Redis Busy").into_response(),
    };

    // LPUSH to the matching engine's ingress queue
    let _: () = conn.lpush(&market_config.redis_key, buf).await.unwrap();

    // 4. Return 202 Accepted
    // This tells the user: "We got it, check your WebSocket for the results."
    (
        StatusCode::ACCEPTED, 
        Json(json!({
            "status": "success",
            "message": "Order placed",
            "market": market_config.redis_key
        }))
    ).into_response()
}

pub async fn cancel_order() {

}