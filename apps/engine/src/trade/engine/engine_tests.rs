

use super::*; // Access private logic in engine.rs
use rust_decimal_macros::dec;

#[cfg(test)]
mod tests {
    use std::process::id;


    use crate::trade::model::{OrderMetadata, OrderType};

    use super::*;

    #[test]
    fn test_buy_order_full_fills_and_single_level() {
        let mut engine = MatchingEngine::new(String::from("BTC/USDT"));

        // Add bids 
        engine.orderbook.add_order(Order { id: 1,user_id: 1, quantity: dec!(1), price: dec!(101),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });
        engine.orderbook.add_order(Order { id: 2,user_id: 1, quantity: dec!(1), price: dec!(102),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });
        engine.orderbook.add_order(Order { id: 3,user_id: 1, quantity: dec!(1), price: dec!(103),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });

        // Add bids
        engine.orderbook.add_order(Order { id: 4,user_id: 2, quantity: dec!(1), price: dec!(99),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });
        engine.orderbook.add_order(Order { id: 5,user_id: 2, quantity: dec!(1), price: dec!(98),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });
        engine.orderbook.add_order(Order { id: 6,user_id: 2, quantity: dec!(1), price: dec!(97),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });

        let id_ref: &u64 = &1;
        assert_eq!(engine.orderbook.orders_lookup.get(id_ref).unwrap(), &OrderMetadata {
            price : dec!(101),
            side : Side::Ask
        });
        // sent an buy order with quantity 1 and price : 101 (exact match for the best ask)
        engine.process_order(Order { id:7,user_id: 3, quantity: dec!(1), price: dec!(101),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });

        assert_eq!(engine.orderbook.best_ask(), Some(dec!(102)));
        let asks_at_102 = engine.orderbook.get_level_mut(dec!(102), Side::Ask).unwrap();
        assert_eq!(asks_at_102[0].id, 2);

    }

    #[test]
    fn test_buy_order_single_level_low_price_fill_complete() {
        let mut engine = MatchingEngine::new(String::from("BTC/USDT"));

        // Add bids 
        engine.orderbook.add_order(Order { id: 1,user_id: 1, quantity: dec!(3), price: dec!(100),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });
        engine.orderbook.add_order(Order { id: 2,user_id: 1, quantity: dec!(1), price: dec!(101),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });
        engine.orderbook.add_order(Order { id: 3,user_id: 1, quantity: dec!(2), price: dec!(102),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });

        // Add bids
        engine.orderbook.add_order(Order { id: 4,user_id: 2, quantity: dec!(1), price: dec!(99),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });
        engine.orderbook.add_order(Order { id: 5,user_id: 2, quantity: dec!(1), price: dec!(98),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });
        engine.orderbook.add_order(Order { id: 6,user_id: 2, quantity: dec!(1), price: dec!(97),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });

        // sent an buy order with quantity 4 and price : 101 (3 order match from the 100 and 1 from the 101)
        engine.process_order(Order { id:7,user_id: 3, quantity: dec!(3), price: dec!(102),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });

        assert_eq!(engine.orderbook.best_ask(), Some(dec!(101)));

        let bids_at_101 =  engine.orderbook.get_level_mut(dec!(101), Side::Ask).unwrap();
        assert_eq!(bids_at_101[0].quantity, dec!(1));
        assert_eq!(bids_at_101[0].id, 2);
    }

    #[test]
    fn test_buy_order_multiple_level_full_fills() {
        let mut engine = MatchingEngine::new(String::from("BTC/USDT"));

        // Add bids 
        engine.orderbook.add_order(Order { id: 1,user_id: 1, quantity: dec!(3), price: dec!(100),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });
        engine.orderbook.add_order(Order { id: 2,user_id: 1, quantity: dec!(1), price: dec!(101),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });
        engine.orderbook.add_order(Order { id: 3,user_id: 1, quantity: dec!(2), price: dec!(102),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });

        // Add bids
        engine.orderbook.add_order(Order { id: 4,user_id: 2, quantity: dec!(1), price: dec!(99),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });
        engine.orderbook.add_order(Order { id: 5,user_id: 2, quantity: dec!(1), price: dec!(98),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });
        engine.orderbook.add_order(Order { id: 6,user_id: 2, quantity: dec!(1), price: dec!(97),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });

        // sent an buy order with quantity 4 and price : 101 (3 order match from the 100 and 1 from the 101)
        engine.process_order(Order { id:7,user_id: 3, quantity: dec!(4), price: dec!(101),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });

        assert_eq!(engine.orderbook.best_ask(), Some(dec!(102)))
    }

