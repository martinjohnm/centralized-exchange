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
    current_order_id : u64
}

impl MatchingEngine {

    pub fn new() -> Self {
        Self { 
            orderbook: Orderbook::new(), 
            current_order_id: 1 // Start at 1
        }
    }

    fn next_id(&mut self) -> u64 {
        let id = self.current_order_id;
        self.current_order_id += 1;
        id
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

                    // --- Matching scope (scoped borrow for maker_order in the Btreemap (&mut VecDeque<Order>)) ---
                    {
                        if let Some(orders_at_level) = self.orderbook.get_level_mut(price, side_to_match) {
                            while taker_order.amount > dec!(0) && !orders_at_level.is_empty() {
                                // mut borrow of the order from the front (most recent)
                                let mut maker_order = orders_at_level.pop_front().unwrap();

                                // The math how much can we actually trade? 
                                let match_amount = taker_order.amount.min(maker_order.amount);

                                taker_order.amount -= match_amount;
                                maker_order.amount -= match_amount;
                                
                                // if the maker partially filled we should put the order where it was
                                if maker_order.amount > dec!(0) {
                                    // Maker was only partially filled, put them back at the FRONT
                                    // Before mut borrow took the order from Vecdeque 
                                    // (we should put it back to the front itself)
                                    orders_at_level.push_front(maker_order);
                                }
                            }
                        }
                    } // ---- Borrow level ends here -----

                    // ---- Check for the level if it is empty -----
                    // If it is empty , We must delete the key from the BtreeMap

                    if let Some(level) = self.orderbook.get_level_mut(price, side_to_match) {
                        if level.is_empty() {
                            // Remove the level from the book
                            self.orderbook.remove_level(price, side_to_match);
                        }
                    }
                   

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buy_order_exact_matching_quantity_and_price() {
        let mut engine = MatchingEngine::new();

        // Add bids 
        engine.orderbook.add_order(Order { id: 1, amount: dec!(1), price: dec!(101), side: Side::Ask });
        engine.orderbook.add_order(Order { id: 2, amount: dec!(1), price: dec!(102), side: Side::Ask });
        engine.orderbook.add_order(Order { id: 3, amount: dec!(1), price: dec!(103), side: Side::Ask });

        // Add bids
        engine.orderbook.add_order(Order { id: 4, amount: dec!(1), price: dec!(99), side: Side::Bid });
        engine.orderbook.add_order(Order { id: 5, amount: dec!(1), price: dec!(98), side: Side::Bid });
        engine.orderbook.add_order(Order { id: 6, amount: dec!(1), price: dec!(97), side: Side::Bid });

        // sent an buy order with amount 1 and price : 101 (exact match for the best ask)
        engine.process_order(Order { id:7, amount: dec!(1), price: dec!(101), side: Side::Bid });

        assert_eq!(engine.orderbook.best_ask(), Some(dec!(102)));
        let asks_at_102 = engine.orderbook.get_level_mut(dec!(102), Side::Ask).unwrap();
        assert_eq!(asks_at_102[0].id, 2);

        println!("{:#?}", engine.orderbook.asks);
        println!("{:#?}", engine.orderbook.bids);

    }
}