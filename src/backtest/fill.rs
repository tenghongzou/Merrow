use crate::models::{Candle, OrderRequest, OrderType, Side, Trade};

#[derive(Clone, Copy, Debug)]
pub struct ExecutionCosts {
    pub fee_rate: f64,
    pub slippage_bps: u32,
}

impl ExecutionCosts {
    pub fn zero() -> Self {
        Self {
            fee_rate: 0.0,
            slippage_bps: 0,
        }
    }
}

fn apply_slippage(price: f64, side: Side, slippage_bps: u32) -> f64 {
    if slippage_bps == 0 {
        return price;
    }
    let factor = slippage_bps as f64 / 10_000.0;
    match side {
        Side::Buy => price * (1.0 + factor),
        Side::Sell => price * (1.0 - factor),
    }
}

pub fn fill_market(order: &OrderRequest, candle: &Candle, costs: ExecutionCosts) -> Option<Trade> {
    let price = apply_slippage(candle.open, order.side.clone(), costs.slippage_bps);
    let fee = price * order.quantity * costs.fee_rate;
    Some(Trade {
        time: candle.time,
        symbol: order.symbol.clone(),
        side: order.side.clone(),
        price,
        quantity: order.quantity,
        fee,
    })
}

pub fn fill_limit(order: &OrderRequest, candle: &Candle, costs: ExecutionCosts) -> Option<Trade> {
    let limit_price = match order.order_type {
        OrderType::Limit { price } => price,
        _ => return None,
    };

    let should_fill = match order.side {
        crate::models::Side::Buy => candle.low <= limit_price,
        crate::models::Side::Sell => candle.high >= limit_price,
    };

    if !should_fill {
        return None;
    }

    let fee = limit_price * order.quantity * costs.fee_rate;
    Some(Trade {
        time: candle.time,
        symbol: order.symbol.clone(),
        side: order.side.clone(),
        price: limit_price,
        quantity: order.quantity,
        fee,
    })
}
