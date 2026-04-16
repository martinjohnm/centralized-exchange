use std::{collections::HashMap};

use redis::Client;
use tokio::sync::{RwLock, broadcast};




pub struct AppState {
    // Map: "btc_usdt" -> broadcast::Sender
    pub market_map : RwLock<HashMap<String, broadcast::Sender<Vec<u8>>>>,
    pub redis_client: redis::Client,
}

impl AppState {
    pub fn new(redis_url : String) -> Self {
        let client = Client::open(redis_url)
            .expect("Invalid Redis URL");
        Self {
            market_map: RwLock::new(HashMap::new()),
            redis_client : client
        }
    }

    pub async fn get_tx(&self, market: &str) -> Option<broadcast::Sender<Vec<u8>>> {
        let map = self.market_map.read().await;
        map.get(market).cloned()
    }
}