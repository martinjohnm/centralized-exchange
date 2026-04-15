use std::{os::linux::raw::stat, sync::Arc};

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use prost::Message;
use redis::AsyncCommands;
use crate::model::{AppState, exchange_proto::{ExchangeRequest, exchange_request::Action}};



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

    // 5. Success
    StatusCode::ACCEPTED.into_response()
}

pub async fn cancel_order() {

}