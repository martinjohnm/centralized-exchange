use prost::Message;
use redis::AsyncCommands;
use tokio::{net::unix::pipe::Receiver, sync::mpsc};

use crate::model::{InternalTrade, exchange_proto::Trade};




pub struct RedisPublisher {
    pub receiver : mpsc::Receiver<InternalTrade>,
    pub redis_url : String
}

impl RedisPublisher {
    pub fn new(receiver :mpsc::Receiver<InternalTrade>, redis_url: String) -> Self {
        Self { 
            receiver,
            redis_url 
        }
    }

    pub async fn run(mut self) {
        let client = redis::Client::open(self.redis_url).unwrap();
        let mut conn = client.get_multiplexed_async_connection()
            .await
            .expect("Redis pub sub error");
        println!("Publisher is online, Multiplexed connection established");

        while let Some(internal_trade) = self.receiver.recv().await {
            // 1. transform the internal_trade to Trade (Proto)
            let proto_trade = Trade {
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
            };

            // 2. Serialize to binary (Protobuf bytes)
            let mut payload = Vec::new();
            if let Err(e) = proto_trade.encode(&mut payload) {
                eprintln!("Failed to encode trade: {}", e);
                continue;
            }

            // 3. publish to redis channel 
            // we use the market_id to craete a dynamic channel name
            let channel = format!("trades:{:?}", internal_trade.market).to_lowercase();
            let result: redis::RedisResult<i32> = conn.publish(&channel, payload).await;
            let _: () = match result {
                Ok(count) => {
                    // track the listeners
                },
                Err(e) => eprintln!("Redis publish error: {}", e)
            };
        }
    }
}