

use std::time::{SystemTime, UNIX_EPOCH};

use rust_decimal::Decimal;
use tokio::sync::mpsc::{self, Sender};

use crate::{ledger::Ledger, model::{ActionType, DepthResponse, InternalOrderStatus, InternalTrade, OrderRequest, exchange_proto::{ExecutionReport, MarketId, OrderUpdate, Trade}}, orderbook::{self, Orderbook}, utils::MarketConfig};

pub struct Engine {
    pub config: MarketConfig,
    pub orderbook : Orderbook,
    pub ledger : Ledger,
    pub trade_transmitter: Sender<InternalTrade>,
    pub report_transmitter: Sender<ExecutionReport>,
    pub order_transmitter: Sender<OrderUpdate>
}

impl Engine {
    pub fn new(config: MarketConfig, trade_producer : Sender<InternalTrade>, report_producer: Sender<ExecutionReport>, orders_producer: Sender<OrderUpdate>) -> Self {
        Self { 
            config,
            orderbook : Orderbook::new(config,trade_producer.clone()),
            ledger : Ledger::new(),
            trade_transmitter : trade_producer,
            report_transmitter : report_producer,
            order_transmitter : orders_producer
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


        let mut taker_proto_trades: Vec<Trade> = Vec::new();

        // 1. Lock the users balance 
        // 2. Match fully or rest remaining
        // we clone the req becasue match_or_rest takes full ownership to potentially rest it 

        
        match self.orderbook.match_or_rest(req.clone()) {
            Ok(res) => {

                // 3 - post match logic--

                // calculate how much filled to find the remainder
                let total_filled : Decimal = res.trades.iter().map(|t| t.quantity).sum();
                let taker_remaining = original_quantity - total_filled;

                
                for internal_trade in res.trades {
                    // unlock the makers (who sat in the orderbook ) fund
                    if let Err(e) = self.trade_transmitter.try_send(internal_trade.clone()) {
                        eprintln!("Global telemetry lag:{}" , e);
                    }

                    let proto_t = Trade {
                        maker_id : internal_trade.maker_user_id,
                        taker_id : internal_trade.taker_user_id,

                        maker_order_id : internal_trade.maker_order_id, 
                        taker_order_id : internal_trade.taker_order_id,

                        price : internal_trade.price.to_string(),
                        quantity : internal_trade.quantity.to_string(),
                        taker_side : internal_trade.taker_side as i32,
                        maker_side : internal_trade.maker_side as i32,
                        timestamp : internal_trade.timestamp,
                        market : internal_trade.market as i32,
                        base : internal_trade.base as i32,
                        quote : internal_trade.quote as i32
                    };

                    taker_proto_trades.push(proto_t.clone());

                    // 3. create a unique rport for the maker
                    // maker needs to know their order was hit
                    let maker_report = ExecutionReport {
                        client_id: req.client_id.unwrap_or(0),
                        user_id: internal_trade.maker_user_id,
                        status: (if internal_trade.maker_remaining.is_zero() {
                            InternalOrderStatus::Filled 
                        } else {
                            InternalOrderStatus::PartiallyFilled
                        }) as i32, // Direct cast to i32 for Protobuf
                        trades : vec![proto_t],
                        remaining_quantity: internal_trade.maker_remaining.to_string(), // this is only used here
                        error_message: String::new(),
                    };

                    let maker_order = OrderUpdate {
                        engine_id : internal_trade.maker_order_id,
                        user_id : internal_trade.maker_user_id,
                        market : internal_trade.market as i32,
                        price : internal_trade.price.to_string(),
                        quantity : internal_trade.maker_initial_quantity.to_string(), // set the quantity as alwaays the initial quantity 
                        filled_quantity : (internal_trade.maker_initial_quantity - internal_trade.maker_remaining).to_string(), // this is the updating quantity
                        side : internal_trade.maker_side as i32,
                        status : (if internal_trade.maker_remaining.is_zero() {
                            InternalOrderStatus::Filled 
                        } else {
                            InternalOrderStatus::PartiallyFilled
                        }) as i32, // Direct cast to i32 for Protobuf,
                        timestamp : internal_trade.timestamp
                    };

                    match self.report_transmitter.try_send(maker_report) {
                        Ok(_) => {}
                        Err(e) => {eprintln!("error senting maker report : {}",e )}
                    }
                    match self.order_transmitter.try_send(maker_order) {
                        Ok(_) => {},
                        Err(e) => {eprintln!("error senting maker order: {}", e)}
                    }
                }

                // 4 ---- The reporing to clients logic
                
                let taker_report = self.create_report(&req, taker_proto_trades, taker_remaining);
                match self.report_transmitter.try_send(taker_report) {
                    Ok(_) => {}
                    Err(e) => {eprintln!("error senting taker report : {}",e )}
                }

                // THE TAKER ORDER UPDATE
                let taker_status = if taker_remaining.is_zero() {
                    InternalOrderStatus::Filled 
                } else if taker_remaining < original_quantity {
                    InternalOrderStatus::PartiallyFilled
                } else {
                    InternalOrderStatus::Placed // It matched nothing, just sitting in the book
                };


                // calculate the time for taker order
                let timestamp = SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .expect("Time went backwards") // This only happens if the system clock is reset
                                    .as_micros() as u64;
                let taker_order = OrderUpdate {
                    engine_id: req.engine_id.unwrap_or(0), // Assigned before match_or_rest
                    user_id: req.user_id,
                    market: req.market as i32,
                    price: req.price.unwrap_or_default().to_string(),
                    quantity: original_quantity.to_string(),
                    filled_quantity: (original_quantity - taker_remaining).to_string(),
                    side: req.side as i32,
                    status: taker_status as i32,
                    timestamp
                };

                let _ = self.order_transmitter.try_send(taker_order);

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