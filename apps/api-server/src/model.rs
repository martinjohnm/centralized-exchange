use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres, prelude::FromRow, types::chrono::{DateTime, Utc}};


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
    pub user_id: i64,
    pub asset: String,     // Matches your 'asset' column
    pub available: Decimal, // Matches NUMERIC
    pub locked: Option<Decimal>,
}

#[derive(Debug, Serialize, Deserialize,FromRow)]
pub struct Kline {
    pub bucket: DateTime<Utc>,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
}

#[derive(Deserialize, Debug)]
pub struct KlineParams {
    pub symbol: String,
    pub interval: String, // e.g., "1m", "1h"
    pub limit : i64
}