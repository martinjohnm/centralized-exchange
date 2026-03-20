use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::trade::{model::{Action, Order, OrderRequest, Side, Trade}, orderbook::Orderbook};




pub struct MatchingEngine {
    pub orderbook : Orderbook,
    current_order_id : u64,
    pub asset_pair: String
}

impl MatchingEngine {

    pub fn new(asset_pair: String) -> Self {
        Self { 
            orderbook: Orderbook::new(), 
            current_order_id: 1, // Start at 1,
            asset_pair : asset_pair.to_uppercase()
        }
    }

    fn next_id(&mut self) -> u64 {
        let id = self.current_order_id;
        self.current_order_id += 1;
        id
    }

    pub fn submit_order(&mut self, req: OrderRequest) -> Vec<Trade> {
        // 1. Assign the order_id
        let order_id = self.next_id();

        // 2. Wrap the request into internal Order
        let taker_order = Order {
            id: order_id,
            user_id: req.user_id,
            price: req.price,
            quantity: req.quantity,
            side: req.side,
            action: req.action,
            order_type: req.order_type
        };

        self.process_order(taker_order)
    }

    /// Processes an incoming order against the existing order book.
    /// 
    /// #Data Flow: 
    /// 1. **Peek**: Check `best price` opposite side.
    /// 2. **Match**: If price cross, enter the matching loop.
    /// 3. **Fill**: Substract volume from Taker and Maker.
    /// 4. **Cleanup**: Remove price levels if they hit zero volume.
    /// 5. **Rest**: Add remaining Taker volume to the book as the Limit order.

    // ---------------- PRIVATE: THe "Hot path" matching logic ----------------
    fn process_order(&mut self, taker_order: Order) -> Vec<Trade> {
        let mut trades = Vec::new();

        match taker_order.action {
            Action::Create => {
                self.process_create_order(taker_order, &mut trades)
            },
            Action::Cancel => {
                self.process_cancel(taker_order.user_id, taker_order.id);
            },
            Action::CancelAll => {
                self.process_cancel_all(taker_order.user_id, taker_order.id);
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

    fn process_create_order(&mut self, mut taker_order : Order, trades: &mut Vec<Trade>) {
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


                                // ---- SELF TRADE PREVENTION (CANCEL MAKER) -------
                                if taker_order.is_self_trade(&maker_order) {
                                    // If taker_order and maker_order users are same it will 
                                    // create a fake trade with itself creating fake volume and 
                                    // unexpected fee paying here we are cancelling the maker_order

                                    // By NOT pushing maker_order back and NOT matching, 
                                    // the maker order is effectively cancelled/dropped.
                                    continue;
                                }
                                
                                // ---- PROCEED FROM HERE IF NOT A SELF TRADE------
                                // The math how much can we actually trade? 
                                let match_quantity = taker_order.quantity.min(maker_order.quantity);

                                taker_order.quantity -= match_quantity;
                                maker_order.quantity -= match_quantity;
                                
                                trades.push(Trade { 
                                    maker_id: maker_order.id, 
                                    taker_id: taker_order.id, 
                                    price, 
                                    quantity: match_quantity, 
                                    taker_side: taker_order.side, 
                                    maker_side: maker_order.side 
                                });
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
    }

    fn process_cancel(&mut self, user_id: u64, order_id: u64) {

    }

    fn process_cancel_all(&mut self, user_id: u64, order_id: u64) {

    }

}

#[cfg(test)]
mod engine_tests;