use std::collections::{BTreeMap, VecDeque}; // Import VecDeque
use rust_decimal::Decimal;

use crate::trade::model::Order;

// Change Vec to VecDeque here
pub fn debug_print_book(
    bids: &BTreeMap<Decimal, VecDeque<Order>>, 
    asks: &BTreeMap<Decimal, VecDeque<Order>>
) {
    println!("\n--- L2 MARKET DEPTH ---");
    
    // Print top 3 Asks (Sell Side)
    for (price, orders) in asks.iter().rev() {
        let total_qty: Decimal = orders.iter().map(|o| o.quantity).sum();
        println!("\x1b[31mASK | Price: {:>10} | Total Qty: {:>10}\x1b[0m", price, total_qty);
    }

    println!("---------- SPREAD ----------");

    // Print top 3 Bids (Buy Side)
    for (price, orders) in bids.iter().rev() {
        let total_qty: Decimal = orders.iter().map(|o| o.quantity).sum();
        println!("\x1b[32mBID | Price: {:>10} | Total Qty: {:>10}\x1b[0m", price, total_qty);
    }
}