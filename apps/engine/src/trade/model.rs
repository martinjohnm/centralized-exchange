use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};




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


pub struct Trade {
    pub maker_id : u64, 
    pub taker_id : u64,
    pub price : Decimal,
    pub quantity : Decimal,
    pub taker_side : Side,
    pub maker_side : Side
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRequest {
    pub user_id: u64,
    pub price: Decimal,
    pub quantity: Decimal, 
    pub side: Side,
    pub symbol: String,    // "BTC_USDT"
}