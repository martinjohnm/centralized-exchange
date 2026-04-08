use axum::Json;

use crate::model::Status;



pub async fn handler() -> &'static str {
    "Hello john"
}

pub async fn get_status() -> Json<Status> {
    Json(Status {
        active: true,
        engine_tps : 10000 as u64
    })
}