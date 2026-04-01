use tokio::sync::broadcast;

use crate::{candle::Candle, model::{InternalTrade, exchange_proto::Trade}};

use prost::Message as ProtoMessage;


pub async fn start_aggregator(
    mut internal_rx : tokio::sync::mpsc::Receiver<Vec<u8>>,
    broadcast_tx: broadcast::Sender<Vec<u8>>
) {
    let mut current_candle  = Candle::default();
    let mut tiker = tokio::time::interval(tokio::time::Duration::from_secs(1));
    const MICROS_PER_MINUTE : u64 = 60_000_000;
    
    loop {
        tokio::select! {
            Some(payload) = internal_rx.recv() => {
                if let Ok(proto) = Trade::decode(&payload[..]) {
                    let trade = InternalTrade::from_proto(proto);
                    let bucket_ts = (trade.timestamp / MICROS_PER_MINUTE) * MICROS_PER_MINUTE;

                    if current_candle.timestamp != 0 && bucket_ts > current_candle.timestamp {
                        current_candle = Candle::default();
                    }

                    current_candle.timestamp = bucket_ts;

                    current_candle.update(trade.price, trade.quantity, trade.timestamp);
                }
            }

            _ = tiker.tick() => {
                if current_candle.open > 0.0 {
                    if let Ok(bytes) = serde_json::to_vec(&current_candle) {
                        let _ = broadcast_tx.send(bytes);
                    }
                }
            }
        }
    }
}