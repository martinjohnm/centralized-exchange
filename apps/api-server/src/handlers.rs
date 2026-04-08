use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use crate::model::{AppState, SeedRequest, Status};



pub async fn handler() -> &'static str {
    "Hello john"
}

pub async fn get_status() -> Json<Status> {
    Json(Status {
        active: true,
        engine_tps : 10000 as u64
    })
}

#[axum::debug_handler]
// Async DB operation handler
pub async fn seed_user_balance(
    State(state): State<AppState>, // Extracting the pooled connection
    Json(payload): Json<SeedRequest>, // Extracting the JSON body
) -> impl IntoResponse {
    
    // Asynchronous query: This task will "yield" to the runtime 
    // while waiting for Postgres to finish the insert.
    let result = sqlx::query(
        "INSERT INTO balances (user_id, asset, available, locked) 
         VALUES ($1, $2, $3, $4) 
         ON CONFLICT (user_id, asset) 
         DO UPDATE SET available = balances.available + EXCLUDED.available"
    )
    .bind(payload.user_id)   // $1
    .bind(&payload.asset)    // $2
    .bind(payload.available) // $3
    .bind(payload.locked)     // $4
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => (StatusCode::CREATED, "Balance seeded successfully").into_response(),
        Err(e) => {
            eprintln!("Database error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal DB error").into_response()
        }
    }
}