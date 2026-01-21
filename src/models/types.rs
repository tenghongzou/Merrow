use crate::{Error, Result};

#[derive(Clone, Debug, PartialEq)]
pub struct Candle {
    pub time: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Clone, Debug, PartialEq)]
pub enum OrderType {
    Market,
    Limit { price: f64 },
}

#[derive(Clone, Debug, PartialEq)]
pub struct OrderRequest {
    pub client_order_id: String,
    pub symbol: String,
    pub side: Side,
    pub order_type: OrderType,
    pub quantity: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum OrderStatus {
    New,
    PartiallyFilled,
    Filled,
    Canceled,
    Rejected,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OrderAck {
    pub client_order_id: String,
    pub exchange_order_id: Option<String>,
    pub status: OrderStatus,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Trade {
    pub time: i64,
    pub symbol: String,
    pub side: Side,
    pub price: f64,
    pub quantity: f64,
    pub fee: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Position {
    pub symbol: String,
    pub quantity: f64,
    pub avg_price: f64,
}

impl Position {
    pub fn new(symbol: impl Into<String>, quantity: f64, avg_price: f64) -> Result<Self> {
        if quantity < 0.0 {
            return Err(Error::new("position quantity must be non-negative"));
        }
        if avg_price < 0.0 {
            return Err(Error::new("avg_price must be non-negative"));
        }
        Ok(Self {
            symbol: symbol.into(),
            quantity,
            avg_price,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Balance {
    pub asset: String,
    pub free: f64,
    pub locked: f64,
}

impl Balance {
    pub fn new(asset: impl Into<String>, free: f64, locked: f64) -> Result<Self> {
        if free < 0.0 || locked < 0.0 {
            return Err(Error::new("balance values must be non-negative"));
        }
        Ok(Self {
            asset: asset.into(),
            free,
            locked,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Account {
    pub cash: f64,
    pub positions: Vec<Position>,
}

impl Account {
    pub fn new(cash: f64, positions: Vec<Position>) -> Result<Self> {
        if cash < 0.0 {
            return Err(Error::new("cash must be non-negative"));
        }
        Ok(Self { cash, positions })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Signal {
    Buy,
    Sell,
    Hold,
}
