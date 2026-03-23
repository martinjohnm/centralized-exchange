use std::collections::HashMap;

use rust_decimal::Decimal;

use crate::model::{Asset, UserId};

pub struct Ledger {
    // Key : user_id
    // Value: A map of asset symbols (eg : "BTC") to their sepcific account
    pub accounts : HashMap<UserId, HashMap<Asset, Account>>
}

impl Ledger {
    pub fn new () -> Self {
        Self {
            accounts : HashMap::new()
        }
    }



    // =========== External (I/O with redis or db) ======================

    pub fn deposite() {

    }

    pub fn withdraw() {

    }

    // Internal helper to get or create an account balance
    fn get_account_mut() {
        
    }

    // =========== Internal (The hot trading path) ======================

    pub fn lock_funds() {

    }

    pub fn unlock_funds() {

    }

    pub fn settle_trade() {

    }
}

pub struct Account {
    pub available: Decimal,
    pub locked : Decimal
}