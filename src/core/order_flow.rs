use crate::config::Config;
use crate::models::{Account, OrderRequest, OrderType, Position, Signal, Side};
use crate::{Error, Result};

use super::{order_builder::OrderBuilder, risk::RiskManager, StrategyContext};

pub struct OrderFlow {
    builder: OrderBuilder,
    risk: RiskManager,
}

impl OrderFlow {
    pub fn new(risk: RiskManager) -> Self {
        Self {
            builder: OrderBuilder::new(),
            risk,
        }
    }

    pub fn plan(
        &mut self,
        signal: Signal,
        ctx: &StrategyContext,
        config: &Config,
    ) -> Result<Vec<OrderRequest>> {
        if ctx.candle.close <= 0.0 {
            return Err(Error::new("reference price must be positive"));
        }
        let orders = self.builder.build_for_signal(signal, ctx, config)?;
        let mut simulated = Account {
            cash: ctx.account.cash,
            positions: ctx.account.positions.clone(),
        };
        for order in &orders {
            self.risk
                .check_order(&simulated, order, ctx.candle.close)?;
            apply_order_to_account(&mut simulated, order, ctx.candle.close)?;
        }
        Ok(orders)
    }
}

fn apply_order_to_account(
    account: &mut Account,
    order: &OrderRequest,
    reference_price: f64,
) -> Result<()> {
    let order_price = match order.order_type {
        OrderType::Market => reference_price,
        OrderType::Limit { price } => price,
    };
    if order_price <= 0.0 {
        return Err(Error::new("order price must be positive"));
    }
    if order.quantity <= 0.0 {
        return Err(Error::new("order quantity must be positive"));
    }

    let order_value = order_price * order.quantity;
    match order.side {
        Side::Buy => {
            account.cash -= order_value;
            let position = account
                .positions
                .iter_mut()
                .find(|pos| pos.symbol == order.symbol);
            match position {
                Some(pos) => {
                    let total_cost = pos.avg_price * pos.quantity + order_value;
                    let new_qty = pos.quantity + order.quantity;
                    pos.quantity = new_qty;
                    pos.avg_price = if new_qty > 0.0 {
                        total_cost / new_qty
                    } else {
                        0.0
                    };
                }
                None => {
                    account.positions.push(Position {
                        symbol: order.symbol.clone(),
                        quantity: order.quantity,
                        avg_price: order_price,
                    });
                }
            }
        }
        Side::Sell => {
            account.cash += order_value;
            let position = account
                .positions
                .iter_mut()
                .find(|pos| pos.symbol == order.symbol)
                .ok_or_else(|| Error::new("sell requires existing position"))?;
            if order.quantity > position.quantity {
                return Err(Error::new("sell quantity exceeds position"));
            }
            position.quantity -= order.quantity;
            if position.quantity == 0.0 {
                position.avg_price = 0.0;
            }
        }
    }

    Ok(())
}
