use std::collections::HashMap;

use rust_decimal_macros::dec;
use rust_decimal::Decimal;

use crate::model::exchange_proto::{AssetId, MarketId};

#[derive(Debug, Clone, Copy)]
pub struct MarketConfig {
    pub market_id: MarketId,
    pub base_asset: AssetId,
    pub quote_asset: AssetId,
    pub min_order_size: Decimal,
    pub redis_key: &'static str,
}

impl MarketConfig {
    /// This is your NEW "load_markets". It's O(1) and zero-allocation.
    pub fn from_id(id: MarketId) -> Self {
        match id {
            MarketId::BtcUsdt => Self {
                market_id: MarketId::BtcUsdt,
                base_asset: AssetId::Btc,
                quote_asset: AssetId::Usdt,
                min_order_size: dec!(1),
                redis_key: "trades:btc_usdt",
            },
            MarketId::EthUsdt => Self {
                market_id: MarketId::EthUsdt,
                base_asset: AssetId::Eth,
                quote_asset: AssetId::Usdt,
                min_order_size: dec!(1),
                redis_key: "trades:eth_usdt",
            },
            _ => panic!("Market ID {:?} not configured in Rust registry!", id),
        }
    }
}

pub fn load_markets_from_proto() -> HashMap<MarketId, MarketConfig> {
    let mut map = HashMap::new();

    // 1. Define which markets are active in this environment
    let active_markets = vec![
        MarketId::BtcUsdt,
        MarketId::EthUsdt,
    ];

    // 2. Build the map using the from_id logic we created
    for id in active_markets {
        map.insert(id, MarketConfig::from_id(id));
    }

    map
}