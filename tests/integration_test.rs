
#[cfg(test)]
mod tests {
    use vanilla_orderbook::prelude::*;
    use vanilla_orderbook::orderbook::OrderBook;

    fn setup() -> OrderBook {
        OrderBook::default()
    }

    #[test]
    fn test_empty_book_placement() {
        // given an empty order book
        let mut book = setup();

        // when a buy order is placed
        let trades = book.place_order(Side::Buy, 100, 10, 1);
        
        // then no trades are returned and the order is on the buy side
        assert!(trades.is_empty());
        assert_eq!(book.best_buy(), Some((100, 10)));
        assert_eq!(book.best_sell(), None);
    }

    #[test]
    fn test_no_match_buy_order() {
        // given a sell order on the book
        let mut book = setup();
        book.place_order(Side::Sell, 105, 5, 101);

        // when a buy order is placed below the sell price
        let trades = book.place_order(Side::Buy, 100, 10, 2);

        // then no match occurs, and both orders are on the book
        assert!(trades.is_empty());
        assert_eq!(book.best_buy(), Some((100, 10)));
        assert_eq!(book.best_sell(), Some((105, 5)));
    }

    #[test]
    fn test_no_match_sell_order() {
        // given a buy order on the book
        let mut book = setup();
        book.place_order(Side::Buy, 95, 5, 101);

        // when a sell order is placed above the buy price
        let trades = book.place_order(Side::Sell, 100, 10, 2);

        // then no match occurs, and both orders are on the book
        assert!(trades.is_empty());
        assert_eq!(book.best_buy(), Some((95, 5)));
        assert_eq!(book.best_sell(), Some((100, 10)));
    }


