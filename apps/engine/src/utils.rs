use std::{collections::HashMap, fmt::format, str, u32};

use serde::Deserialize;

// Load the central file
#[derive(Deserialize, Debug, Clone)]
pub struct MarketConfig {
    pub base : String,
    pub quote: String,
    pub queue_prefix : String
}
#[derive(Deserialize, Debug, Clone)]
pub struct AssetRegistry {
    markets : HashMap<MarketId, InternalMarketConfig>,
    asset_names: HashMap<AssetId, String>,
    symbol_to_id : HashMap<String, AssetId>
}
#[derive(Deserialize, Debug, Clone)]
pub struct InternalMarketConfig {
    market_id: u32,
    base_id : u32,
    quote_id : u32,
}

type AssetPairName = String;
type AssetId = u32;
type MarketId = u32;

pub fn load_markets() -> HashMap<AssetPairName, MarketConfig> {
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


pub fn initialize_registry() -> AssetRegistry {
    
    // 1. Load the raw JSON
    let raw_data: HashMap<AssetPairName, MarketConfig> = load_markets();

    // 2. Prepare the empty Registry
    let mut registry = AssetRegistry {
        markets : HashMap::new(),
        asset_names: HashMap::new(),
        symbol_to_id : HashMap::new()
    };

    // 3. Temporary state 
    let mut asset_counter :  AssetId  = 1;
    let mut market_counter:  MarketId = 1;

    // this map ensures "USDT" gets the same ID regardless of market
    let mut name_to_asset_id: HashMap<String, AssetId> = HashMap::new();

    for (symbol, config) in raw_data {
        // ---- CLosure: The ID generator--------------------
        // This logic checks if an asset (like "BTC") already has a number.
        // If not, it assigns the next available integer.

        let mut get_asset = |name: &String| -> AssetId {
            if let Some(&id) = name_to_asset_id.get(name) {
                id
            } else {
                let id = asset_counter;
                name_to_asset_id.insert(name.clone(), id);
                registry.asset_names.insert(id, name.clone());
                asset_counter += 1;
                id
            }
        };

        // --------- MAPPING --------------
        let base_id = get_asset(&config.base);
        let quote_id = get_asset(&config.quote);

        let m_id = market_counter;

        // Create the internal config
        registry.markets.insert(m_id, InternalMarketConfig {
            market_id : m_id,
            base_id,
            quote_id
        });

        // Map the human string "BTC_USDT" to the internal Id 1;
        registry.symbol_to_id.insert(symbol, m_id);

        market_counter += 1;

    }

    println!("✅ Registry Initialized: {} Markets, {} Assets", market_counter - 1, asset_counter - 1);
    registry
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
