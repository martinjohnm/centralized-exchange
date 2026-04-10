use tokio::sync::broadcast;

use crate::{candle::Candle, model::{InternalTrade, WsOutMessage, exchange_proto::Trade}};

use prost::Message as ProtoMessage;


pub async fn start_aggregator(
    interval: (&str, u64),
    mut internal_rx : tokio::sync::mpsc::Receiver<Vec<u8>>,
    broadcast_tx: broadcast::Sender<Vec<u8>>
) {
    let mut current_candle  = Candle::default();
    let mut tiker = tokio::time::interval(tokio::time::Duration::from_secs(1));
    
    loop {
        tokio::select! {
            Some(payload) = internal_rx.recv() => {
                if let Ok(proto) = Trade::decode(&payload[..]) {
                    let trade = InternalTrade::from_proto(proto);
                    let bucket_ts = (trade.timestamp / interval.1) * interval.1;

                    if current_candle.timestamp != 0 && bucket_ts > current_candle.timestamp {
                        current_candle = Candle::default();
                    }

                    current_candle.timestamp = bucket_ts;

                    current_candle.update(trade.price, trade.quantity, trade.timestamp);
                }
            }

            _ = tiker.tick() => {
                if current_candle.open > 0.0 {
                    // Wrap the candle in the OutMessage Enum
                    let out_msg = WsOutMessage::Candle(current_candle.clone());
                    if let Ok(bytes) = serde_json::to_vec(&out_msg) {
                        let _ = broadcast_tx.send(bytes);
                    }
                }
            }
        }
    }
}