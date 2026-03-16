use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::trade::orderbook::{Order, Orderbook, Side};


pub struct Trade {
    pub maker_id : u64, 
    pub taker_id : u64,
    pub price : Decimal,
    pub amount : Decimal
}

pub struct MatchingEngine {
    pub orderbook : Orderbook,
}

impl MatchingEngine {
    pub fn process_order(&mut self, taker_order: Order) -> Vec<> {
        let mut trades = Vec::new();

        while taker_order.price > dec!(0) {
            // 1. Look for the best price on the oposite side
            let best_price = match taker_order.side {
                Side::Bid => self.orderbook.best_ask(),
                Side::Ask => self.orderbook.best_bid()
            };

            // 2. Check if the best price "crosses the spread"
            match best_price {
                Some(price) if self.is_match(taker_order.price, price, taker_order.side) => {
                    // 3. Match logic here
                    // We will pull the level, fill orders, add call remove_level

                }
                _ => {
                    // No match possible: Add whats left to the book
                    self.orderbook.add_order(taker_order);
                    break;
                }
            }

        }
        trades
    }

    fn is_match(&self, taker_price: Decimal, best_price: Decimal, side: Side) -> bool {
        match side {
            Side::Ask => taker_price <= best_price,
            Side::Bid => taker_price >= best_price
        }
    }

}