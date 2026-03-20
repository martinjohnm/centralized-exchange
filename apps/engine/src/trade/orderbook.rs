use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::{BTreeMap, HashMap, VecDeque};

use crate::trade::model::{Order, OrderMetadata, Side};

pub struct Orderbook {
    // Bids : Sorted descending (Highest price first)
    pub bids : BTreeMap<Decimal, VecDeque<Order>>,

    // Asks : Sorted Ascending (Lowest pirce first)
    pub asks : BTreeMap<Decimal, VecDeque<Order>>,

    // For cancellation : OrderId -> (Price, Side)
    // This tells un where to find the order in the btreemap
    pub orders_lookup: HashMap<u64, OrderMetadata>
}

impl Orderbook {
    pub fn new() -> Self {
        Self { 
            bids: BTreeMap::new(), 
            asks: BTreeMap::new(),
            orders_lookup: HashMap::new()
        }
    }

    pub fn add_order(&mut self, order:Order) {

        // 1. Register in the Lookup HashMap first
        // We store the price and side so the Cancel function knows exactly where to look
        self.orders_lookup.insert(order.id, OrderMetadata {
            price: order.price,
            side: order.side,
        });
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

    pub fn cancel_order(&mut self, order_id : u64) -> bool {
        // 1. find where the order is (0(1) Lookup in a hashmap)
        // If the ID isnt in the lookup, its already filled or never existed

        let metadata = match self.orders_lookup.remove(&order_id){
            Some(m) => m,
            None => return false
        };

        // 2. Go to the specific Price level in the correct Btreemap
        let side_map = match metadata.side {
            Side::Ask => &mut self.asks,
            Side::Bid => &mut self.bids
        };

        if let Some(queue) = side_map.get_mut(&metadata.price) {
            // 3. Remove the order from the VecDequeu
            // retain keeps all elements EXCEPT the one with ID
            queue.retain(|o| o.id != order_id);


            // 4. CLEANUP: If this was the last order at this price,
            // Remove the price key to save RAM 
            if queue.is_empty() {
                side_map.remove(&metadata.price);
            }
            return true;
        } 

        return true;
    }

    // Fn to get best bid (Highest)
    pub fn best_bid(&self) -> Option<Decimal> {
        self.bids.keys().next_back().cloned()
    }
    // Fn to get best ask (Lowest)
    pub fn best_ask(&self) -> Option<Decimal> {
        self.asks.keys().next().cloned()
    }

    pub fn get_order_book_stats(&self) {
        let bid_prices = self.bids.len();
        let total_bids: usize = self.bids.values().map(|v| v.len()).sum();

        let ask_prices = self.asks.len();
        let total_asks: usize = self.asks.values().map(|v| v.len()).sum();

        println!(
            "[Book Stats] Bids: {} (at {} prices) | Asks: {} (at {} prices)",
            total_bids, bid_prices, total_asks, ask_prices
        );
    }

    pub fn calculate_mid_fair_price(&self) -> Option<Decimal>  {
        // next_back to get the highest key (Best bid)
        self.best_bid().zip(self.best_ask())
            .map(|(bid, ask) | (bid + ask) / dec!(2))
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