    #[test]
    fn test_buy_order_multiple_level_partial_fills() {
        let mut engine = MatchingEngine::new(String::from("BTC/USDT"));

        // Add bids 
        engine.orderbook.add_order(Order { id: 1,user_id: 1, quantity: dec!(3), price: dec!(100),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });
        engine.orderbook.add_order(Order { id: 2,user_id: 1, quantity: dec!(1), price: dec!(101),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });
        engine.orderbook.add_order(Order { id: 3,user_id: 1, quantity: dec!(2), price: dec!(102),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });

        // Add bids
        engine.orderbook.add_order(Order { id: 4,user_id: 2, quantity: dec!(1), price: dec!(99),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });
        engine.orderbook.add_order(Order { id: 5,user_id: 2, quantity: dec!(1), price: dec!(98),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });
        engine.orderbook.add_order(Order { id: 6,user_id: 2, quantity: dec!(1), price: dec!(97),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });

        // sent an buy order with quantity 4 and price : 101 (3 order match from the 100 and 1 from the 101)
        engine.process_order(Order { id:7,user_id: 3, quantity: dec!(5), price: dec!(102),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });

        assert_eq!(engine.orderbook.best_ask(), Some(dec!(102)));

        let bids_at_102 =  engine.orderbook.get_level_mut(dec!(102), Side::Ask).unwrap();
        assert_eq!(bids_at_102[0].quantity, dec!(1));
        assert_eq!(bids_at_102[0].id, 3);
    }



    #[test]
    fn test_whale_activity_of_eating_entire_asks () {
        let mut engine = MatchingEngine::new(String::from("BTC/USDT"));

        // Add bids 
        engine.orderbook.add_order(Order { id: 1,user_id: 1, quantity: dec!(4), price: dec!(100),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });
        engine.orderbook.add_order(Order { id: 2,user_id: 1, quantity: dec!(5), price: dec!(101),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });
        
        engine.process_order(Order { id:3,user_id: 2, quantity: dec!(10), price: dec!(110),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });

        assert_eq!(engine.orderbook.best_bid(), Some(dec!(110)));
        let bids_at_110 =  engine.orderbook.get_level_mut(dec!(110), Side::Bid).unwrap();
        assert_eq!(bids_at_110[0].quantity, dec!(1));
        assert_eq!(bids_at_110[0].id, 3);
    }

    #[test]
    fn test_single_level_sell_order() {
        let mut engine = MatchingEngine::new(String::from("BTC/USDT"));

        // Add bids 
        engine.orderbook.add_order(Order { id: 1,user_id: 1, quantity: dec!(1), price: dec!(101),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });
        engine.orderbook.add_order(Order { id: 2,user_id: 1, quantity: dec!(1), price: dec!(102),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });
        engine.orderbook.add_order(Order { id: 3,user_id: 1, quantity: dec!(1), price: dec!(103),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });

        // Add bids
        engine.orderbook.add_order(Order { id: 4,user_id: 2, quantity: dec!(1), price: dec!(99),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });
        engine.orderbook.add_order(Order { id: 5,user_id: 2, quantity: dec!(1), price: dec!(98),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });
        engine.orderbook.add_order(Order { id: 6,user_id: 2, quantity: dec!(1), price: dec!(97),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });

        // sent an sell order with quantity 1 and price : 99 (exact match for the best abidsk)
        engine.process_order(Order { id:7,user_id: 3, quantity: dec!(1), price: dec!(99),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });

        assert_eq!(engine.orderbook.best_bid(), Some(dec!(98)));
        let bids_at_98 = engine.orderbook.get_level_mut(dec!(98), Side::Bid).unwrap();
        assert_eq!(bids_at_98[0].id, 5);
    }

