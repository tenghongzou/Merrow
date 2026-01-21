use crate::models::{Account, OrderRequest, OrderType, Side};
use crate::{Error, Result};

#[derive(Clone, Debug)]
pub struct RiskLimits {
    pub max_trade_ratio: f64,
    pub min_cash_reserve_ratio: f64,
    pub max_position_value_ratio: f64,
}

impl RiskLimits {
    pub fn validate(&self) -> Result<()> {
        for (name, value) in [
            ("max_trade_ratio", self.max_trade_ratio),
            ("min_cash_reserve_ratio", self.min_cash_reserve_ratio),
            ("max_position_value_ratio", self.max_position_value_ratio),
        ] {
            if !(0.0..=1.0).contains(&value) {
                return Err(Error::new(format!("{name} must be in [0, 1]")));
            }
        }
        Ok(())
    }
}

pub struct RiskManager {
    limits: RiskLimits,
}

impl RiskManager {
    pub fn new(limits: RiskLimits) -> Result<Self> {
        limits.validate()?;
        Ok(Self { limits })
    }

    pub fn check_order(
        &self,
        account: &Account,
        order: &OrderRequest,
        reference_price: f64,
    ) -> Result<()> {
        if order.quantity <= 0.0 {
            return Err(Error::new("order quantity must be positive"));
        }
        if account.cash < 0.0 {
            return Err(Error::new("account cash must be non-negative"));
        }

        let order_price = match order.order_type {
            OrderType::Market => reference_price,
            OrderType::Limit { price } => price,
        };
        if order_price <= 0.0 {
            return Err(Error::new("order price must be positive"));
        }

        let order_value = order_price * order.quantity;
        if matches!(order.side, Side::Sell) {
            let position_qty = account
                .positions
                .iter()
                .find(|pos| pos.symbol == order.symbol)
                .map(|pos| pos.quantity)
                .unwrap_or(0.0);
            if order.quantity > position_qty {
                return Err(Error::new("sell quantity exceeds position"));
            }
        }
        if matches!(order.side, crate::models::Side::Buy) {
            let max_trade_value = account.cash * self.limits.max_trade_ratio;
            if order_value > max_trade_value {
                return Err(Error::new("order exceeds max_trade_ratio"));
            }
            let remaining_cash = account.cash - order_value;
            let min_cash = account.cash * self.limits.min_cash_reserve_ratio;
            if remaining_cash < min_cash {
                return Err(Error::new("order violates min_cash_reserve_ratio"));
            }
            let position_qty = account
                .positions
                .iter()
                .find(|pos| pos.symbol == order.symbol)
                .map(|pos| pos.quantity)
                .unwrap_or(0.0);
            let position_value = position_qty * order_price;
            let new_position_value = position_value + order_value;
            let portfolio_value = account.cash + position_value;
            if portfolio_value > 0.0 {
                let ratio = new_position_value / portfolio_value;
                if ratio > self.limits.max_position_value_ratio {
                    return Err(Error::new("order exceeds max_position_value_ratio"));
                }
            }
        }

        Ok(())
    }
}
