use rust_decimal::Decimal;
use std::collections::{BTreeMap, VecDeque};

#[derive(Debug, Clone)]
pub struct Order {
    pub id : u64,
    pub amount : Decimal,
    pub price : Decimal
}

pub struct Orderbook {
    // Bids : Sorted descending (Highest price first)
    pub bids : BTreeMap<Decimal, VecDeque<Order>>,

    // Asks : Sorted Ascending (Lowest pirce first)
    pub asks : BTreeMap<Decimal, VecDeque<Order>>
}

impl Orderbook {
    pub fn new() -> Self {
        Self { 
            bids: BTreeMap::new(), 
            asks: BTreeMap::new()
        }
    }

    // Fn to get best bid (Highest)
    pub fn best_bid(&self) -> Option<Decimal> {
        self.bids.keys().next_back().cloned()
    }
    // Fn to get best ask (Lowest)
    pub fn best_ask(&self) -> Option<Decimal> {
        self.asks.keys().next().cloned()
    }
}