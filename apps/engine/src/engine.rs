
use crate::{model::{ActionType, OrderRequest}, orderbook::Orderbook};

pub struct Engine {
    symbol: String,
    // engine_id (strictly increasing)
    current_engine_id : u64, // It has a helper to create next engine_id

    pub orderbook : Orderbook
}

impl Engine {
    pub fn new(symbol: String) -> Self {
        Self { 
            symbol,
            current_engine_id : 1, // start at zero
            orderbook : Orderbook::new()
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

    // -----------Request handlers --------------------------
    fn handle_create(&mut self, _order: OrderRequest) {
        // Logic for adding to BTreeMap Orderbook goes here
        // 1. add to the client_id_engine_id map
        // 2. add to the order_users map
        
    }

    fn handle_cancel(&mut self, _order: OrderRequest) {
       
    }

    fn handle_deposit(&mut self, _order: OrderRequest) {
        
    }

    fn handle_cancel_all(&mut self, _order: OrderRequest) {
        
    }


    // ------------Helpers--------------------------------
    // 1.0 ====== Helper to create next_engine_id========
    fn next_engine_id(&mut self) -> u64 {
        let next_engine_id = self.current_engine_id;
        self.current_engine_id += 1;
        next_engine_id
    }



}