use rust_decimal::Decimal;




#[derive(Debug, Clone, Copy)]
pub enum Side {
    Bid,
    Ask
}
#[derive(Debug, Clone)]
pub struct Order {
    pub id : u64,
    pub user_id : u64,
    pub quantity : Decimal,
    pub price : Decimal,
    pub side : Side
}


pub struct Trade {
    pub maker_id : u64, 
    pub taker_id : u64,
    pub price : Decimal,
    pub quantity : Decimal
}

#[derive(Debug, Clone)]
pub struct OrderRequest {
    pub user_id: u64,
    pub price: Decimal,
    pub quantity: Decimal, 
    pub side: Side,
    pub symbol: String,    // "BTC_USDT"
}