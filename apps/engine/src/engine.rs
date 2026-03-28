

use tokio::sync::mpsc::{self, Sender};

use crate::{ledger::Ledger, model::{ActionType, OrderRequest, InternalTrade}, orderbook::Orderbook};

pub struct Engine {
    symbol: String,
    pub orderbook : Orderbook,
    pub ledger : Ledger,
    pub transmitter: Sender<InternalTrade>
}

impl Engine {
    pub fn new(symbol: String, trade_producer : Sender<InternalTrade>) -> Self {
        Self { 
            symbol,
            orderbook : Orderbook::new(trade_producer.clone()),
            ledger : Ledger::new(),
            transmitter : trade_producer
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
                    self.transmitter.send(trade);
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