use std::{error::Error, net::SocketAddr};

use axum::{Router, routing::get};

use crate::handlers::{get_status, handler};
mod handlers;
mod model;



#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    
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