    #[test]
    fn test_single_level_partial_fill_sell_order() {
        let mut engine = MatchingEngine::new(String::from("BTC/USDT"));

        // Add bids 
        engine.orderbook.add_order(Order { id: 1,user_id: 1, quantity: dec!(1), price: dec!(101),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });
        engine.orderbook.add_order(Order { id: 2,user_id: 1, quantity: dec!(1), price: dec!(102),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });
        engine.orderbook.add_order(Order { id: 3,user_id: 1, quantity: dec!(1), price: dec!(103),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });

        // Add bids
        engine.orderbook.add_order(Order { id: 4,user_id: 2, quantity: dec!(2), price: dec!(99),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });
        engine.orderbook.add_order(Order { id: 5,user_id: 2, quantity: dec!(1), price: dec!(98),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });
        engine.orderbook.add_order(Order { id: 6,user_id: 2, quantity: dec!(1), price: dec!(97),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });

        // sent an sell order with quantity 1 and price : 99 (exact match for the best bid)
        engine.process_order(Order { id:7,user_id: 3, quantity: dec!(1), price: dec!(99),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });

        assert_eq!(engine.orderbook.best_bid(), Some(dec!(99)));
        let bids_at_99 = engine.orderbook.get_level_mut(dec!(99), Side::Bid).unwrap();
        assert_eq!(bids_at_99[0].id, 4);
        assert_eq!(bids_at_99[0].quantity,dec!(1));
    }

    #[test]
    fn test_single_level_full_fill_low_price_sell_order() {
        let mut engine = MatchingEngine::new(String::from("BTC/USDT"));

        // Add bids 
        engine.orderbook.add_order(Order { id: 1,user_id: 1, quantity: dec!(1), price: dec!(101),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });
        engine.orderbook.add_order(Order { id: 2,user_id: 1, quantity: dec!(1), price: dec!(102),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });
        engine.orderbook.add_order(Order { id: 3,user_id: 1, quantity: dec!(1), price: dec!(103),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });

        // Add bids
        engine.orderbook.add_order(Order { id: 4,user_id: 2, quantity: dec!(6), price: dec!(99),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });
        engine.orderbook.add_order(Order { id: 5,user_id: 2, quantity: dec!(1), price: dec!(98),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });
        engine.orderbook.add_order(Order { id: 6,user_id: 2, quantity: dec!(1), price: dec!(97),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });

        // sent an sell order with quantity 6 and price : 97 (exact match for the best bid)
        engine.process_order(Order { id:7,user_id: 3, quantity: dec!(6), price: dec!(97),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });

        assert_eq!(engine.orderbook.best_bid(), Some(dec!(98)));
        let bids_at_99 = engine.orderbook.get_level_mut(dec!(99), Side::Bid);
        assert!(bids_at_99.is_none(), "Price level $99 should have been deleted from the BTreeMap");
    }
    
    #[test]
    fn test_single_level_partial_fill_low_price_sell_order() {
        let mut engine = MatchingEngine::new(String::from("BTC/USDT"));

        // Add bids 
        engine.orderbook.add_order(Order { id: 1,user_id: 1, quantity: dec!(1), price: dec!(101),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });
        engine.orderbook.add_order(Order { id: 2,user_id: 1, quantity: dec!(1), price: dec!(102),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });
        engine.orderbook.add_order(Order { id: 3,user_id: 1, quantity: dec!(1), price: dec!(103),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });

        // Add bids
        engine.orderbook.add_order(Order { id: 4,user_id: 2, quantity: dec!(6), price: dec!(99),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });
        engine.orderbook.add_order(Order { id: 5,user_id: 2, quantity: dec!(1), price: dec!(98),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });
        engine.orderbook.add_order(Order { id: 6,user_id: 2, quantity: dec!(1), price: dec!(97),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });

        // sent an sell order with quantity 4 and price : 96 (partial match for the best bid)
        engine.process_order(Order { id:7,user_id: 3, quantity: dec!(4), price: dec!(96),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });

        assert_eq!(engine.orderbook.best_bid(), Some(dec!(99)));
        let bids_at_99 = engine.orderbook.get_level_mut(dec!(99), Side::Bid).unwrap();
        assert_eq!(bids_at_99[0].quantity, dec!(2));
    }

