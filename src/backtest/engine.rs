use crate::backtest::fill::{fill_limit, fill_market, ExecutionCosts};
use crate::config::Config;
use crate::core::order_flow::OrderFlow;
use crate::core::strategy::Strategy;
use crate::core::triggers::TriggerEngine;
use crate::core::TriggerContext;
use crate::models::{Account, Candle, OrderRequest, OrderType, Position, Trade};
use crate::{Error, Result};

#[derive(Clone)]
pub struct BacktestOrder {
    pub submit_index: usize,
    pub order: OrderRequest,
}

struct PendingOrder {
    ready_index: usize,
    order: OrderRequest,
}

pub struct BacktestEngine;

pub struct BacktestResult {
    pub trades: Vec<Trade>,
    pub account: Account,
    pub metrics: BacktestMetrics,
    pub equity_curve: Vec<EquityPoint>,
    pub trade_pnls: Vec<Option<f64>>,
}

#[derive(Clone, Debug)]
pub struct BacktestMetrics {
    pub return_rate: f64,
    pub max_drawdown: f64,
    pub win_rate: f64,
    pub trade_count: usize,
    pub sharpe: f64,
}

#[derive(Clone, Debug)]
pub struct EquityPoint {
    pub time: i64,
    pub equity: f64,
}

impl BacktestEngine {
    pub fn run(&self, candles: &[Candle], mut orders: Vec<BacktestOrder>) -> Result<Vec<Trade>> {
        if candles.is_empty() {
            return Ok(Vec::new());
        }

        for order in &orders {
            if order.submit_index >= candles.len() {
                return Err(Error::new("order submit_index out of range"));
            }
        }

        orders.sort_by_key(|order| order.submit_index);

        let mut pending: Vec<PendingOrder> = Vec::new();
        let mut trades: Vec<Trade> = Vec::new();
        let mut order_cursor = 0;

        let costs = ExecutionCosts::zero();

        for (index, candle) in candles.iter().enumerate() {
            while order_cursor < orders.len() && orders[order_cursor].submit_index == index {
                let order = orders[order_cursor].order.clone();
                let ready_index = index.saturating_add(1);
                if ready_index < candles.len() {
                    pending.push(PendingOrder { ready_index, order });
                }
                order_cursor += 1;
            }

            if pending.is_empty() {
                continue;
            }

            let mut next_pending: Vec<PendingOrder> = Vec::new();
            for pending_order in pending {
                if pending_order.ready_index > index {
                    next_pending.push(pending_order);
                    continue;
                }

                let trade = match pending_order.order.order_type {
                    OrderType::Market => fill_market(&pending_order.order, candle, costs),
                    OrderType::Limit { .. } => fill_limit(&pending_order.order, candle, costs),
                };

                match trade {
                    Some(trade) => trades.push(trade),
                    None => {
                        if matches!(pending_order.order.order_type, OrderType::Limit { .. }) {
                            next_pending.push(pending_order);
                        }
                    }
                }
            }

            pending = next_pending;
        }

        Ok(trades)
    }

    pub fn run_strategy(
        &self,
        candles: &[Candle],
        config: &Config,
        trigger_engine: &TriggerEngine,
        strategy: &mut dyn Strategy,
        order_flow: &mut OrderFlow,
        starting_cash: f64,
    ) -> Result<BacktestResult> {
        if starting_cash < 0.0 {
            return Err(Error::new("starting_cash must be non-negative"));
        }
        let account = Account {
            cash: starting_cash,
            positions: Vec::new(),
        };
        self.run_strategy_with_account(
            candles,
            config,
            trigger_engine,
            strategy,
            order_flow,
            account,
        )
    }

    pub fn run_strategy_with_account(
        &self,
        candles: &[Candle],
        config: &Config,
        trigger_engine: &TriggerEngine,
        strategy: &mut dyn Strategy,
        order_flow: &mut OrderFlow,
        mut account: Account,
    ) -> Result<BacktestResult> {
        if account.cash < 0.0 {
            return Err(Error::new("starting_cash must be non-negative"));
        }
        if candles.is_empty() {
            return Ok(BacktestResult {
                trades: Vec::new(),
                account,
                metrics: BacktestMetrics {
                    return_rate: 0.0,
                    max_drawdown: 0.0,
                    win_rate: 0.0,
                    trade_count: 0,
                    sharpe: 0.0,
                },
                equity_curve: Vec::new(),
                trade_pnls: Vec::new(),
            });
        }

        for position in &account.positions {
            if position.quantity < 0.0 {
                return Err(Error::new("position quantity must be non-negative"));
            }
        }

        let mut pending: Vec<PendingOrder> = Vec::new();
        let mut trades: Vec<Trade> = Vec::new();
        let costs = ExecutionCosts {
            fee_rate: config.orders.fee_rate,
            slippage_bps: config.orders.slippage_bps,
        };
        let mut equity_curve: Vec<EquityPoint> = Vec::new();
        let mut trade_pnls: Vec<Option<f64>> = Vec::new();
        let mut win_count = 0usize;
        let mut loss_count = 0usize;
        let starting_equity = account.cash
            + account
                .positions
                .iter()
                .filter(|pos| pos.symbol == config.symbol)
                .map(|pos| pos.quantity * candles[0].close)
                .sum::<f64>();

        for (index, candle) in candles.iter().enumerate() {
            let history = &candles[..=index];
            let trigger_ctx = TriggerContext {
                candle,
                history,
                now: candle.time,
            };

            if trigger_engine.should_fire(&trigger_ctx) {
                let strategy_ctx = crate::core::StrategyContext {
                    candle,
                    history,
                    account: &account,
                    now: candle.time,
                };
                let signals = strategy.on_tick(&strategy_ctx);
                for signal in signals {
                    let orders = order_flow.plan(signal, &strategy_ctx, config)?;
                    for order in orders {
                        let ready_index = index.saturating_add(1);
                        if ready_index < candles.len() {
                            pending.push(PendingOrder { ready_index, order });
                        }
                    }
                }
            }

            if pending.is_empty() {
                continue;
            }

            let mut next_pending: Vec<PendingOrder> = Vec::new();
            for pending_order in pending {
                if pending_order.ready_index > index {
                    next_pending.push(pending_order);
                    continue;
                }

                let trade = match pending_order.order.order_type {
                    OrderType::Market => fill_market(&pending_order.order, candle, costs),
                    OrderType::Limit { .. } => fill_limit(&pending_order.order, candle, costs),
                };

                match trade {
                    Some(trade) => {
                        if let Some(pnl) = apply_trade(&mut account, &trade)? {
                            if pnl > 0.0 {
                                win_count += 1;
                            } else if pnl < 0.0 {
                                loss_count += 1;
                            }
                            trade_pnls.push(Some(pnl));
                        } else {
                            trade_pnls.push(None);
                        }
                        trades.push(trade);
                    }
                    None => {
                        if matches!(pending_order.order.order_type, OrderType::Limit { .. }) {
                            next_pending.push(pending_order);
                        }
                    }
                }
            }

            pending = next_pending;

            let equity = account.cash
                + account
                    .positions
                    .iter()
                    .filter(|pos| pos.symbol == config.symbol)
                    .map(|pos| pos.quantity * candle.close)
                    .sum::<f64>();
            equity_curve.push(EquityPoint {
                time: candle.time,
                equity,
            });
        }

        let metrics = compute_metrics(
            starting_equity,
            &equity_curve,
            trades.len(),
            win_count,
            loss_count,
        );
        Ok(BacktestResult {
            trades,
            account,
            metrics,
            equity_curve,
            trade_pnls,
        })
    }
}

