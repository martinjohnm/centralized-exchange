

use rust_decimal::Decimal;
use tokio::sync::mpsc::{self, Sender};

use crate::{ledger::Ledger, model::{ActionType, DepthResponse, InternalOrderStatus, InternalTrade, OrderRequest, exchange_proto::{ExecutionReport, MarketId, Trade}}, orderbook::{self, Orderbook}, utils::MarketConfig};

pub struct Engine {
    pub config: MarketConfig,
    pub orderbook : Orderbook,
    pub ledger : Ledger,
    pub trade_transmitter: Sender<InternalTrade>,
    pub report_transmitter: Sender<ExecutionReport>
}

impl Engine {
    pub fn new(config: MarketConfig, trade_producer : Sender<InternalTrade>, report_producer: Sender<ExecutionReport>) -> Self {
        Self { 
            config,
            orderbook : Orderbook::new(config,trade_producer.clone()),
            ledger : Ledger::new(),
            trade_transmitter : trade_producer,
            report_transmitter : report_producer
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

    pub fn get_market_depth(&self, levels: usize) -> DepthResponse {
        self.orderbook.get_depth(levels)
    }

    // -----------Request handlers --------------------------
    fn handle_create(&mut self, req: OrderRequest) {

        let client_id = req.client_id.unwrap_or(0);
        let original_quantity = req.quantity.unwrap_or(rust_decimal_macros::dec!(0));


        // 1. Lock the users balance 
        // 2. Match fully or rest remaining
        // we clone the req becasue match_or_rest takes full ownership to potentially rest it 

        
        match self.orderbook.match_or_rest(req.clone()) {
            Ok(trades) => {

                // 3 - post match logic--

                // calculate how much filled to find the remainder
                let total_filled : Decimal = trades.iter().map(|t| t.quantity).sum();
                let remaining = original_quantity - total_filled;

                let mut  proto_trade : Vec<Trade> = Vec::new();
                for internal_trade in trades {
                    // unlock the makers (who sat in the orderbook ) fund
                    if let Err(e) = self.trade_transmitter.try_send(internal_trade.clone()) {
                        eprintln!("Global telemetry lag:{}" , e);
                    }

                    proto_trade.push(Trade { 
                        maker_id : internal_trade.maker_id,
                        taker_id : internal_trade.taker_id,
                        price : internal_trade.price.to_string(),
                        quantity : internal_trade.quantity.to_string(),
                        taker_side : internal_trade.taker_side as i32,
                        maker_side : internal_trade.maker_side as i32,
                        timestamp : internal_trade.timestamp,
                        market : internal_trade.market as i32,
                        base : internal_trade.base as i32,
                        quote : internal_trade.quote as i32
                     });
                }

                // 4 ---- The reporing to clients logic
                let report = self.create_report(&req, proto_trade, remaining);
                if let Err(e) = self.report_transmitter.try_send(report) {
                        eprintln!("Global telemetry lag:{}" , e);
                    }
                // send to the redis reponder (Targeted pub sub)
                // This resolves the async fn in the trade placer

            },
            Err(e) => {
                // 5 -- handler rejection here 

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



    // helpers
    pub fn send_succes() {

    }

    pub fn send_rejection() {

    }

    pub fn create_report(
        &self, 
        req: &OrderRequest, 
        trades: Vec<Trade>, 
        remaining: Decimal
    ) -> ExecutionReport {
        // 1. Determine Status
        let status = if req.action == ActionType::Cancel {
            InternalOrderStatus::Cancelled
        } else if trades.is_empty() {
            InternalOrderStatus::Placed
        } else if remaining.is_zero() {
            InternalOrderStatus::Filled
        } else {
            InternalOrderStatus::PartiallyFilled
        };

        // 2. Map to Protobuf Struct
        ExecutionReport {
            client_id: req.client_id.unwrap_or(0),
            user_id: req.user_id,
            status: status as i32, // Direct cast to i32 for Protobuf
            trades,
            remaining_quantity: remaining.to_string(),
            error_message: String::new(),
        }
}



}