use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

use rust_decimal::Decimal;

use crate::model::{ClientOrderId, EngineOrderId, OrderRequest, UserId};




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
    pub user_orders : HashMap<UserId, HashSet<EngineOrderId>>

}

impl Orderbook {
    pub fn new() -> Self {
        Self {
            bids : BTreeMap::new(),
            asks : BTreeMap::new(),
            client_id_map: HashMap::new(), 
            user_orders: HashMap::new()
        }
    }

    pub fn create_order() {

    }

    pub fn cancel_order() {

    }
    
}