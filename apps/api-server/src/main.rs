use std::{env::var, error::Error, net::SocketAddr};

use axum::{Router, routing::get};
use sqlx::postgres::PgPoolOptions;

use crate::{handlers::{get_status, handler}, model::AppState};
mod handlers;
mod model;



#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let db_url = var("DATABASE_URL").expect("Database url must be set");

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&db_url)
        .await?;

    let state = AppState {
        db: pool
    };
    
    let app = Router::new()
        .route("/", get(handler))
        .route("/status", get(get_status));

    let addr = SocketAddr::from(([0,0,0,0], 3000));
    println!("listening on {:?}", addr);

    // run the server 
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
