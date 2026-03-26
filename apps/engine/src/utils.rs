use std::{collections::HashMap, fmt::format, u32};

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
    pub fn get_base(&self) -> String {
        format!("{}", self.base)
    }
    pub fn get_quote(&self) -> String {
        format!("{}", self.quote)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_markets_validity() {
        // This will panic if the JSON is malformed or the file is missing,
        // which is exactly what we want to catch in a test.
        let markets = load_markets();
        
        // Assert that the map isn't empty
        assert!(!markets.is_empty(), "Markets map should not be empty");

        // Optional: Check for a specific key you know should exist
        // assert!(markets.contains_key("BTC-USD"));
    }
}

pub fn initialize_registry()  {
    let raw_data: HashMap<String, MarketConfig> = load_markets();

    for r in raw_data {
        println!("{:?}", r);
    }
    // let mut registry = AssetRegistry {
    //     markets: HashMap::new(),
    //     asset_names: HashMap::new(),
    //     symbol_to_id: HashMap::new(),
    // };

    // let mut asset_counter: AssetId = 1 as u32;
    // let mut market_counter: MarketId = 1 as u32;
    // let mut name_to_asset_id: HashMap<String, AssetId> = HashMap::new();

    // for (symbol, config) in raw_data {
    //     // Helper to get or create AssetId for "BTC", "USDT", etc.
    //     let get_asset = |name: &String, counter: &mut AssetId, map: &mut HashMap<String, AssetId>, names: &mut HashMap<AssetId, String>| {
    //         *map.entry(name.clone()).or_insert_with(|| {
    //             let id = *counter;
    //             names.insert(id, name.clone());
    //             *counter += 1;
    //             id
    //         })
    //     };

    //     let base_id = get_asset(&config.base, &mut asset_counter, &mut name_to_asset_id, &mut registry.asset_names);
    //     let quote_id = get_asset(&config.quote, &mut asset_counter, &mut name_to_asset_id, &mut registry.asset_names);

    //     let m_id = market_counter;
    //     registry.markets.insert(m_id, InternalMarketConfig {
    //         market_id: m_id,
    //         base_id,
    //         quote_id,
    //     });
    //     registry.symbol_to_id.insert(symbol, m_id);
    //     market_counter += 1;
    // }

    
}

