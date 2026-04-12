use std::collections::HashMap;

use prost::Message;
use redis::AsyncCommands;
use tokio::{net::unix::pipe::Receiver, sync::mpsc};

use crate::model::{DepthResponse, InternalTrade, exchange_proto::{MarketId, Trade}};




pub struct RedisPublisher {
    pub trade_receiver : mpsc::Receiver<InternalTrade>,
    pub depth_receiver : mpsc::Receiver<DepthResponse>,
    pub redis_url : String,
    pub trade_channels : HashMap<MarketId , &'static str>,
    pub depth_channels : HashMap<MarketId, &'static str>,
    pub db_queue : &'static str
}

impl RedisPublisher {
    pub fn new(trade_receiver :mpsc::Receiver<InternalTrade>, depth_receiver : mpsc::Receiver<DepthResponse>, redis_url: String) -> Self {

        let mut trade_channels = HashMap::new();

        trade_channels.insert(MarketId::BtcUsdt, "trades:btcusdt");
        trade_channels.insert(MarketId::EthUsdt, "trades:ethusdt");

        let mut depth_channels = HashMap::new();

        depth_channels.insert(MarketId::BtcUsdt, "depth:btcusdt");
        depth_channels.insert(MarketId::EthUsdt, "depth:ethusdt");

        Self { 
            trade_receiver,
            depth_receiver,
            redis_url,
            trade_channels,
            depth_channels,
            db_queue : "db_processor"
        }
    }

    pub async fn run(mut self) {
        let client = redis::Client::open(self.redis_url).unwrap();
        let mut conn = client.get_multiplexed_async_connection()
            .await
            .expect("Redis pub sub error");
        println!("Publisher is online, Multiplexed connection established");


        let mut batch = Vec::with_capacity(100); // 100 is a sweet spot for throughput

        while let Some(internal_trade) = self.trade_receiver.recv().await {
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

            
            // very important concept!!!!!!!!!!!!!!!

            // 3. publish to redis channel 
            // we use the market_id to craete a dynamic channel name
            let channel = self.trade_channels.get(&internal_trade.market).expect("No such channel available");
            batch.push((channel, payload));
            
            // 3. TRIGGER FLUSH: If batch is full OR the channel is currently empty
            // receiver.is_empty() is great for "Opportunistic Batching"
            if batch.len() >= 100 || self.trade_receiver.is_empty() {
                let mut pipe = redis::pipe();
                
                for (chan, data) in batch.drain(..) {
                    // Broadcast (Pub/Sub) for Websocket and frontend (Live market data)
                    pipe.publish(chan, data.clone());

                    // Persist (List/Queue): For the DB worker
                    // We use Lpush to put it into the db_processor queue
                    pipe.lpush(self.db_queue, data); 

                }

                // Execute the whole batch in ONE network round-trip
                let _: redis::RedisResult<()> = pipe.query_async(&mut conn).await;
            }
            
        }
    }
}