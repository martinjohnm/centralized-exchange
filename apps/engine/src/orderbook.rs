use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

use rust_decimal::Decimal;

use crate::model::{ClientOrderId, EngineOrderId, MatchingError, Order, OrderError, OrderRequest, Side, Trade, UserId};




pub struct Orderbook {

    // Bids : Sorted descending (Highest price first)
    pub bids : BTreeMap<Decimal, VecDeque<Order>>,

    // Asks : Sorted ascending  (Lowest price first)
    pub asks : BTreeMap<Decimal, VecDeque<Order>>,

    // Key   : user_id, maping a (userid, cleintId) pair to get the order id in engine for fastlookup while 
    // Value : engine_id, canceling orders by market makers (Only for market making)
    pub client_id_map : HashMap<(UserId, ClientOrderId), EngineOrderId>,

    // Key   : user_id
    // Value : Set(engine_id), A HashSet of all active internal_order_ids for this user
    pub user_orders : HashMap<UserId, HashSet<EngineOrderId>>,

    last_engine_id: u64,

    // Key: EngineOrderId -> Value: (Price, Side)
    pub orders_metadata: HashMap<EngineOrderId, (Decimal, Side)>,
}

impl Orderbook {
    pub fn new() -> Self {
        Self {
            bids : BTreeMap::new(),
            asks : BTreeMap::new(),
            client_id_map: HashMap::new(), 
            user_orders: HashMap::new(),
            last_engine_id : 1,
            orders_metadata : HashMap::new(),
        }
    }

    fn next_engine_id(&mut self) -> u64 {
        self.last_engine_id += 1;
        self.last_engine_id
    }

    // ========== THE HOT PATH ===========
    pub fn match_or_rest(&mut self, req: OrderRequest) -> Result<Vec<Trade>, MatchingError> {


        // --- VALIDATION ---
        // If validation fails, it exits here with the Err.
        let mut taker_order = self.validate_and_promote(req)?;

        self.add_to_indexes(&taker_order);

        
        // matching logic later

        let trades: Vec<Trade> = Vec::new();
        Ok(trades)
    }

    // ========= CANCELLATION LOGIC =============

    // =========== OUR MAIN AIM HERE IS TO GET THE engine_id asap =========================

    // cancel using the exact engine_id (the detail get from the user_orders) map
    // Normal cancellation using the engine_id path (used by retailers)
    pub fn cancel_by_id(&mut self, engine_id: EngineOrderId) -> Result<Order, OrderError> {
        // 1. cancel and get it from the fn return value
        self.execute_cancel(engine_id)
    }
    
    // fast lane we can cancel an order using the client_id and the user_id composite key 
    // (user_id, client_id) => engine_id (we get it constant time O(1))
    pub fn cancel_by_client_id(&mut self, user_id: UserId, client_id: ClientOrderId) -> Result<Order, OrderError> {
        // 1. get the engine_id from the client_id_map (use the client_id and user_id composite key)
        let engine_id = self.client_id_map
            .get(&(user_id, client_id))
            .copied()
            .ok_or(OrderError::OrderNotFound)?;
        // 2. get it from the cancel fn return value and return
        self.execute_cancel(engine_id)
    }
    

    // we get all the engine_id for a particular user from the HASHSET 
    // user_id => Hashset(engine_id)
    // for bulk cancellation (Cancelling all orders for a particular user)
    pub fn cancel_all_for_user(&mut self, user_id: UserId) -> Vec<Order> {
        let mut cancelled = Vec::new();

        // We must collect IDs first to avoid borrowing 'self' while mutating in the loop
        if let Some(ids) = self.user_orders.get(&user_id) {
            let ids_to_remove : Vec<EngineOrderId> = ids.iter().copied().collect();
            for id in ids_to_remove {
                if let Ok(order) = self.execute_cancel(id) {
                    cancelled.push(order);
                }
            }
        }
        cancelled
    }
   
    // -============================================ HELPERS ==============================================================
    // 1. Fn to get the best bid
    fn best_bid(&self) -> Option<Decimal> {
        self.bids.keys().next_back().cloned()
    }
    // 2. Fn to get the best ask
    fn best_ask(&self) -> Option<Decimal> {
        self.asks.keys().next().cloned()
    }

    // Get all the vecdequeu array of orders of a price level
    // Which is Option<&mut Vecdeque(Order)> the ref & 
    fn get_level_mut(&mut self, price: Decimal, side: Side) -> Option<&mut VecDeque<Order>> {
        match side {
            Side::Buy => self.bids.get_mut(&price),
            Side::Sell => self.asks.get_mut(&price)
        }
    }

