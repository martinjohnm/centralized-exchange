use std::{env::var, error::Error, net::SocketAddr};

use axum::{Router, routing::{delete, get, post, put}};
use deadpool_redis::{Config, Runtime};
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::{Any, CorsLayer};

use crate::{handler::{cancel_order, create_order, get_klines, get_status, seed_user_balance}, model::AppState, utils::load_markets_from_proto};
mod handler;
mod model;
mod utils;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    
    let db_url = var("DATABASE_URL").expect("Database url must be set");
    
    let markets = load_markets_from_proto();
    let redis_url = var("REDIS_URL")
        .expect("REDIS_URL must be set in .env or system environment");
    println!("{:?}", redis_url);


    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&db_url)
        .await?;


        // 3. Initialize Redis Pool (Deadpool)
    let cfg = Config::from_url(redis_url);
    let redis_pool = cfg.create_pool(Some(Runtime::Tokio1))?;

    let state = AppState {
        db: pool,
        redis : redis_pool,
        markets
    };

    // add the cors layer to hit the frontned 
    let cors = CorsLayer::new()
        .allow_origin("http://localhost:5173".parse::<axum::http::HeaderValue>().unwrap())
        .allow_methods(Any)
        .allow_headers(Any);
    
    let app = Router::new()
        .route("/status", get(get_status))
        .route("/seed", post(seed_user_balance))
        .route("/get-klines", get(get_klines))
        .route("/order/create", post(create_order))
        .route("/order/cancel", delete(cancel_order))
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from(([0,0,0,0], 3000));
    println!("listening on {:?}", addr);

    // run the server 
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
