use rust_decimal::Decimal;
use std::collections::{BTreeMap, VecDeque};

#[derive(Debug, Clone)]
pub enum Side {
    Bid,
    Ask
}
#[derive(Debug, Clone)]
pub struct Order {
    pub id : u64,
    pub amount : Decimal,
    pub price : Decimal,
    pub side : Side
}

pub struct Orderbook {
    // Bids : Sorted descending (Highest price first)
    pub bids : BTreeMap<Decimal, VecDeque<Order>>,

    // Asks : Sorted Ascending (Lowest pirce first)
    pub asks : BTreeMap<Decimal, VecDeque<Order>>
}

impl Orderbook {
    pub fn new() -> Self {
        Self { 
            bids: BTreeMap::new(), 
            asks: BTreeMap::new()
        }
    }

    pub fn add_order(&mut self, order: Order) {

        // use the side defined inside the order struct
        let side_map = match order.side {
            Side::Ask => &mut self.asks,
            Side::Bid => &mut self.bids
        };

        // Entry API handles the rest
        side_map
            .entry(order.price)
            .or_insert_with(|| VecDeque::new())
            .push_back(order);
    }

    // Fn to get best bid (Highest)
    pub fn best_bid(&self) -> Option<Decimal> {
        self.bids.keys().next_back().cloned()
    }
    // Fn to get best ask (Lowest)
    pub fn best_ask(&self) -> Option<Decimal> {
        self.asks.keys().next().cloned()
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use super::*;
    
    #[test]
    fn test_orderbook_sorting() {
        let mut book = Orderbook::new();

        // Add bids out of order
        book.add_order(Order { id: 1, amount: dec!(1), price: dec!(100), side: Side::Bid });
        book.add_order(Order { id: 2, amount: dec!(1), price: dec!(150), side: Side::Bid });
        book.add_order(Order { id: 3, amount: dec!(1), price: dec!(120), side: Side::Bid });
        
        // Best bid should be highest (150)
        assert_eq!(book.best_bid(), Some(dec!(150)));

        // Add asks out of order 
        book.add_order(Order { id: 1, amount: dec!(1), price: dec!(300), side: Side::Ask });
        book.add_order(Order { id: 2, amount: dec!(1), price: dec!(290), side: Side::Ask });
        book.add_order(Order { id: 3, amount: dec!(1), price: dec!(450), side: Side::Ask });

        // Best ask should be the Lowest (290)
        assert_eq!(book.best_ask(), Some(dec!(290)));


    }

    #[test]
    fn test_empty_book_behavior() {
        let book = Orderbook::new();
        assert_eq!(book.best_bid(), None);
        assert_eq!(book.best_ask(), None);
    }

    #[test]
    fn test_price_time_priority() {
        let mut book = Orderbook::new();

        // Add first bid at 100
        book.add_order(Order { id: 1, amount: dec!(1), price: dec!(100), side: Side::Bid });

        // Add second bid at 100
        book.add_order(Order { id: 2, amount: dec!(1), price: dec!(100), side: Side::Bid });
        
        // Add third bid at 101
        book.add_order(Order { id: 3, amount: dec!(1), price: dec!(101), side: Side::Bid });

        // Best bid must be 101
        assert_eq!(book.best_bid(), Some(dec!(101)));

        // Now check the internal structure for the 100 price
        let bids_at_100 = book.bids.get(&dec!(100)).unwrap();
        assert_eq!(bids_at_100[0].id, 1);
        assert_eq!(bids_at_100[1].id, 2);

        if let Some(bids) = book.bids.get(&dec!(100)) {
            println!("--- Current Bids at $100 ---");
            println!("{:#?}", bids); 
            println!("---------------------------");
        }


        // Add first ask
        book.add_order(Order { id: 4, amount: dec!(1), price: dec!(110), side: Side::Ask });

        // Add second ask
        book.add_order(Order { id: 5, amount: dec!(1), price: dec!(108), side: Side::Ask });

        // Add third ask
        book.add_order(Order { id: 6, amount: dec!(1), price: dec!(110), side: Side::Ask });

        assert_eq!(book.best_ask(), Some(dec!(108)));

        let asks_at_110 = book.asks.get(&dec!(110)).unwrap();
        // check the internal structure of the 110 price asks

        assert_eq!(asks_at_110[0].id, 4);
        assert_eq!(asks_at_110[1].id, 6);

        if let Some(asks) = book.asks.get(&dec!(110)) {
            println!("--- Current Asks at $110 ---");
            println!("{:#?}", asks); 
            println!("---------------------------");
        }

    }
}