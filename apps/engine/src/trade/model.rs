use rust_decimal::Decimal;




#[derive(Debug, Clone, Copy)]
pub enum Side {
    Bid,
    Ask
}
#[derive(Debug, Clone)]
pub struct Order {
    pub id : u64,
    pub quantity : Decimal,
    pub price : Decimal,
    pub side : Side
}