    #[test]
    fn test_multi_level_partial_fill_low_price_sell_order() {
        let mut engine = MatchingEngine::new(String::from("BTC/USDT"));

        // Add bids 
        engine.orderbook.add_order(Order { id: 1,user_id: 1, quantity: dec!(1), price: dec!(101),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });
        engine.orderbook.add_order(Order { id: 2,user_id: 1, quantity: dec!(1), price: dec!(102),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });
        engine.orderbook.add_order(Order { id: 3,user_id: 1, quantity: dec!(1), price: dec!(103),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });

        // Add bids
        engine.orderbook.add_order(Order { id: 4,user_id: 2, quantity: dec!(4), price: dec!(99),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });
        engine.orderbook.add_order(Order { id: 5,user_id: 2, quantity: dec!(5), price: dec!(98),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });
        engine.orderbook.add_order(Order { id: 6,user_id: 2, quantity: dec!(1), price: dec!(97),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });

        // sent an sell order with quantity 7 and price : 96 (partial match for the best bid)
        engine.process_order(Order { id:7,user_id: 3, quantity: dec!(7), price: dec!(96),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });

        assert_eq!(engine.orderbook.best_bid(), Some(dec!(98)));
        let bids_at_98 = engine.orderbook.get_level_mut(dec!(98), Side::Bid).unwrap();
        assert_eq!(bids_at_98[0].quantity, dec!(2));
    }

    #[test]
    fn test_multi_level_full_fill_low_price_sell_order() {
        let mut engine = MatchingEngine::new(String::from("BTC/USDT"));

        // Add bids 
        engine.orderbook.add_order(Order { id: 1,user_id: 1, quantity: dec!(1), price: dec!(101),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });
        engine.orderbook.add_order(Order { id: 2,user_id: 1, quantity: dec!(1), price: dec!(102),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });
        engine.orderbook.add_order(Order { id: 3,user_id: 1, quantity: dec!(1), price: dec!(103),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });

        // Add bids
        engine.orderbook.add_order(Order { id: 4,user_id: 2, quantity: dec!(4), price: dec!(99),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });
        engine.orderbook.add_order(Order { id: 5,user_id: 2, quantity: dec!(5), price: dec!(98),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });
        engine.orderbook.add_order(Order { id: 6,user_id: 2, quantity: dec!(1), price: dec!(97),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });

        // sent an sell order with quantity 9 and price : 96 (partial match for the best bid)
        engine.process_order(Order { id:7,user_id: 3, quantity: dec!(9), price: dec!(96),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });

        assert_eq!(engine.orderbook.best_bid(), Some(dec!(97)));
        let bids_at_97 = engine.orderbook.get_level_mut(dec!(97), Side::Bid).unwrap();
        assert_eq!(bids_at_97[0].quantity, dec!(1));
    }

    #[test]
    fn test_whale_activity_eating_entire_bids() {
        let mut engine = MatchingEngine::new(String::from("BTC/USDT"));

        // Add bids 
        engine.orderbook.add_order(Order { id: 1,user_id: 1, quantity: dec!(1), price: dec!(101),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });
        engine.orderbook.add_order(Order { id: 2,user_id: 1, quantity: dec!(1), price: dec!(102),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });
        engine.orderbook.add_order(Order { id: 3,user_id: 1, quantity: dec!(1), price: dec!(103),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });

        // Add bids
        engine.orderbook.add_order(Order { id: 4,user_id: 2, quantity: dec!(4), price: dec!(99),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });
        engine.orderbook.add_order(Order { id: 5,user_id: 2, quantity: dec!(5), price: dec!(98),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });
        engine.orderbook.add_order(Order { id: 6,user_id: 2, quantity: dec!(1), price: dec!(97),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });

        // sent an sell order with quantity 7 and price : 96 (partial match for the best bid)
        engine.process_order(Order { id:7,user_id: 3, quantity: dec!(70), price: dec!(96),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });

        assert_eq!(engine.orderbook.best_ask(), Some(dec!(96)));
        let asks_at_96 = engine.orderbook.get_level_mut(dec!(96), Side::Ask).unwrap();
        assert_eq!(asks_at_96[0].quantity, dec!(60));
    }

