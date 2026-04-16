use tokio::sync::broadcast;
use prost::Message;

use crate::{candle::InternalCandle, model::{InternalTrade, exchange_proto::{Candle, MarketId, StreamType, Trade, WsOutMessage, ws_out_message::Data}}};

use prost::Message as ProtoMessage;


pub async fn start_aggregator(
    interval: (&str, u64),
    mut internal_rx : tokio::sync::mpsc::Receiver<Vec<u8>>,
    broadcast_tx: broadcast::Sender<Vec<u8>>
) {
    let mut current_candle  = InternalCandle::default();
    let mut tiker = tokio::time::interval(tokio::time::Duration::from_secs(1));
    
    loop {
        tokio::select! {
            Some(payload) = internal_rx.recv() => {
                if let Ok(proto) = Trade::decode(&payload[..]) {
                    let trade = InternalTrade::from_proto(proto);
                    let bucket_ts = (trade.timestamp / interval.1) * interval.1;

                    if current_candle.timestamp != 0 && bucket_ts > current_candle.timestamp {
                        current_candle = InternalCandle::default();
                    }

                    current_candle.timestamp = bucket_ts;

                    current_candle.update(trade.price, trade.quantity, trade.timestamp);
                }
            }

            _ = tiker.tick() => {
            if current_candle.open > 0.0 {
                // 1. Create the Proto Candle (Mapping f64 to String)
                let proto_candle = Candle {
                    open: current_candle.open.to_string(),
                    high: current_candle.high.to_string(),
                    low: current_candle.low.to_string(),
                    close: current_candle.close.to_string(),
                    volume: current_candle.volume.to_string(),
                    timestamp: current_candle.timestamp,
                };

                // 2. Wrap in the Unified Message
                let out_msg = WsOutMessage {
                    stream: StreamType::Candle as i32,
                    data: Some(Data::Candle(proto_candle)),
                    // Add market context if your WsOutMessage has a market field
                };

                // 3. Encode and Broadcast
                let mut payload = Vec::new();
                if out_msg.encode(&mut payload).is_ok() {
                    let _ = broadcast_tx.send(payload);
                }
            }
        }
        }
    }
}