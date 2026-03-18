use rust_decimal::Decimal;
use std::collections::HashMap;

use crate::trade::model::Side;


#[derive(Debug, Clone, Copy, Default)]
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
        let bal : &mut Balance = user_map.entry(asset.to_uppercase()).or_default();        
        

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
        matched_qty: Decimal,  // eg: 0.5 BTC   
        quote_cost: Decimal, // eg: 25,000 USDT (Price* Qty)
        base_asset: &str,
        quote_asset: &str,
        taker_side: Side 
    ) {
        match taker_side {
            Side::Bid => { // Taker Buys BTC
                // Taker gives USDT (Locked) -> Maker gets USDT (Available)
                self.transfer(taker_id, maker_id, quote_asset, quote_cost, true);
                // Maker gives BTC (Locked) -> Taker gets BTC (Available)
                self.transfer(maker_id, taker_id, base_asset, matched_qty, true);

            },
            Side::Ask => { // Taker Sells BTC
                // Taker gives BTC (Locked) -> Maker gets BTC (Available)
                self.transfer(taker_id, maker_id, base_asset, matched_qty, true);
                // Maker gives USDT (Locked) -> Taker gets USDT (Available)
                self.transfer(maker_id, taker_id, quote_asset, quote_cost, true);
            }
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
            let to_bal : &mut Balance = self.accounts.entry(to).or_default().entry(asset.to_string()).or_default();
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
        let bal: &mut Balance = self.accounts.entry(user_id).or_default().entry(asset.to_uppercase()).or_default();
        bal.available += amount
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_partial_fill_with_price_improvement_integrity() {
        let mut bank = Bank::new();
        let btc = "BTC";
        let usdt = "USDT";

        // 1. SETUP: Initial Deposits
        // Maker (Seller) has 1 BTC. Taker (Buyer) has 50,000 USDT.
        bank.deposit(1, btc, dec!(1.0));
        bank.deposit(2, usdt, dec!(50000.0));

        // 2. LOCK: Maker places Limit Sell at $49,000
        bank.lock_funds(1, btc, dec!(1.0)).unwrap();

        // 3. LOCK: Taker places Limit Buy at $50,000 (Over-collateralized)
        // We lock the FULL $50k because that's their "Max Budget"
        bank.lock_funds(2, usdt, dec!(50000.0)).unwrap();

        // 4. MATCH: Partial Fill happens! 
        // Taker buys only 0.4 BTC at the Maker's price ($49,000)
        let matched_qty = dec!(0.4);
        let matched_price = dec!(49000.0);
        let quote_cost = matched_qty * matched_price; // $19,600

        bank.settle_trade(
            1, 2, 
            matched_qty, quote_cost, 
            btc, usdt, 
            Side::Bid
        );

        // 5. VERIFY: Intermediate Balances
        let maker_btc = bank.accounts.get(&1).unwrap().get(btc).unwrap();
        let taker_usdt = bank.accounts.get(&2).unwrap().get(usdt).unwrap();

        // Maker should have 0.6 BTC still LOCKED (1.0 - 0.4)
        assert_eq!(maker_btc.locked, dec!(0.6));
        // Taker should have $30,400 still LOCKED (50,000 - 19,600)
        assert_eq!(taker_usdt.locked, dec!(30400.0));

        // 6. CLEANUP: Taker cancels the rest of the order
        // The "Price Improvement" ($1,000) and the "Unused Budget" ($29,400) 
        // are both sitting in LOCKED. We must return all $30,400.
        bank.unlock_funds(2, usdt, dec!(30400.0));

        // 7. FINAL INVARIANT CHECK
        let final_taker_btc = bank.accounts.get(&2).unwrap().get(btc).unwrap().available;
        let final_maker_usdt = bank.accounts.get(&1).unwrap().get(usdt).unwrap().available;

        assert_eq!(final_taker_btc, dec!(0.4));   // Taker got their 0.4 BTC
        assert_eq!(final_maker_usdt, dec!(19600.0)); // Maker got their $19.6k
        
        // CRITICAL: Total BTC in system must still be 1.0
        // Total USDT in system must still be 50,000.0
    }
}