    #[test]
    fn test_self_trade_buy_cancel_maker_same_user_id() {
        let mut engine = MatchingEngine::new(String::from("BTC/USDT"));

        // Add bids 
        engine.orderbook.add_order(Order { id: 1,user_id: 1, quantity: dec!(1), price: dec!(101),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });
        engine.orderbook.add_order(Order { id: 2,user_id: 1, quantity: dec!(1), price: dec!(102),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });
        engine.orderbook.add_order(Order { id: 3,user_id: 1, quantity: dec!(1), price: dec!(103),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });

        // Add bids
        engine.orderbook.add_order(Order { id: 4,user_id: 2, quantity: dec!(4), price: dec!(99),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });
        engine.orderbook.add_order(Order { id: 5,user_id: 2, quantity: dec!(5), price: dec!(98),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });
        engine.orderbook.add_order(Order { id: 6,user_id: 2, quantity: dec!(1), price: dec!(97),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });

        // sent an sell order with quantity 7 and price : 96 (partial match for the best bid)
        engine.process_order(Order { id:7,user_id: 1, quantity: dec!(70), price: dec!(96),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });

        assert_eq!(engine.orderbook.best_ask(), Some(dec!(96)));
        let asks_at_96 = engine.orderbook.get_level_mut(dec!(96), Side::Ask).unwrap();
        assert_eq!(asks_at_96[0].quantity, dec!(60));
    }

    #[test]
    fn test_self_trade_buy_cancel_maker_same_different_id() {
        let mut engine = MatchingEngine::new(String::from("BTC/USDT"));

        // Add bids 
        engine.orderbook.add_order(Order { id: 1,user_id: 1, quantity: dec!(1), price: dec!(101),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });
        engine.orderbook.add_order(Order { id: 2,user_id: 2, quantity: dec!(1), price: dec!(102),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });
        engine.orderbook.add_order(Order { id: 3,user_id: 1, quantity: dec!(1), price: dec!(103),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });

        // Add bids
        engine.orderbook.add_order(Order { id: 4,user_id: 2, quantity: dec!(4), price: dec!(99),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });
        engine.orderbook.add_order(Order { id: 5,user_id: 2, quantity: dec!(5), price: dec!(98),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });
        engine.orderbook.add_order(Order { id: 6,user_id: 2, quantity: dec!(1), price: dec!(97),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });

        // sent an buy order with quantity 1 and price : 101 
        engine.process_order(Order { id:7,user_id: 1, quantity: dec!(1), price: dec!(101),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });

        assert_eq!(engine.orderbook.best_ask(), Some(dec!(102)));
        
        let asks_at_103 = engine.orderbook.get_level_mut(dec!(103), Side::Ask).unwrap();
        assert_eq!(asks_at_103[0].quantity, dec!(1));
    }

