use crate::exchange::Exchange;
use crate::models::{Balance, OrderAck, Position};
use crate::Result;

pub struct AccountSnapshot {
    pub balances: Vec<Balance>,
    pub positions: Vec<Position>,
    pub open_orders: Vec<OrderAck>,
}

pub fn sync_account(exchange: &dyn Exchange) -> Result<AccountSnapshot> {
    let balances = exchange.fetch_balances()?;
    let positions = exchange.fetch_positions()?;
    let open_orders = exchange.fetch_open_orders()?;
    Ok(AccountSnapshot {
        balances,
        positions,
        open_orders,
    })
}
