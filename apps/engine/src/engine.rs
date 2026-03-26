
use crate::{ledger::Ledger, model::{ActionType, OrderRequest}, orderbook::Orderbook};

pub struct Engine {
    symbol: String,
    pub orderbook : Orderbook,
    pub ledger : Ledger,
}

impl Engine {
    pub fn new(symbol: String) -> Self {
        Self { 
            symbol,
            orderbook : Orderbook::new(),
            ledger : Ledger::new()
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
    fn handle_create(&mut self, req: OrderRequest) {
        // Logic for adding to BTreeMap Orderbook goes here
        // 1. add to the client_id_engine_id map
        // 2. add to the order_users map
        
        let t = self.orderbook.match_or_rest(req);
        match t {
            Ok(t) => {
                println!("{:?}", t);
            },
            Err(e) => {

            }
        }
    }

    fn handle_cancel(&mut self, _order: OrderRequest) {
       
    }

    fn handle_deposit(&mut self, _order: OrderRequest) {
        
    }

    fn handle_cancel_all(&mut self, _order: OrderRequest) {
        
    }





}