    // is match possible checker ??===================
    fn is_match(&self, taker_price: Decimal, best_price: Decimal, side: Side) -> bool {
        match side {
            Side::Sell  => taker_price <= best_price,
            Side::Buy   => taker_price >= best_price
        }
    }

    /// Phase 1: Validates the request and promotes it to a "Real" Order.
    /// This function does NOT change any state.
    fn validate_and_promote(&mut self, req: OrderRequest) -> Result<Order, MatchingError> {
        let price = req.price.ok_or(MatchingError::MissingPrice)?;
        let quantity = req.quantity.ok_or(MatchingError::MissingQuantity)?;
        let client_id = req.client_id.ok_or(MatchingError::MissingClientId)?;

        Ok(Order {
            engine_id: self.next_engine_id(), // We assign the ID here
            client_id,
            user_id: req.user_id,
            price,
            quantity,
            side: req.side,
            timestamp: req.timestamp,
        })
    }

    /// Phase 2: Updates all secondary HashMaps.
    /// Call this only once you are sure the order is staying in the system.
    fn add_to_indexes(&mut self, order: &Order) {
        // 1. Client ID -> Engine ID
        self.client_id_map.insert((order.user_id, order.client_id), order.engine_id);
        
        // 2. Engine ID -> (Price, Side)
        self.orders_metadata.insert(order.engine_id, (order.price, order.side));

        // 3. User -> Set of Engine IDs
        self.user_orders
            .entry(order.user_id)
            .or_default()
            .insert(order.engine_id);
    }

    /// Phase 3: Removes all traces of an order from secondary indexes.
    /// Call this when an order is Fully Filled or Cancelled.
    fn remove_indexes(&mut self, engine_id: u64, user_id: u64, client_id: u64) {
        // 1. Remove from Client ID Map
        self.client_id_map.remove(&(user_id, client_id));

        // 2. Remove from Metadata Map (The Price/Side GPS)
        self.orders_metadata.remove(&engine_id);

        // 3. Remove from User's active order set
        if let Some(user_set) = self.user_orders.get_mut(&user_id) {
            user_set.remove(&engine_id);
            
            // Cleanup: If the user has no more active orders, 
            // remove the HashSet entirely to save memory.
            if user_set.is_empty() {
                self.user_orders.remove(&user_id);
            }
        }
    }

    /// Phase 4. Inserts the order into the actual BTreemap
    /// This should be called After match_or_rest has finished its matching
    fn rest_in_book(&mut self, order: Order) {
        let price = order.price;
        let side = order.side;

        // 1. Get the side-specific map
        let target_map = match side {
            Side::Buy => &mut self.bids,
            Side::Sell => &mut  self.asks
        };

        // 2. Find the price level or create a new one 
        // entry() is highly efficient for BTreemap
        target_map
            .entry(price)
            .or_insert_with( || VecDeque::new()) // This or_insert_with is LAZY (only calls if it is empty) but or_insert viceversa
            .push_back(order);
    }

    // Cancel helper fn with the engine_id which can be called either by 
    // retil orders cancellation , market makers cancellation (with client_id) or 
    // retailers bulk cancellation with user_id

    fn execute_cancel(&mut self, engine_id: EngineOrderId) -> Result<Order, OrderError> {
        // 1. get the metadata (from orders_metadata)
        let (price, side) = self.orders_metadata.get(&engine_id)
            .copied()
            .ok_or(OrderError::OrderNotFound)?;

        // 2. get the Price level VecDeque
        let level = self.get_level_mut(price, side)
            .ok_or(OrderError::OrderNotFound)?;

        // 3. Remove from the VecDeque (Time Priority remains)
        let pos = level.iter().position(|o| o.engine_id == engine_id)
            .ok_or(OrderError::OrderNotFound)?;
        let removed_order = level.remove(pos)
            .ok_or(OrderError::OrderNotFound)?;
        // 4. Cleanup BTreemap if level is empty
        if level.is_empty() {
            match side {
                Side::Buy => self.bids.remove(&price),
                Side::Sell => self.asks.remove(&price)
            };
        };

        // 5. Remove indexes from all hashmaps
        self.remove_indexes(removed_order.engine_id, removed_order.user_id, removed_order.client_id);

        Ok(removed_order)
    } 
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::prelude::FromPrimitive;
    use rust_decimal_macros::dec;
    use crate::model::{Side, OrderType, ActionType};

