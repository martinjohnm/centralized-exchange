use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;

use crate::model::{AppState, Status};

#[derive(Deserialize)]
struct SeedRequest {
    user_id: i32,
    symbol: String,
    amount: f64,
}


pub async fn handler() -> &'static str {
    "Hello john"
}

pub async fn get_status() -> Json<Status> {
    Json(Status {
        active: true,
        engine_tps : 10000 as u64
    })
}
// Async DB operation handler
async fn seed_user_balance(
    State(state): State<AppState>, // Extracting the pooled connection
    Json(payload): Json<SeedRequest>, // Extracting the JSON body
) -> Result<(StatusCode, String), (StatusCode, String)> {
    
    // Asynchronous query: This task will "yield" to the runtime 
    // while waiting for Postgres to finish the insert.
    sqlx::query(
        "INSERT INTO balances (user_id, symbol, total_amount) 
         VALUES ($1, $2, $3) 
         ON CONFLICT (user_id, symbol) 
         DO UPDATE SET total_amount = balances.total_amount + $3"
    )
    .bind(payload.user_id)      // $1
    .bind(&payload.symbol)     // $2
    .bind(payload.amount)      // $3
    .execute(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::CREATED, "Balance seeded successfully".to_string()))
}