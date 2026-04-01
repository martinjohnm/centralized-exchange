use futures_util::StreamExt;

pub async fn start_redis_listener(
    market: String, 
    agg_tx: tokio::sync::mpsc::Sender<Vec<u8>>
) {
    // 1. Establish connection
    let client = redis::Client::open("redis://127.0.0.1:6379/")
        .expect("Failed to create Redis client");
        
    let mut pubsub = client
        .get_async_pubsub()
        .await
        .expect("Failed to get async pubsub");

    // 2. Subscribe to the specific market channel
    // Example: "trades:btc_usdt"
    let channel_name = format!("trades:{}", market);
    pubsub.subscribe(&channel_name)
        .await
        .expect(&format!("Failed to subscribe to {}", channel_name));

    println!("📡 Redis Listener started for: {}", channel_name);

    let mut stream = pubsub.on_message();

    // 3. Listen and forward to Aggregator
    while let Some(msg) = stream.next().await {
        match msg.get_payload::<Vec<u8>>() {
            Ok(payload) => {
                // We send the raw Protobuf bytes to the Aggregator
                if let Err(e) = agg_tx.send(payload).await {
                    eprintln!("❌ Aggregator for {} closed: {}", market, e);
                    break;
                }
            }
            Err(e) => eprintln!("❌ Redis payload error on {}: {}", market, e),
        }
    }
}