fn apply_trade(account: &mut Account, trade: &Trade) -> Result<Option<f64>> {
    let trade_value = trade.price * trade.quantity;
    let mut realized_pnl = None;
    match trade.side {
        crate::models::Side::Buy => {
            account.cash -= trade_value + trade.fee;
            let position = account
                .positions
                .iter_mut()
                .find(|pos| pos.symbol == trade.symbol);
            match position {
                Some(pos) => {
                    let total_cost = pos.avg_price * pos.quantity + trade_value;
                    let new_qty = pos.quantity + trade.quantity;
                    pos.quantity = new_qty;
                    pos.avg_price = if new_qty > 0.0 {
                        total_cost / new_qty
                    } else {
                        0.0
                    };
                }
                None => {
                    account.positions.push(Position {
                        symbol: trade.symbol.clone(),
                        quantity: trade.quantity,
                        avg_price: trade.price,
                    });
                }
            }
        }
        crate::models::Side::Sell => {
            account.cash += trade_value - trade.fee;
            let position = account
                .positions
                .iter_mut()
                .find(|pos| pos.symbol == trade.symbol)
                .ok_or_else(|| Error::new("trade sell requires existing position"))?;
            if trade.quantity > position.quantity {
                return Err(Error::new("trade sell exceeds position"));
            }
            realized_pnl = Some((trade.price - position.avg_price) * trade.quantity);
            position.quantity -= trade.quantity;
            if position.quantity == 0.0 {
                position.avg_price = 0.0;
            }
        }
    }
    Ok(realized_pnl)
}

fn compute_metrics(
    starting_cash: f64,
    equity_curve: &[EquityPoint],
    trade_count: usize,
    win_count: usize,
    loss_count: usize,
) -> BacktestMetrics {
    let return_rate = if equity_curve.is_empty() || starting_cash <= 0.0 {
        0.0
    } else {
        let last = equity_curve.last().map(|point| point.equity).unwrap_or(starting_cash);
        (last - starting_cash) / starting_cash
    };

    let max_drawdown = max_drawdown(equity_curve);
    let win_rate = if win_count + loss_count == 0 {
        0.0
    } else {
        win_count as f64 / (win_count + loss_count) as f64
    };
    let sharpe = sharpe_ratio(equity_curve);

    BacktestMetrics {
        return_rate,
        max_drawdown,
        win_rate,
        trade_count,
        sharpe,
    }
}

fn max_drawdown(equity_curve: &[EquityPoint]) -> f64 {
    if equity_curve.is_empty() {
        return 0.0;
    }
    let mut peak = equity_curve[0].equity;
    let mut max_dd = 0.0;
    for point in equity_curve {
        let equity = point.equity;
        if equity > peak {
            peak = equity;
        }
        if peak > 0.0 {
            let dd = (peak - equity) / peak;
            if dd > max_dd {
                max_dd = dd;
            }
        }
    }
    max_dd
}

fn sharpe_ratio(equity_curve: &[EquityPoint]) -> f64 {
    if equity_curve.len() < 2 {
        return 0.0;
    }
    let mut returns: Vec<f64> = Vec::new();
    for i in 1..equity_curve.len() {
        let prev = equity_curve[i - 1].equity;
        let curr = equity_curve[i].equity;
        if prev > 0.0 {
            returns.push((curr - prev) / prev);
        }
    }
    if returns.len() < 2 {
        return 0.0;
    }
    let mean = returns.iter().sum::<f64>() / returns.len() as f64;
    let variance = returns
        .iter()
        .map(|r| {
            let diff = r - mean;
            diff * diff
        })
        .sum::<f64>()
        / returns.len() as f64;
    let std = variance.sqrt();
    if std == 0.0 {
        0.0
    } else {
        mean / std * (returns.len() as f64).sqrt()
    }
}