    fn mock_request(client_id: u64, price: Option<Decimal>, qty: Option<Decimal>) -> OrderRequest {
        OrderRequest {
            user_id: 1,
            symbol: "BTC/USDT".to_string(),
            side: Side::Buy,
            price,
            quantity: qty,
            order_type: OrderType::Limit,
            action: ActionType::Create,
            client_id: Some(client_id),
            engine_id: None,
            timestamp: 123456789,
        }
    }

    #[test]
    fn test_validation_errors() {
        let mut ob = Orderbook::new();

        // 1. Test Missing Price
        let req = mock_request(101, None, Some(dec!(1.0)));
        let result = ob.validate_and_promote(req);
        
        // Check for specific error pattern
        assert!(matches!(result, Err(MatchingError::MissingPrice)));

        // 2. Test Missing Quantity
        let req_qty = mock_request(102, Some(dec!(50000)), None);
        let result_qty = ob.validate_and_promote(req_qty);
        
        assert!(matches!(result_qty, Err(MatchingError::MissingQuantity)));
    }
    #[test]
    fn test_promotion_and_id_generation() {
        let mut ob = Orderbook::new();
        let req = mock_request(101, Some(dec!(50000)), Some(dec!(1.5)));

        // First promotion
        let order1 = ob.validate_and_promote(req.clone()).unwrap();
        assert_eq!(order1.engine_id, 2);
        assert_eq!(order1.price, dec!(50000));

        // Second promotion should increment ID
        let order2 = ob.validate_and_promote(req).unwrap();
        assert_eq!(order2.engine_id, 3);
    }

    #[test]
    fn test_indexing_logic() {
        let mut ob = Orderbook::new();
        let user_id = 1;
        let client_id = 101;
        
        let order = Order {
            engine_id: 55,
            client_id,
            user_id,
            price: dec!(45000),
            quantity: dec!(0.5),
            side: Side::Buy,
            timestamp: 999,
        };

        // Execute indexing
        ob.add_to_indexes(&order);

        // 1. Verify Client ID Map
        assert_eq!(ob.client_id_map.get(&(user_id, client_id)), Some(&55));

        // 2. Verify Metadata Map
        let (p, s) = ob.orders_metadata.get(&55).unwrap();
        assert_eq!(*p, dec!(45000));
        assert_eq!(*s, Side::Buy);

        // 3. Verify User Orders Set
        let user_set = ob.user_orders.get(&user_id).unwrap();
        assert!(user_set.contains(&55));
    }
    #[test]
    fn test_remove_indexes() {
        let mut ob = Orderbook::new();
        let (u_id, c_id, e_id) = (1, 101, 55);
        
        // Setup: Add an order first
        let order = Order {
            engine_id: e_id,
            client_id: c_id,
            user_id: u_id,
            price: dec!(100),
            quantity: dec!(1),
            side: Side::Buy,
            timestamp: 0,
        };
        ob.add_to_indexes(&order);

        // Action: Remove it
        ob.remove_indexes(e_id, u_id, c_id);

        // Assert: Maps should be empty
        assert!(ob.client_id_map.is_empty());
        assert!(ob.orders_metadata.is_empty());
        assert!(ob.user_orders.is_empty());
    }

    #[test]
    fn test_get_level_mut_and_modify() {
        let mut ob = Orderbook::new();
        let price = dec!(50000);
        let side = Side::Buy;

        // 1. Setup: Manually insert an order into the bids
        let order = Order {
            engine_id: 1,
            price,
            quantity: dec!(1.0),
            side,
            user_id: 42,
            client_id: 101,
            timestamp: 0,
        };
        
        let mut queue = VecDeque::new();
        queue.push_back(order);
        ob.bids.insert(price, queue);

        // 2. Action: Get the level mutably
        {
            let level = ob.get_level_mut(price, side).expect("Level should exist");
            assert_eq!(level.len(), 1);
            
            // Modify it: Pop the order
            level.pop_front();
        }

        // 3. Assert: Verify the change persisted in the Orderbook
        let level_after = ob.bids.get(&price).unwrap();
        assert!(level_after.is_empty(), "Order should have been removed from the book");
    }

    #[test]
    fn test_get_level_mut_missing() {
        let mut ob = Orderbook::new();
        
        // Try to get a price that hasn't been added
        let result = ob.get_level_mut(dec!(99999), Side::Sell);
        
        assert!(result.is_none(), "Should return None for non-existent price level");
    }

