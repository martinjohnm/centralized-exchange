use std::collections::HashMap;

use serde::Deserialize;

// Load the central file
#[derive(Deserialize, Debug, Clone)]
pub struct MarketConfig {
    pub base : String,
    pub quote: String,
    pub queue_prefix : String
}

pub fn load_markets() -> HashMap<String, MarketConfig> {
    // This macro looks relative to the FILE it is written in.
    // From src/trade/model.rs, we need to go up 4 times to hit root.
    let data = include_str!("../../../markets.json"); 
    
    serde_json::from_str(data).expect("JSON was not well-formatted")
}

impl MarketConfig {
    pub fn get_redis_key(&self) -> String {
        format!("{}:{}_{}", self.queue_prefix, self.base, self.quote)
    }

    pub fn get_symbol(&self) -> String {
        format!("{}_{}", self.base, self.quote)
    }
}