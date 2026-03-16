use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::trade::orderbook::{self, Order, Orderbook, Side};


pub struct Trade {
    pub maker_id : u64, 
    pub taker_id : u64,
    pub price : Decimal,
    pub amount : Decimal
}

pub struct MatchingEngine {
    pub orderbook : Orderbook,
    current_order_id : u64
}

impl MatchingEngine {

    pub fn new() -> Self {
        Self { 
            orderbook: Orderbook::new(), 
            current_order_id: 1 // Start at 1
        }
    }

    /// Processes an incoming order against the existing order book.
    /// 
    /// #Data Flow: 
    /// 1. **Peek**: Check `best price` opposite side.
    /// 2. **Match**: If price cross, enter the matching loop.
    /// 3. **Fill**: Substract volume from Taker and Maker.
    /// 4. **Cleanup**: Remove price levels if they hit zero volume.
    /// 5. **Rest**: Add remaining Taker volume to the book as the Limit order.

    pub fn process_order(&mut self, mut taker_order: Order) -> Vec<Trade> {
        let mut trades = Vec::new();

        // loop on the amount while it is non zero
        while taker_order.amount > dec!(0) {
            // 1. Look for the best price on the oposite side
            let best_price = match taker_order.side {
                Side::Bid => self.orderbook.best_ask(),
                Side::Ask => self.orderbook.best_bid()
            };

            // 2. Check if the best price "crosses the spread"
            match best_price {
                Some(price) if self.is_match(taker_order.price, price, taker_order.side) => {
                    // 3. Match logic here
                    let side_to_match = match taker_order.side {
                        Side::Bid => Side::Ask,
                        Side::Ask => Side::Bid
                    };

                    // scoped borrow for maker_order in the Btreemap (&mut VecDeque<Order>)
                    {
                        if let Some(orders_at_level) = self.orderbook.get_level_mut(price, side_to_match) {
                            while taker_order.amount > dec!(0) && !orders_at_level.is_empty() {
                                let mut maker_order = orders_at_level.pop_front().unwrap();

                                // The math how much can we actually trade? 
                                let match_amount = taker_order.amount.min(maker_order.amount);

                                taker_order.amount -= match_amount;
                                maker_order.amount -= match_amount;
                            }
                        }
                    }
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