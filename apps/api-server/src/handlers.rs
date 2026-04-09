use axum::{Json, extract::{Query, State}, http::StatusCode, response::IntoResponse};
use sqlx::query_as;
use crate::model::{AppState, Kline, KlineParams, SeedRequest, Status};



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
    // 1. Start a Database Transaction
    // This ensures either BOTH queries succeed, or NEITHER do.
    let mut tx = match state.db.begin().await {
        Ok(t) => t,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, format!("Transaction failed: {}", e)).into_response();
        }
    };

    // 2. Ensure the User exists (Parent Table)
    // Using .bind() for safety and to avoid macro compile-time errors
    let user_result = sqlx::query(
        "INSERT INTO users (id, username, email) 
         VALUES ($1, $2, $3) 
         ON CONFLICT (id) DO NOTHING"
    )
    .bind(payload.user_id)                         // $1
    .bind(format!("user_{}", payload.user_id))     // $2
    .bind(format!("{}@example.com", payload.user_id)) // $3
    .execute(&mut *tx) 
    .await;

    if let Err(e) = user_result {
        return (StatusCode::INTERNAL_SERVER_ERROR, format!("User step failed: {}", e)).into_response();
    }

    // 3. Seed the Balance (Child Table)
    // Uses EXCLUDED to add to existing balance if they already have that asset
    let balance_result = sqlx::query(
        "INSERT INTO balances (user_id, asset, available, locked) 
         VALUES ($1, $2, $3, $4) 
         ON CONFLICT (user_id, asset) 
         DO UPDATE SET 
            available = balances.available + EXCLUDED.available,
            locked = balances.locked + EXCLUDED.locked"
    )
    .bind(payload.user_id)   // $1
    .bind(&payload.asset)    // $2
    .bind(payload.available) // $3
    .bind(payload.locked)    // $4
    .execute(&mut *tx)
    .await;

    // 4. Finalize the Transaction
    match balance_result {
        Ok(_) => {
            // Commit makes the changes permanent in the DB
            if let Err(e) = tx.commit().await {
                (StatusCode::INTERNAL_SERVER_ERROR, format!("Commit failed: {}", e)).into_response()
            } else {
                (StatusCode::CREATED, "User and Balance seeded successfully").into_response()
            }
        }
        Err(e) => {
            // If we hit an error here (like a constraint violation), 
            // the transaction 'tx' is dropped and automatically rolls back.
            eprintln!("Database error during balance seed: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Balance step failed: {}", e)).into_response()
        }
    }
}


pub async fn get_klines(
    State(state) : State<AppState>,
    Query(params) : Query<KlineParams>
) -> impl IntoResponse {

    // 1. match the table name (got rid of possible sql injection )
    let table_name = match params.interval.as_str() {
        "1m" => "klines_1m",
        "5m" => "klines_5m",
        "15m" => "klines_15m",
        _ => return (StatusCode::BAD_REQUEST, "Invalid interval").into_response(),
    };

    // The query 
    let query_str = format!(
        "SELECT bucket, open, high, low, close, volume 
         FROM {} 
         WHERE symbol = $1 
         ORDER BY bucket DESC 
         LIMIT 500",
         table_name
    );
    let klines = query_as::<_, Kline>(&query_str)
        .bind(&params.symbol)
        .fetch_all(&state.db)
        .await;

    match klines {
        Ok(data) => Json(data).into_response(),
        Err(e) => {
            eprintln!("Klines fetch error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "failed to fetch klines").into_response()
        }
    }
}