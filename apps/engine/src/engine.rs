use std::collections::{HashMap, HashSet};

use crate::model::{ActionType, OrderRequest, exchange_proto::ExchangeRequest};

type UserId = u64;
type ClientOrderId = u64;
type EngineOrderId = u64;

pub struct Engine {
    symbol: String,

    // Key   : user_id, maping a (userid, cleintId) pair to get the order id in engine for fastlookup while 
    // Value : engine_id, canceling orders by market makers (Only for market making)
    pub client_id_map : HashMap<(UserId, ClientOrderId), EngineOrderId>,

    // Key   : user_id
    // Value : Set(engine_id), A HashSet of all active internal_order_ids for this user
    pub user_orders : HashMap<UserId, HashSet<EngineOrderId>>
}

impl Engine {
    pub fn new(symbol: String) -> Self {
        Self { 
            symbol,
            client_id_map : HashMap::new(),
            user_orders : HashMap::new()
        }
    }

    pub fn process_request(&mut self, request: OrderRequest) {
        // We match on the 'action' field of your clean OrderRequest struct
        match request.action {
            ActionType::Create => {
                self.handle_create(request);
            }
            ActionType::Cancel => {
                self.handle_cancel(request);
            }
            ActionType::Deposit => {
                self.handle_deposit(request);
            }
            ActionType::CancelAll => {
                self.handle_cancel_all(request);
            }
        }
    }

    fn handle_create(&mut self, order: OrderRequest) {
        // Logic for adding to BTreeMap Orderbook goes here
        // 1. add to the client_id_engine_id map
        // 2. add to the order_users map

    }

    fn handle_cancel(&mut self, order: OrderRequest) {
       
    }

    fn handle_deposit(&mut self, order: OrderRequest) {
        
    }

    fn handle_cancel_all(&mut self, order: OrderRequest) {
        
    }
}