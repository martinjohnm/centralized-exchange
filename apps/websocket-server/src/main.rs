use axum::{Router, extract::{WebSocketUpgrade, ws::WebSocket}, response::Response, routing::get};



mod model;
mod state;
mod handler;



#[tokio::main]
async fn main() {
    // 1. simple Axum router
    let app = Router::new()
        .route("/ws", get(handler));

    // 2. create a tcp listener
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();

    println!("Axum websocket listening on 8080");

    axum::serve(listener, app).await.unwrap();
}

async fn handler(ws : WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    println!("Connection establishes");


    // Test loop
    loop {
        let heartbeat = vec![1,2,3,4,5];

        if socket.send(axum::extract::ws::Message::Binary(heartbeat.into())).await.is_err() {
            break;
        }

        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
}