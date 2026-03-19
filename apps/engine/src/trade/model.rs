use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs;
use std::str::FromStr;


#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Side {
    Bid,
    Ask
}
#[derive(Debug, Clone)]
pub struct Order {
    pub id : u64,
    pub user_id : u64,
    pub quantity : Decimal,
    pub price : Decimal,
    pub side : Side
}

impl Order {
    pub fn is_self_trade(&self, other: &Order) -> bool {
        self.user_id == other.user_id
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Trade {
    pub maker_id : u64, 
    pub taker_id : u64,
    pub price : Decimal,
    pub quantity : Decimal,
    pub taker_side : Side,
    pub maker_side : Side
}
#[derive(Deserialize, Debug, Clone)]
pub struct MarketConfig {
    pub base : String,
    pub quote: String,
    pub queue_prefix : String
}

// Load the central file
pub fn load_markets() -> HashMap<String, MarketConfig> {
    // This macro looks relative to the FILE it is written in.
    // From src/trade/model.rs, we need to go up 4 times to hit root.
    let data = include_str!("../../../../markets.json"); 
    
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRequest {
    pub user_id: u64,
    pub price: Decimal,
    pub quantity: Decimal, 
    pub side: Side,
    pub symbol: String,    // "BTC_USDT"
}

// 2. THE GENERATED TYPES (The "Dirty" ones from build.rs)
pub mod exchange_proto {
    include!(concat!(env!("OUT_DIR"), "/exchange.rs"));
}

// --- THE BRIDGE (The Conversion Logic) ---
// This is the "TryFrom" trait. It attempts to turn the "Dirty" 
// Protobuf struct into your "Clean" OrderRequest struct.

impl TryFrom<exchange_proto::OrderRequestProto> for OrderRequest {
    type Error = String;

    fn try_from(proto: exchange_proto::OrderRequestProto) -> Result<Self, Self::Error> {
        let side = match proto.side {
            0 => Side::Bid,
            1 => Side::Ask,
            _ => return Err("Invalid Side".to_string()),
        };

        Ok(OrderRequest {
            user_id: proto.user_id,
            symbol: proto.symbol,
            price: Decimal::from_str(&proto.price).map_err(|_| "Invalid price".to_string())?,
            quantity: Decimal::from_str(&proto.quantity).map_err(|_| "Invalid qty".to_string())?,
            side,
        })
    }
}