    #[test]
    fn test_get_level_mut_wrong_side() {
        let mut ob = Orderbook::new();
        let price = dec!(100);
        
        // Add a BID (Buy)
        ob.bids.insert(price, VecDeque::new());

        // Try to get a SELL level at that same price
        let result = ob.get_level_mut(price, Side::Sell);
        
        assert!(result.is_none(), "Should not find a Sell level when only a Buy level exists");
    }


    // The Cancel logic unit tests 
    // Helper to bootstrap an orderbook with a resting order
    fn setup_with_order(engine_id: u64, user_id: u64, client_id: u64) -> Orderbook {
        let mut ob = Orderbook::new();
        let order = Order {
            engine_id,
            user_id,
            client_id,
            price: dec!(50000),
            quantity: dec!(1.0),
            side: Side::Buy,
            timestamp: 123456789,
        };
        // Setup state manually to isolate cancellation logic
        ob.add_to_indexes(&order);
        ob.rest_in_book(order);
        ob
    }

    #[test]
    fn test_cancel_by_id_retail_path() {
        let (e_id, u_id, c_id) = (1001, 1, 55);
        let mut ob = setup_with_order(e_id, u_id, c_id);

        // Act: Direct internal ID cancel
        let result = ob.cancel_by_id(e_id);

        // Assert: Order is returned and maps are clean
        assert!(result.is_ok());
        let cancelled = result.unwrap();
        assert_eq!(cancelled.engine_id, e_id);
        
        // Final state check
        assert!(ob.orders_metadata.get(&e_id).is_none());
        assert!(ob.client_id_map.get(&(u_id, c_id)).is_none());
        assert!(ob.bids.is_empty());
    }

    #[test]
    fn test_cancel_by_client_id_mm_path() {
        let (e_id, u_id, c_id) = (2002, 7, 999);
        let mut ob = setup_with_order(e_id, u_id, c_id);

        // Act: Market Maker style cancel (User + Client ID)
        let result = ob.cancel_by_client_id(u_id, c_id);

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap().engine_id, e_id);
        
        // Ensure index resolution is broken
        assert!(!ob.client_id_map.contains_key(&(u_id, c_id)));
    }

    #[test]
    fn test_bulk_cancel_all_for_user() {
        let mut ob = Orderbook::new();
        let user_id = 42;
        let common_price = dec!(50000); // All orders at the same price
        // 1. Setup: Give user 42 three different orders at different prices
        for i in 1..=3 {
            let order = Order {
                engine_id: i as u64,
                user_id,
                client_id: 100 + i as u64,
                price: common_price,
                quantity: dec!(0.5),
                side: Side::Buy,
                timestamp: 0,
            };
            ob.add_to_indexes(&order);
            ob.rest_in_book(order);
        }

        // 2. Act: Nuclear option (Cancel All)
        let cancelled_orders = ob.cancel_all_for_user(user_id);

        // 3. Assert
        assert_eq!(cancelled_orders.len(), 3);
        
        // Verify user metadata is nuked
        assert!(ob.user_orders.get(&user_id).is_none());
        // Verify book is empty
        assert!(ob.bids.is_empty());
        // Verify cross-reference maps are empty
        assert!(ob.client_id_map.is_empty());
        assert!(ob.orders_metadata.is_empty());
    }

    #[test]
    fn test_cancel_order_not_found_scenarios() {
        let mut ob = Orderbook::new();
        
        // Try to cancel non-existent engine_id
        assert!(matches!(ob.cancel_by_id(999), Err(OrderError::OrderNotFound)));
        
        // Try to cancel non-existent client_id combo
        assert!(matches!(ob.cancel_by_client_id(1, 123), Err(OrderError::OrderNotFound)));
        assert!(ob.cancel_all_for_user(1).is_empty());
    }

    fn test_is_match () {

        let ob = Orderbook::new();

        let mut best_price = Decimal::from_i32(120).unwrap();
        let mut taker_price = Decimal::from_i32(100).unwrap();

        assert_eq!(ob.is_match(taker_price, best_price, Side::Buy), false);
        taker_price = Decimal::from_i32(90).unwrap();
        best_price = Decimal::from_i32(89).unwrap();
        assert_eq!(ob.is_match(taker_price, best_price, Side::Sell), false);

        taker_price = Decimal::from_i32(100).unwrap();
        best_price = Decimal::from_i32(101).unwrap();
        assert_eq!(ob.is_match(taker_price, best_price, Side::Sell), true);
        taker_price = Decimal::from_i32(100).unwrap();
        best_price = Decimal::from_i32(91).unwrap();
        assert_eq!(ob.is_match(taker_price, best_price, Side::Buy), true);
    }
}