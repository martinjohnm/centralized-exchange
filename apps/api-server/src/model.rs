use serde::Serialize;
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