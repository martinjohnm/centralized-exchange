use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

use rust_decimal::Decimal;

use crate::model::{ClientOrderId, EngineOrderId, MatchingError, Order, OrderRequest, Side, Trade, UserId};




pub struct Orderbook {

    // Bids : Sorted descending (Highest price first)
    pub bids : BTreeMap<Decimal, VecDeque<OrderRequest>>,

    // Asks : Sorted ascending  (Lowest price first)
    pub asks : BTreeMap<Decimal, VecDeque<OrderRequest>>,

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

    fn cancel_order() {

    }
    
    // == client_id_map helpers ==========


    // == user_orders_helpers  ===========

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
    fn get_level_mut(&mut self, price: Decimal, side: Side) {

    }

    /// Phase 1: Validates the request and promotes it to a "Real" Order.
    /// This function does NOT change any state.
    fn validate_and_promote(&mut self, req: OrderRequest) -> std::result::Result<Order, MatchingError> {
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
}

#[cfg(test)]
mod tests {
    use super::*;
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
    // fn test_validation_errors() {
    //     let mut ob = Orderbook::new();

    //     // Test Missing Price
    //     let req_no_price = mock_request(101, None, Some(dec!(1.0)));
    //     let result = ob.validate_and_promote(req_no_price);

    //     // Explicitly use std::result::Result::Err to avoid "Archived" conflicts
    //     assert_eq!(result, std::result::Result::Err(MatchingError::MissingPrice));

    //     // Test Missing Quantity
    //     let req_no_qty = mock_request(102, Some(dec!(50000)), None);
    //     let result_qty = ob.validate_and_promote(req_no_qty);
    //     assert_eq!(result_qty, std::result::Result::Err(MatchingError::MissingQuantity));
    // }
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
}