use std::collections::{HashMap, HashSet};


type UserId = u64;
type ClientOrderId = u64;
type EngineOrderId = u64;


pub struct Orderbook {

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
            client_id_map: HashMap::new(), 
            user_orders: HashMap::new()
        }
    }

    pub fn create_order() {

    }

    pub fn cancel_order() {

    }
    
}