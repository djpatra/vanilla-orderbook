use std::cmp::Ordering;

pub mod orderbook;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone)]
pub struct Order {
    pub id: u64,
    pub price: u64,
    pub quantity: u64,
    pub timestamp: u128,
}

#[derive(Debug, Clone)]
pub struct Trade {
    pub price: u64,
    pub quantity: u64,
    pub maker_id: u64,
    pub taker_id: u64,
}

// Custom key for BTreeMap that implements price-time priority
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PriceLevel {
    price: u64,
    side: Side,
}

impl PriceLevel {
    pub fn new(price: u64, side: Side) -> Self {
        Self {
            price,
            side
        }
    }
}

impl PartialOrd for PriceLevel {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PriceLevel {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.side {
            Side::Buy => {
                // For buy orders, higher price is better (reverse order)
                other.price.cmp(&self.price)
            }
            Side::Sell => {
                // For sell orders, lower price is better (normal order)
                self.price.cmp(&other.price)
            }
        }
    }
}


pub mod prelude {
    pub use crate::{Order, Trade, Side, PriceLevel};
}