    #[test]
    fn test_self_trade_buy_cancel_maker_same_different_id_2() {
        let mut engine = MatchingEngine::new(String::from("BTC/USDT"));

        // Add bids 
        engine.orderbook.add_order(Order { id: 1,user_id: 1, quantity: dec!(1), price: dec!(101),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });
        engine.orderbook.add_order(Order { id: 2,user_id: 2, quantity: dec!(1), price: dec!(102),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });
        engine.orderbook.add_order(Order { id: 3,user_id: 1, quantity: dec!(1), price: dec!(103),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });

        // Add bids
        engine.orderbook.add_order(Order { id: 4,user_id: 2, quantity: dec!(4), price: dec!(99),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });
        engine.orderbook.add_order(Order { id: 5,user_id: 2, quantity: dec!(5), price: dec!(98),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });
        engine.orderbook.add_order(Order { id: 6,user_id: 2, quantity: dec!(1), price: dec!(97),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });

        // sent an buy order with quantity 1 and price : 102 
        engine.process_order(Order { id:7,user_id: 1, quantity: dec!(1), price: dec!(102),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });

        assert_eq!(engine.orderbook.best_ask(), Some(dec!(103)));
        
        let asks_at_103 = engine.orderbook.get_level_mut(dec!(103), Side::Ask).unwrap();
        assert_eq!(asks_at_103[0].quantity, dec!(1));
    }

    #[test]
    fn test_self_trade_buy_cancel_maker_same_different_id_3_whale_buy() {
        let mut engine = MatchingEngine::new(String::from("BTC/USDT"));

        // Add bids 
        engine.orderbook.add_order(Order { id: 1,user_id: 1, quantity: dec!(1), price: dec!(101),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });
        engine.orderbook.add_order(Order { id: 2,user_id: 2, quantity: dec!(1), price: dec!(102),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });
        engine.orderbook.add_order(Order { id: 3,user_id: 1, quantity: dec!(1), price: dec!(103),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });

        // Add bids
        engine.orderbook.add_order(Order { id: 4,user_id: 2, quantity: dec!(4), price: dec!(99),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });
        engine.orderbook.add_order(Order { id: 5,user_id: 2, quantity: dec!(5), price: dec!(98),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });
        engine.orderbook.add_order(Order { id: 6,user_id: 2, quantity: dec!(1), price: dec!(97),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });

        // sent an buy order with quantity 1 and price : 102 
        engine.process_order(Order { id:7,user_id: 1, quantity: dec!(30), price: dec!(110),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });

        assert!(engine.orderbook.best_ask().is_none(), "Whale at all the asks now sits at buy side");
        
        let asks_at_110: &mut std::collections::VecDeque<Order> = engine.orderbook.get_level_mut(dec!(110), Side::Bid).unwrap();
        assert_eq!(asks_at_110[0].quantity, dec!(29));
    }


    #[test]
    fn test_self_trade_sell_cancel_maker_same_user_id() {
        let mut engine = MatchingEngine::new(String::from("BTC/USDT"));

        // Add bids 
        engine.orderbook.add_order(Order { id: 1,user_id: 1, quantity: dec!(1), price: dec!(101),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });
        engine.orderbook.add_order(Order { id: 2,user_id: 1, quantity: dec!(1), price: dec!(102),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });
        engine.orderbook.add_order(Order { id: 3,user_id: 1, quantity: dec!(1), price: dec!(103),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });

        // Add bids
        engine.orderbook.add_order(Order { id: 4,user_id: 2, quantity: dec!(4), price: dec!(99),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });
        engine.orderbook.add_order(Order { id: 5,user_id: 2, quantity: dec!(5), price: dec!(98),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });
        engine.orderbook.add_order(Order { id: 6,user_id: 2, quantity: dec!(1), price: dec!(97),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Bid });

        // sent an sell order with quantity 7 and price : 96 (partial match for the best bid)
        engine.process_order(Order { id:7,user_id: 2, quantity: dec!(70), price: dec!(99),action: Action::Create, order_type: OrderType::Limit,client_id: 0, engine_id: 0, side: Side::Ask });

        assert_eq!(engine.orderbook.best_ask(), Some(dec!(99)));
        assert_eq!(engine.orderbook.best_bid(), Some(dec!(98)));
        let bids_at_98 = engine.orderbook.get_level_mut(dec!(98), Side::Bid).unwrap();
        assert_eq!(bids_at_98[0].quantity, dec!(5));

        let asks_at_99 = engine.orderbook.get_level_mut(dec!(99), Side::Ask).unwrap();
        assert_eq!(asks_at_99[0].quantity, dec!(70));
    }
}