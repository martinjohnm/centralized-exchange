use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};


#[derive(Serialize)]
pub struct Status {
    pub active : bool,
    pub engine_tps : u64
}

#[derive(Clone)] // clonable state for different green threads
pub struct AppState {
    pub db : Pool<Postgres>
}
#[derive(Deserialize)]
pub struct SeedRequest {
    pub user_id: i32,
    pub asset: String,     // Matches your 'asset' column
    pub available: Decimal, // Matches NUMERIC
    pub locked: Option<Decimal>,
}