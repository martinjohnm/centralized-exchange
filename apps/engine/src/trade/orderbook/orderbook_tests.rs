
use super::*; // Access private logic in engine.rs
use rust_decimal_macros::dec;



#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use super::*;
    
    #[test]
    fn test_orderbook_sorting() {
        let mut book = Orderbook::new();

        // Add bids out of order
        book.add_order(Order { id: 1,user_id : 2, quantity: dec!(1), price: dec!(100), side: Side::Bid, action: crate::trade::model::Action::Create, order_type: crate::trade::model::OrderType::Limit });
        book.add_order(Order { id: 2,user_id : 2, quantity: dec!(1), price: dec!(150), side: Side::Bid, action: crate::trade::model::Action::Create, order_type: crate::trade::model::OrderType::Limit });
        book.add_order(Order { id: 3,user_id : 2, quantity: dec!(1), price: dec!(120), side: Side::Bid, action: crate::trade::model::Action::Create, order_type: crate::trade::model::OrderType::Limit });
        
        // Best bid should be highest (150)
        assert_eq!(book.best_bid(), Some(dec!(150)));

        // Add asks out of order 
        book.add_order(Order { id: 1,user_id : 2, quantity: dec!(1), price: dec!(300), side: Side::Ask, action: crate::trade::model::Action::Create, order_type: crate::trade::model::OrderType::Limit });
        book.add_order(Order { id: 2,user_id : 2, quantity: dec!(1), price: dec!(290), side: Side::Ask, action: crate::trade::model::Action::Create, order_type: crate::trade::model::OrderType::Limit });
        book.add_order(Order { id: 3,user_id : 2, quantity: dec!(1), price: dec!(450), side: Side::Ask, action: crate::trade::model::Action::Create, order_type: crate::trade::model::OrderType::Limit });

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
        book.add_order(Order { id: 1,user_id : 2, quantity: dec!(1), price: dec!(100), side: Side::Bid, action: crate::trade::model::Action::Create, order_type: crate::trade::model::OrderType::Limit });

        // Add second bid at 100
        book.add_order(Order { id: 2,user_id : 2, quantity: dec!(1), price: dec!(100), side: Side::Bid, action: crate::trade::model::Action::Create, order_type: crate::trade::model::OrderType::Limit });
        
        // Add third bid at 101
        book.add_order(Order { id: 3,user_id : 2, quantity: dec!(1), price: dec!(101), side: Side::Bid, action: crate::trade::model::Action::Create, order_type: crate::trade::model::OrderType::Limit });

        // Best bid must be 101
        assert_eq!(book.best_bid(), Some(dec!(101)));

        // Now check the internal structure for the 100 price
        let bids_at_100 = book.bids.get(&dec!(100)).unwrap();
        assert_eq!(bids_at_100[0].id, 1);
        assert_eq!(bids_at_100[1].id, 2);


        // Add first ask
        book.add_order(Order { id: 4,user_id : 2, quantity: dec!(1), price: dec!(110), side: Side::Ask, action: crate::trade::model::Action::Create, order_type: crate::trade::model::OrderType::Limit });

        // Add second ask
        book.add_order(Order { id: 5,user_id : 2, quantity: dec!(1), price: dec!(108), side: Side::Ask, action: crate::trade::model::Action::Create, order_type: crate::trade::model::OrderType::Limit });

        // Add third ask
        book.add_order(Order { id: 6,user_id : 2, quantity: dec!(1), price: dec!(110), side: Side::Ask, action: crate::trade::model::Action::Create, order_type: crate::trade::model::OrderType::Limit });

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

    #[test]
    fn test_get_level_orders () {
        let mut book = Orderbook::new();

        // Add first bid at 100
        book.add_order(Order { id: 1,user_id : 2, quantity: dec!(1), price: dec!(100), side: Side::Bid, action: crate::trade::model::Action::Create, order_type: crate::trade::model::OrderType::Limit });

        // Add second bid at 100
        book.add_order(Order { id: 2,user_id : 2, quantity: dec!(1), price: dec!(100), side: Side::Bid, action: crate::trade::model::Action::Create, order_type: crate::trade::model::OrderType::Limit });
        
        // Add third bid at 101
        book.add_order(Order { id: 3,user_id : 2, quantity: dec!(1), price: dec!(101), side: Side::Bid, action: crate::trade::model::Action::Create, order_type: crate::trade::model::OrderType::Limit });

        // Add first ask
        book.add_order(Order { id: 4,user_id : 2, quantity: dec!(1), price: dec!(110), side: Side::Ask, action: crate::trade::model::Action::Create, order_type: crate::trade::model::OrderType::Limit });

        // Add second ask
        book.add_order(Order { id: 5,user_id : 2, quantity: dec!(1), price: dec!(108), side: Side::Ask, action: crate::trade::model::Action::Create, order_type: crate::trade::model::OrderType::Limit });

        // Add third ask
        book.add_order(Order { id: 6,user_id : 2, quantity: dec!(1), price: dec!(110), side: Side::Ask, action: crate::trade::model::Action::Create, order_type: crate::trade::model::OrderType::Limit });


        let bids_at_100 = book.get_level_mut(dec!(100), Side::Bid).unwrap();
        assert_eq!(bids_at_100[0].id, 1);
        assert_eq!(bids_at_100[1].id, 2);

        let asks_at_100 = book.get_level_mut(dec!(110), Side::Ask).unwrap();
        assert_eq!(asks_at_100[0].id, 4);
        assert_eq!(asks_at_100[1].id, 6);

    }

    #[test]
    fn test_remove_level_orders() {
        let mut book = Orderbook::new();

        // Add first bid at 100
        book.add_order(Order { id: 1,user_id : 2, quantity: dec!(1), price: dec!(100), side: Side::Bid, action: crate::trade::model::Action::Create, order_type: crate::trade::model::OrderType::Limit });

        // Add second bid at 100
        book.add_order(Order { id: 2,user_id : 2, quantity: dec!(1), price: dec!(100), side: Side::Bid, action: crate::trade::model::Action::Create, order_type: crate::trade::model::OrderType::Limit });
        
        // Add third bid at 101
        book.add_order(Order { id: 3,user_id : 2, quantity: dec!(1), price: dec!(101), side: Side::Bid, action: crate::trade::model::Action::Create, order_type: crate::trade::model::OrderType::Limit });

        // Add first ask
        book.add_order(Order { id: 4,user_id : 2, quantity: dec!(1), price: dec!(110), side: Side::Ask, action: crate::trade::model::Action::Create, order_type: crate::trade::model::OrderType::Limit });

        // Add second ask
        book.add_order(Order { id: 5,user_id : 2, quantity: dec!(1), price: dec!(108), side: Side::Ask, action: crate::trade::model::Action::Create, order_type: crate::trade::model::OrderType::Limit });

        // Add third ask
        book.add_order(Order { id: 6,user_id : 2, quantity: dec!(1), price: dec!(110), side: Side::Ask, action: crate::trade::model::Action::Create, order_type: crate::trade::model::OrderType::Limit });


        book.remove_level(dec!(100), Side::Bid);

        let bids_at_100 =  book.get_level_mut(dec!(100), Side::Bid);
        assert!(bids_at_100.is_none(), "Price level $100 should have been deleted from the BTreeMap");
    }

    #[test]
    fn test_cancel_order() {
        let mut book = Orderbook::new();

        // Add first bid at 100
        book.add_order(Order { id: 1,user_id : 2, quantity: dec!(1), price: dec!(100), side: Side::Bid, action: crate::trade::model::Action::Create, order_type: crate::trade::model::OrderType::Limit });

        // Add second bid at 100
        book.add_order(Order { id: 2,user_id : 2, quantity: dec!(1), price: dec!(100), side: Side::Bid, action: crate::trade::model::Action::Create, order_type: crate::trade::model::OrderType::Limit });
        
        // Add third bid at 101
        book.add_order(Order { id: 3,user_id : 2, quantity: dec!(1), price: dec!(101), side: Side::Bid, action: crate::trade::model::Action::Create, order_type: crate::trade::model::OrderType::Limit });

        // Add first ask
        book.add_order(Order { id: 4,user_id : 2, quantity: dec!(1), price: dec!(110), side: Side::Ask, action: crate::trade::model::Action::Create, order_type: crate::trade::model::OrderType::Limit });

        // Add second ask
        book.add_order(Order { id: 5,user_id : 2, quantity: dec!(1), price: dec!(108), side: Side::Ask, action: crate::trade::model::Action::Create, order_type: crate::trade::model::OrderType::Limit });

        // Add third ask
        book.add_order(Order { id: 6,user_id : 2, quantity: dec!(1), price: dec!(110), side: Side::Ask, action: crate::trade::model::Action::Create, order_type: crate::trade::model::OrderType::Limit });

        let id_ref: &u64 = &1;
        assert_eq!(book.orders_lookup.get(id_ref).unwrap(), &OrderMetadata {
            price : dec!(100),
            side : Side::Bid
        });

        // cancel an order at the level 100 which has 2 orders now
        book.cancel_order(*id_ref);
        let canceled_order = book.orders_lookup.get(id_ref);
        assert!(canceled_order.is_none());
        // make sure the len of the price vec is 1 now
        assert_eq!(book.get_level_mut(dec!(100), Side::Bid).unwrap().len(), 1);
        
        // cancel the second order
        let id_ref: &u64 = &2;
        book.cancel_order(*id_ref);
        // make sure the vec is removed
        assert!(book.get_level_mut(dec!(100), Side::Bid).is_none());

    }
}