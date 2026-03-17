use rust_decimal::Decimal;
use std::collections::{BTreeMap, VecDeque};

use crate::trade::model::{Order, Side};

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

    pub fn add_order(&mut self, order: Order) {

        // use the side defined inside the order struct
        let side_map = match order.side {
            Side::Ask => &mut self.asks,
            Side::Bid => &mut self.bids
        };

        // Entry API handles the rest
        side_map
            .entry(order.price)
            .or_insert_with(|| VecDeque::new())
            .push_back(order);
    }

    // Fn to get best bid (Highest)
    pub fn best_bid(&self) -> Option<Decimal> {
        self.bids.keys().next_back().cloned()
    }
    // Fn to get best ask (Lowest)
    pub fn best_ask(&self) -> Option<Decimal> {
        self.asks.keys().next().cloned()
    }

    // get all the vecdeque array of orders of a price level 
    // which is Optin<&mut VecDeque(Order)> the ref & engine to modify list of orders(matching them) without taking 
    // the ownership of the entire orderbook

    // Since you are returning a mutable reference, remember: While the Engine is holding that &mut VecDeque, the OrderBook is "locked."
    // We cannot call self.orderbook.best_ask() while we are still holding the mutable reference from get_level_mut.
    pub fn get_level_mut(&mut self , price: Decimal, side: Side) -> Option<&mut VecDeque<Order>> {
        match side {
            Side::Bid => self.bids.get_mut(&price),
            Side::Ask => self.asks.get_mut(&price)
        }
    }

    pub fn remove_level(&mut self, price: Decimal, side : Side) {
        match side {
            Side::Bid => self.bids.remove(&price),
            Side::Ask => self.asks.remove(&price)
        };
    }
}


#[cfg(test)]
mod orderbook_tests;