use crate::config::Config;
use crate::models::{OrderRequest, OrderType, Side, Signal};
use crate::{Error, Result};

use super::StrategyContext;

pub struct OrderBuilder {
    next_id: u64,
}

impl OrderBuilder {
    pub fn new() -> Self {
        Self { next_id: 1 }
    }

    pub fn build_for_signal(
        &mut self,
        signal: Signal,
        ctx: &StrategyContext,
        config: &Config,
    ) -> Result<Vec<OrderRequest>> {
        let price = ctx.candle.close;
        if price <= 0.0 {
            return Err(Error::new("price must be positive"));
        }

        match signal {
            Signal::Hold => Ok(Vec::new()),
            Signal::Buy => {
                let cash = ctx.account.cash;
                if cash <= 0.0 {
                    return Ok(Vec::new());
                }
                let amount = cash * config.strategy.buy_cash_ratio;
                if amount <= 0.0 {
                    return Ok(Vec::new());
                }
                let qty = amount / price;
                Ok(vec![self.new_order(
                    &config.symbol,
                    Side::Buy,
                    price,
                    qty,
                    config,
                )?])
            }
            Signal::Sell => {
                let position_qty = ctx
                    .account
                    .positions
                    .iter()
                    .find(|pos| pos.symbol == config.symbol)
                    .map(|pos| pos.quantity)
                    .unwrap_or(0.0);
                if position_qty <= 0.0 {
                    return Ok(Vec::new());
                }
                let sell_qty = position_qty * config.strategy.sell_pos_ratio;
                if sell_qty <= 0.0 {
                    return Ok(Vec::new());
                }
                let mut orders = Vec::new();
                orders.push(self.new_order(
                    &config.symbol,
                    Side::Sell,
                    price,
                    sell_qty,
                    config,
                )?);

                let rebuy_ratio = config.strategy.rebuy_cash_ratio;
                if rebuy_ratio > 0.0 {
                    let rebuy_qty = sell_qty * rebuy_ratio;
                    if rebuy_qty > 0.0 {
                        orders.push(self.new_order(
                            &config.symbol,
                            Side::Buy,
                            price,
                            rebuy_qty,
                            config,
                        )?);
                    }
                }

                Ok(orders)
            }
        }
    }

    fn new_order(
        &mut self,
        symbol: &str,
        side: Side,
        reference_price: f64,
        quantity: f64,
        config: &Config,
    ) -> Result<OrderRequest> {
        if quantity <= 0.0 {
            return Err(Error::new("order quantity must be positive"));
        }
        let order_type = match config.orders.order_type.as_str() {
            "market" => OrderType::Market,
            "limit" => {
                let offset_bps = config.orders.limit_price_offset_bps as f64 / 10_000.0;
                let limit_price = match side {
                    Side::Buy => reference_price * (1.0 - offset_bps),
                    Side::Sell => reference_price * (1.0 + offset_bps),
                };
                OrderType::Limit { price: limit_price }
            }
            _ => return Err(Error::new("orders.order_type must be market or limit")),
        };

        let client_order_id = format!("order-{}", self.next_id);
        self.next_id += 1;

        Ok(OrderRequest {
            client_order_id,
            symbol: symbol.to_string(),
            side,
            order_type,
            quantity,
        })
    }
}
