use rust_decimal::Decimal;
use std::collections::HashMap;

use crate::trade::model::Side;


#[derive(Debug, Clone, Copy)]
pub struct Balance {
    pub available : Decimal, // Free to trade or withdraw
    pub locked : Decimal // Escrawed in open orders
}

pub struct Bank {
    // Key : UserId -> (Key: Assetsymbol -> Balance)
    pub accounts : HashMap<u64, HashMap<String, Balance>>
}

impl Bank {
    pub fn new() -> Self {
        Self { accounts: HashMap::new() }
    }

    /// Moved fund from available to locked
    /// Returns error if the user dosent have enoguh Available funds.
    pub fn lock_funds(&mut self, user_id: u64, asset: &str, amount: Decimal) -> Result<(), String> {
        let user_map = self.accounts.entry(user_id).or_default();
        let bal : Balance = user_map.entry(asset.to_uppercase()).or_default();        
        

        if bal.available < amount {
            return Err(format!("Insufficient {} availabel", asset));
        };

        bal.available -= amount;
        bal.locked += amount;
        Ok(())
    }

    pub fn settle_trade(
        &mut self,
        maker_id: u64,
        taker_id: u64,
        base_qty: Decimal,  // eg: 0.5 BTC   
        quote_qty: Decimal, // eg: 25,000 USDT (Price* Qty)
        base_asset: &str,
        quote_asset: &str,
        taker_side: Side 
    ) {
        match taker_side {
            Side::Bid => { // Taker Buys BTC
                // Taker gives USDT (Locked) -> Maker gets USDT (Available)
                self.transfer(taker_id, maker_id, quote_asset, quote_qty, true);
                // Maker gives BTC (Locked) -> Taker gets BTC (Available)
                self.transfer(maker_id, taker_id, base_asset, base_qty, true);

            },
            Side::Ask => {}
        }
    }

    fn transfer(&mut self, from: u64, to: u64, asset: &str, amount: Decimal, from_locked: bool) {
        // Sustract from sender
        if let Some(user_assets) = self.accounts.get_mut(&from) {
            if let Some(bal) = user_assets.get_mut(asset) {
                if from_locked {
                    bal.locked -= amount
                } else {
                    bal.available -= amount
                }
            } 

            // Add to receiver
            let to_bal : Balance = self.accounts.entry(to).or_default().entry(asset.to_string()).or_default();
            to_bal.available += amount;
        } 
    }

    pub fn unlock_funds(&mut self, user_id: u64, asset: &str, amount: Decimal)  {
        if let Some(user_assets) = self.accounts.get_mut(&user_id) {
            if let Some(bal) = user_assets.get_mut(asset) {
                // Return to available
                bal.locked -= amount;
                bal.available += amount
            }
        }
    }

    // Helper for testing
    pub fn deposit(&mut self, user_id: u64, asset: &str, amount: Decimal) {
        let bal: Balance = self.accounts.entry(user_id).or_default().entry(asset.to_uppercase()).or_default();
        bal.available += amount
    }
}