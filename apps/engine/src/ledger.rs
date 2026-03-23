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
    fn get_account_mut(&mut self, user_id: UserId, asset : Asset) -> &mut Account {
        self.accounts
            .entry(user_id)
            .or_insert_with(|| HashMap::new())
            .entry(asset)
            .or_insert(Account { available: Decimal::ZERO, locked: Decimal::ZERO })
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

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_get_account_mut_initialization() {
        let mut ledger = Ledger::new();
        let user_id = 1;
        let asset = "USDT".to_string();

        // 1. Accessing a non-existent account should initialize it to zero
        {
            let account = ledger.get_account_mut(user_id, asset.clone());
            assert_eq!(account.available, dec!(0));
            assert_eq!(account.locked, dec!(0));
            
            // Modify the account
            account.available = dec!(100);
        }

        // 2. Accessing it again should return the modified value, not a new zeroed one
        let account_again = ledger.get_account_mut(user_id, asset);
        assert_eq!(account_again.available, dec!(100));
    }

    #[test]
    fn test_multiple_assets_per_user() {
        let mut ledger = Ledger::new();
        let user_id = 1;

        // Initialize two different assets for the same user
        ledger.get_account_mut(user_id, "BTC".to_string()).available = dec!(1);
        ledger.get_account_mut(user_id, "USDT".to_string()).available = dec!(50000);

        assert_eq!(ledger.get_account_mut(user_id, "BTC".to_string()).available, dec!(1));
        assert_eq!(ledger.get_account_mut(user_id, "USDT".to_string()).available, dec!(50000));
    }
}