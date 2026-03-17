use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::trade::{model::{Order, Side}, orderbook::Orderbook};



pub struct Trade {
    pub maker_id : u64, 
    pub taker_id : u64,
    pub price : Decimal,
    pub quantity : Decimal
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

        // loop on the quantity while it is non zero
        while taker_order.quantity > dec!(0) {
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
                            while taker_order.quantity > dec!(0) && !orders_at_level.is_empty() {
                                // mut borrow of the order from the front (most recent)
                                let mut maker_order = orders_at_level.pop_front().unwrap();

                                // The math how much can we actually trade? 
                                let match_quantity = taker_order.quantity.min(maker_order.quantity);

                                taker_order.quantity -= match_quantity;
                                maker_order.quantity -= match_quantity;
                                
                                // if the maker partially filled we should put the order where it was
                                if maker_order.quantity > dec!(0) {
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
                    // -------- IMPORTANT Loop BREAKING LOGIC ---------
                    // No match possible: (NEver add taker order here because what if the taker
                    // quantity is 0 it should create a zero volume order!!).
                    // Loop will break with the taker_order with quantity (which may or may not be 0)
                    break;
                }
            }

        }

        // 2. After the loop , check if anything left to store.
        if taker_order.quantity > dec!(0) {
            // Taker is partially filled. Adding remaining to the orderbook.
            self.orderbook.add_order(taker_order);
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
    fn test_buy_order_full_fills_and_single_level() {
        let mut engine = MatchingEngine::new();

        // Add bids 
        engine.orderbook.add_order(Order { id: 1, quantity: dec!(1), price: dec!(101), side: Side::Ask });
        engine.orderbook.add_order(Order { id: 2, quantity: dec!(1), price: dec!(102), side: Side::Ask });
        engine.orderbook.add_order(Order { id: 3, quantity: dec!(1), price: dec!(103), side: Side::Ask });

        // Add bids
        engine.orderbook.add_order(Order { id: 4, quantity: dec!(1), price: dec!(99), side: Side::Bid });
        engine.orderbook.add_order(Order { id: 5, quantity: dec!(1), price: dec!(98), side: Side::Bid });
        engine.orderbook.add_order(Order { id: 6, quantity: dec!(1), price: dec!(97), side: Side::Bid });

        // sent an buy order with quantity 1 and price : 101 (exact match for the best ask)
        engine.process_order(Order { id:7, quantity: dec!(1), price: dec!(101), side: Side::Bid });

        assert_eq!(engine.orderbook.best_ask(), Some(dec!(102)));
        let asks_at_102 = engine.orderbook.get_level_mut(dec!(102), Side::Ask).unwrap();
        assert_eq!(asks_at_102[0].id, 2);

        println!("{:#?}", engine.orderbook.asks);
        println!("{:#?}", engine.orderbook.bids);

    }

    #[test]
    fn test_buy_order_single_level_low_price_fill_complete() {
        let mut engine = MatchingEngine::new();

        // Add bids 
        engine.orderbook.add_order(Order { id: 1, quantity: dec!(3), price: dec!(100), side: Side::Ask });
        engine.orderbook.add_order(Order { id: 2, quantity: dec!(1), price: dec!(101), side: Side::Ask });
        engine.orderbook.add_order(Order { id: 3, quantity: dec!(2), price: dec!(102), side: Side::Ask });

        // Add bids
        engine.orderbook.add_order(Order { id: 4, quantity: dec!(1), price: dec!(99), side: Side::Bid });
        engine.orderbook.add_order(Order { id: 5, quantity: dec!(1), price: dec!(98), side: Side::Bid });
        engine.orderbook.add_order(Order { id: 6, quantity: dec!(1), price: dec!(97), side: Side::Bid });

        // sent an buy order with quantity 4 and price : 101 (3 order match from the 100 and 1 from the 101)
        engine.process_order(Order { id:7, quantity: dec!(3), price: dec!(102), side: Side::Bid });

        assert_eq!(engine.orderbook.best_ask(), Some(dec!(101)));

        let bids_at_101 =  engine.orderbook.get_level_mut(dec!(101), Side::Ask).unwrap();
        assert_eq!(bids_at_101[0].quantity, dec!(1));
        assert_eq!(bids_at_101[0].id, 2);
    }

    #[test]
    fn test_buy_order_multiple_level_full_fills() {
        let mut engine = MatchingEngine::new();

        // Add bids 
        engine.orderbook.add_order(Order { id: 1, quantity: dec!(3), price: dec!(100), side: Side::Ask });
        engine.orderbook.add_order(Order { id: 2, quantity: dec!(1), price: dec!(101), side: Side::Ask });
        engine.orderbook.add_order(Order { id: 3, quantity: dec!(2), price: dec!(102), side: Side::Ask });

        // Add bids
        engine.orderbook.add_order(Order { id: 4, quantity: dec!(1), price: dec!(99), side: Side::Bid });
        engine.orderbook.add_order(Order { id: 5, quantity: dec!(1), price: dec!(98), side: Side::Bid });
        engine.orderbook.add_order(Order { id: 6, quantity: dec!(1), price: dec!(97), side: Side::Bid });

        // sent an buy order with quantity 4 and price : 101 (3 order match from the 100 and 1 from the 101)
        engine.process_order(Order { id:7, quantity: dec!(4), price: dec!(101), side: Side::Bid });

        assert_eq!(engine.orderbook.best_ask(), Some(dec!(102)))
    }

    #[test]
    fn test_buy_order_multiple_level_partial_fills() {
        let mut engine = MatchingEngine::new();

        // Add bids 
        engine.orderbook.add_order(Order { id: 1, quantity: dec!(3), price: dec!(100), side: Side::Ask });
        engine.orderbook.add_order(Order { id: 2, quantity: dec!(1), price: dec!(101), side: Side::Ask });
        engine.orderbook.add_order(Order { id: 3, quantity: dec!(2), price: dec!(102), side: Side::Ask });

        // Add bids
        engine.orderbook.add_order(Order { id: 4, quantity: dec!(1), price: dec!(99), side: Side::Bid });
        engine.orderbook.add_order(Order { id: 5, quantity: dec!(1), price: dec!(98), side: Side::Bid });
        engine.orderbook.add_order(Order { id: 6, quantity: dec!(1), price: dec!(97), side: Side::Bid });

        // sent an buy order with quantity 4 and price : 101 (3 order match from the 100 and 1 from the 101)
        engine.process_order(Order { id:7, quantity: dec!(5), price: dec!(102), side: Side::Bid });

        assert_eq!(engine.orderbook.best_ask(), Some(dec!(102)));

        let bids_at_102 =  engine.orderbook.get_level_mut(dec!(102), Side::Ask).unwrap();
        assert_eq!(bids_at_102[0].quantity, dec!(1));
        assert_eq!(bids_at_102[0].id, 3);
    }



    #[test]
    fn test_whale_activity_of_eating_entire_asks () {
        let mut engine = MatchingEngine::new();

        // Add bids 
        engine.orderbook.add_order(Order { id: 1, quantity: dec!(4), price: dec!(100), side: Side::Ask });
        engine.orderbook.add_order(Order { id: 2, quantity: dec!(5), price: dec!(101), side: Side::Ask });
        
        engine.process_order(Order { id:3, quantity: dec!(10), price: dec!(110), side: Side::Bid });

        assert_eq!(engine.orderbook.best_bid(), Some(dec!(110)));
        let bids_at_110 =  engine.orderbook.get_level_mut(dec!(110), Side::Bid).unwrap();
        assert_eq!(bids_at_110[0].quantity, dec!(1));
        assert_eq!(bids_at_110[0].id, 3);
    }

    #[test]
    fn test_single_level_sell_order() {
        let mut engine = MatchingEngine::new();

        // Add bids 
        engine.orderbook.add_order(Order { id: 1, quantity: dec!(1), price: dec!(101), side: Side::Ask });
        engine.orderbook.add_order(Order { id: 2, quantity: dec!(1), price: dec!(102), side: Side::Ask });
        engine.orderbook.add_order(Order { id: 3, quantity: dec!(1), price: dec!(103), side: Side::Ask });

        // Add bids
        engine.orderbook.add_order(Order { id: 4, quantity: dec!(1), price: dec!(99), side: Side::Bid });
        engine.orderbook.add_order(Order { id: 5, quantity: dec!(1), price: dec!(98), side: Side::Bid });
        engine.orderbook.add_order(Order { id: 6, quantity: dec!(1), price: dec!(97), side: Side::Bid });

        // sent an sell order with quantity 1 and price : 99 (exact match for the best abidsk)
        engine.process_order(Order { id:7, quantity: dec!(1), price: dec!(99), side: Side::Ask });

        assert_eq!(engine.orderbook.best_bid(), Some(dec!(98)));
        let bids_at_98 = engine.orderbook.get_level_mut(dec!(98), Side::Bid).unwrap();
        assert_eq!(bids_at_98[0].id, 5);
    }

    #[test]
    fn test_single_level_partial_fill_sell_order() {
        let mut engine = MatchingEngine::new();

        // Add bids 
        engine.orderbook.add_order(Order { id: 1, quantity: dec!(1), price: dec!(101), side: Side::Ask });
        engine.orderbook.add_order(Order { id: 2, quantity: dec!(1), price: dec!(102), side: Side::Ask });
        engine.orderbook.add_order(Order { id: 3, quantity: dec!(1), price: dec!(103), side: Side::Ask });

        // Add bids
        engine.orderbook.add_order(Order { id: 4, quantity: dec!(2), price: dec!(99), side: Side::Bid });
        engine.orderbook.add_order(Order { id: 5, quantity: dec!(1), price: dec!(98), side: Side::Bid });
        engine.orderbook.add_order(Order { id: 6, quantity: dec!(1), price: dec!(97), side: Side::Bid });

        // sent an sell order with quantity 1 and price : 99 (exact match for the best bid)
        engine.process_order(Order { id:7, quantity: dec!(1), price: dec!(99), side: Side::Ask });

        assert_eq!(engine.orderbook.best_bid(), Some(dec!(99)));
        let bids_at_99 = engine.orderbook.get_level_mut(dec!(99), Side::Bid).unwrap();
        assert_eq!(bids_at_99[0].id, 4);
        assert_eq!(bids_at_99[0].quantity,dec!(1));
    }

    #[test]
    fn test_single_level_full_fill_low_price_sell_order() {
        let mut engine = MatchingEngine::new();

        // Add bids 
        engine.orderbook.add_order(Order { id: 1, quantity: dec!(1), price: dec!(101), side: Side::Ask });
        engine.orderbook.add_order(Order { id: 2, quantity: dec!(1), price: dec!(102), side: Side::Ask });
        engine.orderbook.add_order(Order { id: 3, quantity: dec!(1), price: dec!(103), side: Side::Ask });

        // Add bids
        engine.orderbook.add_order(Order { id: 4, quantity: dec!(6), price: dec!(99), side: Side::Bid });
        engine.orderbook.add_order(Order { id: 5, quantity: dec!(1), price: dec!(98), side: Side::Bid });
        engine.orderbook.add_order(Order { id: 6, quantity: dec!(1), price: dec!(97), side: Side::Bid });

        // sent an sell order with quantity 6 and price : 97 (exact match for the best bid)
        engine.process_order(Order { id:7, quantity: dec!(6), price: dec!(97), side: Side::Ask });

        assert_eq!(engine.orderbook.best_bid(), Some(dec!(98)));
        let bids_at_99 = engine.orderbook.get_level_mut(dec!(99), Side::Bid);
        assert!(bids_at_99.is_none(), "Price level $99 should have been deleted from the BTreeMap");
    }
    
    #[test]
    fn test_single_level_partial_fill_low_price_sell_order() {
        let mut engine = MatchingEngine::new();

        // Add bids 
        engine.orderbook.add_order(Order { id: 1, quantity: dec!(1), price: dec!(101), side: Side::Ask });
        engine.orderbook.add_order(Order { id: 2, quantity: dec!(1), price: dec!(102), side: Side::Ask });
        engine.orderbook.add_order(Order { id: 3, quantity: dec!(1), price: dec!(103), side: Side::Ask });

        // Add bids
        engine.orderbook.add_order(Order { id: 4, quantity: dec!(6), price: dec!(99), side: Side::Bid });
        engine.orderbook.add_order(Order { id: 5, quantity: dec!(1), price: dec!(98), side: Side::Bid });
        engine.orderbook.add_order(Order { id: 6, quantity: dec!(1), price: dec!(97), side: Side::Bid });

        // sent an sell order with quantity 4 and price : 96 (partial match for the best bid)
        engine.process_order(Order { id:7, quantity: dec!(4), price: dec!(96), side: Side::Ask });

        assert_eq!(engine.orderbook.best_bid(), Some(dec!(99)));
        let bids_at_99 = engine.orderbook.get_level_mut(dec!(99), Side::Bid).unwrap();
        assert_eq!(bids_at_99[0].quantity, dec!(2));
    }

    #[test]
    fn test_multi_level_partial_fill_low_price_sell_order() {
        let mut engine = MatchingEngine::new();

        // Add bids 
        engine.orderbook.add_order(Order { id: 1, quantity: dec!(1), price: dec!(101), side: Side::Ask });
        engine.orderbook.add_order(Order { id: 2, quantity: dec!(1), price: dec!(102), side: Side::Ask });
        engine.orderbook.add_order(Order { id: 3, quantity: dec!(1), price: dec!(103), side: Side::Ask });

        // Add bids
        engine.orderbook.add_order(Order { id: 4, quantity: dec!(4), price: dec!(99), side: Side::Bid });
        engine.orderbook.add_order(Order { id: 5, quantity: dec!(5), price: dec!(98), side: Side::Bid });
        engine.orderbook.add_order(Order { id: 6, quantity: dec!(1), price: dec!(97), side: Side::Bid });

        // sent an sell order with quantity 7 and price : 96 (partial match for the best bid)
        engine.process_order(Order { id:7, quantity: dec!(7), price: dec!(96), side: Side::Ask });

        assert_eq!(engine.orderbook.best_bid(), Some(dec!(98)));
        let bids_at_98 = engine.orderbook.get_level_mut(dec!(98), Side::Bid).unwrap();
        assert_eq!(bids_at_98[0].quantity, dec!(2));
    }

    #[test]
    fn test_multi_level_full_fill_low_price_sell_order() {
        let mut engine = MatchingEngine::new();

        // Add bids 
        engine.orderbook.add_order(Order { id: 1, quantity: dec!(1), price: dec!(101), side: Side::Ask });
        engine.orderbook.add_order(Order { id: 2, quantity: dec!(1), price: dec!(102), side: Side::Ask });
        engine.orderbook.add_order(Order { id: 3, quantity: dec!(1), price: dec!(103), side: Side::Ask });

        // Add bids
        engine.orderbook.add_order(Order { id: 4, quantity: dec!(4), price: dec!(99), side: Side::Bid });
        engine.orderbook.add_order(Order { id: 5, quantity: dec!(5), price: dec!(98), side: Side::Bid });
        engine.orderbook.add_order(Order { id: 6, quantity: dec!(1), price: dec!(97), side: Side::Bid });

        // sent an sell order with quantity 9 and price : 96 (partial match for the best bid)
        engine.process_order(Order { id:7, quantity: dec!(9), price: dec!(96), side: Side::Ask });

        assert_eq!(engine.orderbook.best_bid(), Some(dec!(97)));
        let bids_at_97 = engine.orderbook.get_level_mut(dec!(97), Side::Bid).unwrap();
        assert_eq!(bids_at_97[0].quantity, dec!(1));
    }
}