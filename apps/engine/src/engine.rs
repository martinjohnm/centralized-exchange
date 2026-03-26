
use std::sync::Arc;

use crate::{ledger::Ledger, model::{ActionType, AssetRegistry, MarketId, OrderRequest}, orderbook::Orderbook};

pub struct Engine {
    market_id: MarketId,
    pub orderbook : Orderbook,
    pub ledger : Ledger,
    pub registry: Arc<AssetRegistry>,
}

impl Engine {
    pub fn new(market_id: MarketId, registry : Arc<AssetRegistry>) -> Self {
        Self { 
            market_id,
            orderbook : Orderbook::new(),
            ledger : Ledger::new(),
            registry
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
        // 1. Lock the users balance 
        // 2. Match fully or rest remaining
        let trades = self.orderbook.match_or_rest(req);
        match trades {
            Ok(trades) => {
                for trade in trades {
                    // unlock the makers (who sat in the orderbook ) fund

                }
            },
            Err(e) => {

            }
        }
        // 3. Unlock the takers funds 
        
        // 4. broadcast the events of the result

    }

    fn handle_cancel(&mut self, _order: OrderRequest) {
       
    }

    fn handle_deposit(&mut self, _order: OrderRequest) {
        
    }

    fn handle_cancel_all(&mut self, _order: OrderRequest) {
        
    }





}