        #[test]
    fn test_full_taker_fill() {
        // given a sell order of quantity 10
        let mut book = setup();
        book.place_order(Side::Sell, 100, 10, 101);

        // when an incoming buy order of quantity 15 is placed
        let trades = book.place_order(Side::Buy, 105, 15, 2);

        // then the incoming order is partially filled, the resting order is fully filled and removed
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].quantity, 10);
        assert_eq!(trades[0].taker_id, 2);
        assert_eq!(trades[0].maker_id, 101);
        
        // remaining quantity (5) is added to the book
        assert_eq!(book.best_buy(), Some((105, 5)));
        assert_eq!(book.best_sell(), None);
    }

    #[test]
    fn test_full_maker_fill() {
        // given a sell order of quantity 15
        let mut book = setup();
        book.place_order(Side::Sell, 100, 15, 101);

        // when an incoming buy order of quantity 10 is placed
        let trades = book.place_order(Side::Buy, 100, 10, 2);

        // then the resting order is partially filled (5 left) and the incoming order is fully filled
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].quantity, 10);
        
        assert_eq!(book.best_sell(), Some((100, 5)));
        assert_eq!(book.best_buy(), None);
    }


    #[test]
    fn test_partial_fill_buy_matches_one_sell() {
        // given a sell order on the book
        let mut book = setup();
        book.place_order(Side::Sell, 100, 10, 101);

        // when a buy order with smaller quantity is placed
        let trades = book.place_order(Side::Buy, 100, 5, 2);

        // then a single trade occurs, and the resting sell order's quantity is reduced
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].quantity, 5);
        assert_eq!(trades[0].taker_id, 2);
        assert_eq!(trades[0].maker_id, 101);
        
        // The buy side is empty, sell side has the remainder
        assert_eq!(book.best_buy(), None);
        assert_eq!(book.best_sell(), Some((100, 5)));
    }

    #[test]
    fn test_partial_fill_sell_matches_one_buy() {
        // given a buy order on the book
        let mut book = setup();
        book.place_order(Side::Buy, 100, 10, 101);

        // when a sell order with smaller quantity is placed
        let trades = book.place_order(Side::Sell, 100, 5, 2);

        // then a single trade occurs, and the resting buy order's quantity is reduced
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].quantity, 5);
        assert_eq!(trades[0].taker_id, 2);
        assert_eq!(trades[0].maker_id, 101);
        
        // sell side is empty, buy side has the remainder
        assert_eq!(book.best_sell(), None);
        assert_eq!(book.best_buy(), Some((100, 5)));
    }


    #[test]
    fn test_buy_price_priority() {
        // given a book with two sell orders at different prices
        let mut book = setup();
        book.place_order(Side::Sell, 99, 5, 101); // Better price
        book.place_order(Side::Sell, 100, 5, 102);

        // when an incoming buy order is placed that can fill both
        let trades = book.place_order(Side::Buy, 100, 10, 2);

        // then it fills the best price (99) first
        assert_eq!(trades.len(), 2);
        assert_eq!(trades[0].price, 99);
        assert_eq!(trades[0].maker_id, 101);
        assert_eq!(trades[0].quantity, 5);

        // and then the next best price (100)
        assert_eq!(trades[1].price, 100);
        assert_eq!(trades[1].maker_id, 102);
        assert_eq!(trades[1].quantity, 5);

        // book is empty after 
        assert!(book.is_buy_side_empty());
        assert!(book.is_sell_side_empty());
    }

    #[test]
    fn test_sell_price_priority() {
        // given a book with two buy orders at different prices
        let mut book = setup();
        book.place_order(Side::Buy, 101, 5, 101); // Better price
        book.place_order(Side::Buy, 100, 5, 102);

        // when an incoming sell order is placed that can fill both
        let trades = book.place_order(Side::Sell, 100, 10, 2);

        // then it fills the best price (101) first
        assert_eq!(trades.len(), 2);
        assert_eq!(trades[0].price, 101);
        assert_eq!(trades[0].maker_id, 101);
        assert_eq!(trades[0].quantity, 5);

        // and then the next best price (100)
        assert_eq!(trades[1].price, 100);
        assert_eq!(trades[1].maker_id, 102);
        assert_eq!(trades[1].quantity, 5);

        // book is empty after 
        assert!(book.is_buy_side_empty());
        assert!(book.is_sell_side_empty());
    }


    #[test]
    fn test_time_priority_at_same_price() {
        // given two sell orders at the same price, placed at different times
        let mut book = setup();
        book.place_order(Side::Sell, 100, 5, 101); // Oldest order
        book.place_order(Side::Sell, 100, 5, 102); // Newest order

        // when a single incoming buy order of quantity 5 is placed
        let trades = book.place_order(Side::Buy, 100, 5, 2);

        // then it fills the oldest order (ID 101) first
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].maker_id, 101);
        assert_eq!(trades[0].quantity, 5);

        // older order is gone, the newer one remains
        let remaining_orders = book.get_orders(
            &PriceLevel::new(100, Side::Sell)).unwrap();
        assert_eq!(remaining_orders.len(), 1);
        assert_eq!(remaining_orders[0].id, 102);
    }
    

    #[test]
    fn test_large_order_spanning_multiple_levels() {
        // given a book with multiple sell orders at different prices
        let mut book = setup();
        book.place_order(Side::Sell, 100, 5, 101);
        book.place_order(Side::Sell, 101, 10, 102);
        book.place_order(Side::Sell, 102, 15, 103);

        // when a large buy order is placed that consumes all orders
        let trades = book.place_order(Side::Buy, 105, 30, 2);

        // then all resting orders are filled and trades are in price order
        assert_eq!(trades.len(), 3);
        assert_eq!(trades[0].price, 100); // Fills first
        assert_eq!(trades[1].price, 101); // Fills second
        assert_eq!(trades[2].price, 102); // Fills third
        
        assert!(book.is_buy_side_empty()); // Remainder of 30-30=0 is not added
        assert!(book.get_orders(&PriceLevel::new(105, Side::Buy)).is_none());
    }

    #[test]
    fn test_zero_quantity_order() {
        // given an empty book
        let mut book = setup();

        // when an order with zero quantity is placed
        let trades = book.place_order(Side::Buy, 100, 0, 1);

        // then no trades occur and the book remains empty
        assert!(trades.is_empty());
        assert_eq!(book.best_buy(), None);
        assert_eq!(book.best_sell(), None);
    }

    #[test]
    fn test_market_cross() {
        // given a market where bid > ask
        let mut book = setup();
        book.place_order(Side::Sell, 99, 10, 101); // Ask
        book.place_order(Side::Buy, 100, 11, 102); // Bid

        // when a new order is placed (doesn't matter which side)
        let trades = book.place_order(Side::Sell, 98, 10, 2);

        // then it first matches the best bid at 100
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].price, 100);
        assert_eq!(trades[0].maker_id, 102);
        assert_eq!(trades[0].taker_id, 2);
    }
}
