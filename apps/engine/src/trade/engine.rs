use std::collections::{HashMap, HashSet};

use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::trade::{model::{Action, EngineError, EngineRequest, Order, OrderCancel, OrderCancelAll, OrderRequest, Side, Trade}, orderbook::Orderbook, utils::debug_print_book};


type UserId = u64;
type ClientOrderId = u64;
type EngineOrderId = u64;

pub struct MatchingEngine {
    pub orderbook : Orderbook,
    // Incrementing orderId for unique identification
    current_order_id : u64,
    pub asset_pair: String,
}

impl MatchingEngine {

    pub fn new(asset_pair: String) -> Self {
        Self { 
            orderbook: Orderbook::new(), 
            current_order_id: 1, // Start at 1,
            asset_pair : asset_pair.to_uppercase(),
            
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

    // ---------------- PRIVATE: THe "Hot path" matching logic ----------------
    pub fn process_order(&mut self, req: OrderRequest) {
        
        let order = self.create_engine_order_id_if_it_is_create_else_leave(req);
        match order {

            Ok(order_type) => {
                match order_type {
                    EngineRequest::Create(create_order) => {
                        let mut trades = Vec::new();
                        
                        self.process_create_order(create_order, &mut trades)
                    
                    },
                    EngineRequest::Cancel(cancel_order) => {

                        self.process_cancel(cancel_order);
                    
                    },
                    EngineRequest::CancelAll(cancel_all_orders) => {
                        // 1. Atomically take the entire set of orders for this user
                        // if let Some(order_ids) = self.user_orders.remove(&cancel_all_orders.user_id) {
                        //     // 2. Loop through every active engine_id they have
                        //     for eid in order_ids {
                        //         self.process_cancel_by_id(eid);
                        //     }
                        // }
                    }
                }
            },
            Err(error) => {

            }

        }        
        
    }

    fn is_match(&self, taker_price: Decimal, best_price: Decimal, side: Side) -> bool {
        match side {
            Side::Ask => taker_price <= best_price,
            Side::Bid => taker_price >= best_price
        }
    }

    pub fn process_create_order(&mut self, mut taker_order : Order, trades: &mut Vec<Trade>) {
        // loop on the quantity while it is non zero
        // If a whale created the trade map his client_id -> engine_id map (if !client_id = 0)
        if taker_order.client_id != 0 {
        //     self.client_id_map.insert(
        //         (taker_order.user_id, taker_order.client_id), 
        //         taker_order.engine_id
        // );
        }
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

                                // if the order fully filled we should remove it from both the clien_id_map and 
                                // from the user_orders map 
                                if maker_order.quantity == dec!(0) {
                                    // 1. client_id_map removal
                                    // 2. user_orders_map removal

                                } // if the maker partially filled we should put the order where it was
                                else  {
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
        // THIS IS A TAKET -> MAKER TRANSITION
        if taker_order.quantity > dec!(0) {
            // 1. add it to taker -> client_id_map
            // if taker_order.client_id != 0 {
            //     self.client_id_map.insert(
            //         (taker_order.user_id, taker_order.client_id), 
            //         taker_order.engine_id);
            // }
            // 2. User oders map 
            // Taker is partially filled. Adding remaining to the orderbook.
            self.orderbook.add_order(taker_order);
        }
        
    }

    pub fn process_cancel(&mut self, cancel_order : OrderCancel) {
        
        self.orderbook.cancel_order(cancel_order.engine_id);
        // 2. settle the bank account amounts 
    }

    pub fn process_cancel_by_id(&mut self, order_id: u64) {
        self.orderbook.cancel_order(order_id);
    }


    pub fn create_engine_order_id_if_it_is_create_else_leave(&mut self, req: OrderRequest) -> Result<EngineRequest, EngineError> {
        match req.action {
            Action::Create => {
                // if it is create assign a new order id 
                let order_id = self.next_id();
                // 2. Wrap the request into internal Order
                let taker_order = Order {
                    id: order_id,
                    user_id: req.user_id,
                    price: req.price,
                    quantity: req.quantity,
                    side: req.side,
                    action: req.action,
                    order_type: req.order_type,
                    client_id: req.client_id,
                    engine_id: order_id
                };
                Ok(EngineRequest::Create(taker_order))
            },
            Action::Cancel => {
                let cancel_order = OrderCancel {
                    user_id : req.user_id,
                    client_id: req.client_id,
                    engine_id: req.engine_id
                };
                Ok(EngineRequest::Cancel(cancel_order))
            },
            Action::CancelAll => {
                let cancel_all_order = OrderCancelAll {
                    user_id : req.user_id
                };
                Ok(EngineRequest::CancelAll(cancel_all_order))
            },
            
        }
    }



}

#[cfg(test)]
mod engine_tests;