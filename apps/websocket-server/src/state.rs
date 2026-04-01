use std::{collections::HashMap};

use tokio::sync::{RwLock, broadcast};



pub struct AppState {
    // Map: "btc_usdt" -> broadcast::Sender
    pub market_map : RwLock<HashMap<String, broadcast::Sender<Vec<u8>>>>
}

impl AppState {
    pub async fn get_tx(&self, market: &str) -> Option<broadcast::Sender<Vec<u8>>> {
        let map = self.market_map.read().await;
        map.get(market).cloned()
    }
}