use std::collections::HashMap;

use rust_decimal::Decimal;

use crate::model::{Asset, UserId};

pub struct Ledger {
    // Key : user_id
    // Value: A map of asset symbols (eg : "BTC") to their sepcific account
    pub accounts : HashMap<UserId, HashMap<Asset, Account>>
}

impl Ledger {
    fn new () -> Self {
        Self {
            accounts : HashMap::new()
        }
    }
}

pub struct Account {
    pub available: Decimal,
    pub locked : Decimal
}