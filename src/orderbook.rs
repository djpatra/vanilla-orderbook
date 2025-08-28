use std::{collections::{BTreeMap, VecDeque}, time::{SystemTime, UNIX_EPOCH}};

use crate::prelude::*;

/// An order book that matches buy and sell orders based on price-time priority.
#[derive(Default)]
pub struct OrderBook {
    /// Buy side orders, highest priority first.
    buy_side: BTreeMap<PriceLevel, VecDeque<Order>>,
    /// Sell side orders, lowest priority first.
    sell_side: BTreeMap<PriceLevel, VecDeque<Order>>,
}

impl OrderBook {
    /// Places a new order into the order book.
    ///
    /// - Attempts to match the incoming order against the opposite side of the book.
    /// - Executes trades until the order is either fully matched or no more matches are possible.
    /// - Any unfilled remainder is added to the appropriate side of the book.
    ///
    /// Returns a list of all trades generated from this order.
    pub fn place_order(&mut self, side: Side, price: u64, quantity: u64, id: u64) -> Vec<Trade> {
        let mut trades = Vec::new();

        // current timestamp since epoch
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backward")
            .as_millis();

        let mut incoming_order = Order {
            id,
            quantity,
            price,
            timestamp: now,
        };

        // Match against opposite side
        match side {
            Side::Buy => {
                // Match against best sell orders
                while incoming_order.quantity > 0 {
                    // Get the best sell price level
                    let best_sell_price = self.sell_side.first_key_value()
                        .map(|(k, _)| k.price);
                    
                    if let Some(best_price) = best_sell_price {
                        if price >= best_price {
                            // Can match this buy order with best sell order
                            incoming_order.quantity = self.match_at_price_level(
                                Side::Sell,
                                best_price,
                                incoming_order.quantity,
                                id,
                                &mut trades
                            );
                        } else {
                            break; // No more matches possible
                        }
                    } else {
                        break; // No sell orders to match
                    }
                }
            }
            Side::Sell => {
                // Match against best buy orders
                while incoming_order.quantity > 0 {
                    // Get the best buy price level
                    let best_buy_price = self.buy_side.first_key_value()
                        .map(|(k, _)| k.price);
                    
                    if let Some(best_price) = best_buy_price {
                        if price <= best_price {
                            // Can match this sell order with the best buy order
                            incoming_order.quantity = self.match_at_price_level(
                                Side::Buy,
                                best_price,
                                incoming_order.quantity,
                                id,
                                &mut trades
                            );
                        } else {
                            break; // No more matches possible
                        }
                    } else {
                        break; // No buy orders to match
                    }
                }
            }
        }

        // Add remainder to book if unfilled
        if incoming_order.quantity > 0 {
            self.add_to_book(side, incoming_order);
        }

        trades
    }

    /// Attempts to match a taker order at a given price level.
    ///
    /// - Iterates through maker orders at this price level (FIFO order).
    /// - Executes trades until either the taker is fully filled or no makers remain.
    /// - Removes maker orders that are fully filled.
    ///
    /// Returns the remaining unfilled quantity of the taker order.
    fn match_at_price_level(
        &mut self,
        book_side: Side,
        price: u64,
        mut qty: u64,
        taker_id: u64,
        trades: &mut Vec<Trade>
    ) -> u64 {
        let price_key = PriceLevel { price, side: book_side };
        let book = match book_side {
            Side::Buy => &mut self.buy_side,
            Side::Sell => &mut self.sell_side,
        };

        if let Some(orders) = book.get_mut(&price_key) {
            while qty > 0 && !orders.is_empty() {
                let front_order = orders.front_mut().unwrap();
                
                let trade_qty = qty.min(front_order.quantity);
                
                // Create trade between maker and taker
                trades.push(Trade {
                    price,
                    quantity: trade_qty,
                    maker_id: front_order.id,
                    taker_id,
                });

                // Update quantities
                qty -= trade_qty;
                front_order.quantity -= trade_qty;

                // Remove order if fully filled
                if front_order.quantity == 0 {
                    orders.pop_front();
                }
            }

            // Remove price level if no orders left
            if orders.is_empty() {
                book.remove(&price_key);
            }
        }

        qty
    }

    /// Adds a new order to the order book at the given price level.
    /// Preserves FIFO order at each price level.
    fn add_to_book(&mut self, side: Side, order: Order) {
        let price_key = PriceLevel { price: order.price, side };
        
        match side {
            Side::Buy => {
                self.buy_side
                    .entry(price_key)
                    .or_default()
                    .push_back(order);
            }
            Side::Sell => {
                self.sell_side
                    .entry(price_key)
                    .or_default()
                    .push_back(order);
            }
        }
    }


    /// Returns the best available buy price and total quantity at that price.
    ///
    /// - Best buy = highest bid price.
    /// - If no buy orders exist, returns None.
    pub fn best_buy(&self) -> Option<(u64, u64)> {
        self.buy_side.first_key_value().map(|(k, orders)| {
            let total_qty = orders.iter().map(|o| o.quantity).sum();
            (k.price, total_qty)
        })
    }

    /// Returns the best available sell price and total quantity at that price.
    ///
    /// - Best sell = lowest ask price.
    /// - If no sell orders exist, returns None.
    pub fn best_sell(&self) -> Option<(u64, u64)> {
        self.sell_side.first_key_value().map(|(k, orders)| {
            let total_qty = orders.iter().map(|o| o.quantity).sum();
            (k.price, total_qty)
        })
    }

    /// Returns a reference to the orders at the given price level.
    ///
    /// Useful for inspecting the order book depth at a specific price.
    pub fn get_orders(&self, price_key: &PriceLevel) -> Option<&VecDeque<Order>> {
        match price_key.side {
            Side::Buy => self.buy_side.get(price_key),
            Side::Sell => self.sell_side.get(price_key)
        }
    }

    pub fn is_buy_side_empty(&self) -> bool {
        self.buy_side.is_empty()
    }

    pub fn is_sell_side_empty(&self) -> bool {
        self.sell_side.is_empty()
    }
}
