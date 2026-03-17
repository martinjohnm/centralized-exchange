use rust_decimal::Decimal;
use std::collections::HashMap;


#[derive(Debug, Clone, Copy)]
pub struct Balance {
    pub available : Decimal,
    pub locked : Decimal
}

pub struct Bank {
    // Key : UserId -> (Key: Assetsymbol -> Balance)
    pub accounts : HashMap<u64, HashMap<String, Balance>>
}

impl Bank {
    
}