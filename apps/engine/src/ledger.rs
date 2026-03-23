use std::collections::HashMap;

use rust_decimal::Decimal;

use crate::model::{Asset, LedgerError, UserId};

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

    pub fn deposit(&mut self, user_id : UserId, asset : Asset, amount : Decimal) {
        let account = self.get_account_mut(user_id, &asset);
        account.available += amount
    }

    pub fn withdraw(&mut self, user_id : UserId, asset : Asset, amount : Decimal) -> Result<(), LedgerError> {
        let account = self.get_account_mut(user_id, &asset);

        if account.available >= amount {
            account.available -= amount;
            Ok(())
        } else {
            Err(LedgerError::InsufficientFunds { 
                user_id, 
                asset, 
                available: account.available, 
                requested: amount
            })
        }
    }

    // Internal helper to get or create an account balance
    fn get_account_mut(&mut self, user_id: UserId, asset : &Asset) -> &mut Account {
        self.accounts
            .entry(user_id)
            .or_insert_with(|| HashMap::new())
            .entry(asset.clone())
            .or_insert(Account { available: Decimal::ZERO, locked: Decimal::ZERO })
        
    }

    // =========== Internal (The hot trading path) ======================

    // Move money from available to locked (open order)
    pub fn lock_funds(&mut self, user_id : UserId, asset: Asset, amount : Decimal) -> Result<(), LedgerError> {
        let account = self.get_account_mut(user_id, &asset);

        if account.available >= amount {
            account.available -= amount;
            account.locked += amount;
            Ok(())
        } else {
            Err(LedgerError::InsufficientFunds { 
                user_id, 
                asset, 
                available: account.available, 
                requested: amount 
            })
        }


    }

    pub fn unlock_funds(&mut self, user_id : UserId, asset : Asset, amount : Decimal) {
        let account = self.get_account_mut(user_id, &asset);

        // assume orderbook validated this amount
        account.locked -= amount;
        account.available += amount;
    }

    // Moving quote and base togethor
    pub fn settle_trade(
        &mut self,
        buyer : UserId,
        seller : UserId,
        quote_asset: Asset, // eg: USDT / INR
        base_asset: Asset, //  eg: BTC / TATA
        qty : u64,
        match_price : Decimal // always the makers price
    ) {
        let total_quote = match_price * Decimal::from(qty);
        let total_base = Decimal::from(qty);

        // ======== Settle the money (Quote asset)
        // -------- Buyer pays from LOCKED (since they already committed)----
        self.get_account_mut(buyer, &quote_asset).locked -= total_quote;
        // -------- Seller receives into AVAILABLE (ready to spent or withdraw)
        self.get_account_mut(seller, &quote_asset).available += total_quote;

        // ======== Settle the Asset (Base asset) ===
        // -------- Seller gives from locked (Since their asset was sitting in the book)
        self.get_account_mut(seller, &base_asset).locked -= total_base;
        // -------- Buyer receives into AVAILABLE ---------------
        self.get_account_mut(buyer, &base_asset).available += total_base; 


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
            let account = ledger.get_account_mut(user_id, &asset);
            assert_eq!(account.available, dec!(0));
            assert_eq!(account.locked, dec!(0));
            
            // Modify the account
            account.available = dec!(100);
        }

        // 2. Accessing it again should return the modified value, not a new zeroed one
        let account_again = ledger.get_account_mut(user_id, &asset);
        assert_eq!(account_again.available, dec!(100));
    }

    #[test]
    fn test_multiple_assets_per_user() {
        let mut ledger = Ledger::new();
        let user_id = 1;

        // Initialize two different assets for the same user
        ledger.get_account_mut(user_id, &"BTC".to_string()).available = dec!(1);
        ledger.get_account_mut(user_id, &"USDT".to_string()).available = dec!(50000);

        assert_eq!(ledger.get_account_mut(user_id, &"BTC".to_string()).available, dec!(1));
        assert_eq!(ledger.get_account_mut(user_id, &"USDT".to_string()).available, dec!(50000));
    }

    
}