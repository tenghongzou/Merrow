pub mod adapter;
pub mod binance;
pub mod bybit;
pub mod okx;
pub mod rest;
pub mod sync;

use crate::models::{Balance, Candle, OrderAck, OrderRequest, OrderStatus, Position};
use crate::Result;

pub struct CandleRequest {
    pub symbol: String,
    pub interval: String,
    pub start_time: i64,
    pub end_time: i64,
}

pub trait Exchange {
    fn place_order(&self, order: &OrderRequest) -> Result<OrderAck>;
    fn cancel_order(&self, order_id: &str) -> Result<()>;
    fn fetch_balances(&self) -> Result<Vec<Balance>>;
    fn fetch_positions(&self) -> Result<Vec<Position>>;
    fn fetch_open_orders(&self) -> Result<Vec<OrderAck>>;
    fn fetch_candles(&self, req: &CandleRequest) -> Result<Vec<Candle>>;
}

pub fn new_order_ack(order: &OrderRequest) -> OrderAck {
    OrderAck {
        client_order_id: order.client_order_id.clone(),
        exchange_order_id: None,
        status: OrderStatus::New,
    }
}
