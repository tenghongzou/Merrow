use crate::backtest::BacktestResult;
use crate::config::Config;
use crate::data::csv_loader::parse_time;
use crate::models::{Account, Balance, Candle, OrderStatus, Side, Trade};
use crate::{Error, Result};
use chrono::{DateTime, TimeZone, Utc};
use postgres::{Client, NoTls};
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

const SCHEMA_SQL: &str = include_str!("../../DB_SCHEMA.sql");

pub struct PostgresStorage {
    pub dsn: String,
}

impl PostgresStorage {
    pub fn new(dsn: impl Into<String>) -> Self {
        Self { dsn: dsn.into() }
    }

    pub fn ensure_schema(&self) -> Result<()> {
        let mut client = self.connect()?;
        for statement in split_statements(SCHEMA_SQL) {
            if statement.trim().is_empty() {
                continue;
            }
            client
                .batch_execute(&statement)
                .map_err(|err| Error::new(format!("schema execute failed: {err}")))?;
        }
        Ok(())
    }

    pub fn persist_backtest(
        &self,
        config: &Config,
        candles: &[Candle],
        result: &BacktestResult,
    ) -> Result<String> {
        let mut client = self.connect()?;
        let mut tx = client
            .transaction()
            .map_err(|err| Error::new(format!("db transaction failed: {err}")))?;

        let run_id = generate_run_id();
        let start_time = parse_time(
            config
                .backtest
                .start_time
                .as_deref()
                .ok_or_else(|| Error::new("backtest.start_time must be set"))?,
        )?;
        let end_time = parse_time(
            config
                .backtest
                .end_time
                .as_deref()
                .ok_or_else(|| Error::new("backtest.end_time must be set"))?,
        )?;
        let start_ts = to_timestamp(start_time)?;
        let end_ts = to_timestamp(end_time)?;
        let params = build_params_json(config);

        tx.execute(
            "INSERT INTO backtest_runs (id, start_time, end_time, params) VALUES ($1, $2, $3, $4)",
            &[&run_id, &start_ts, &end_ts, &params],
        )
        .map_err(|err| Error::new(format!("insert backtest_runs failed: {err}")))?;

        tx.execute(
            "INSERT INTO backtest_metrics (run_id, return, max_drawdown, win_rate, trade_count, sharpe) VALUES ($1, $2, $3, $4, $5, $6)",
            &[
                &run_id,
                &result.metrics.return_rate,
                &result.metrics.max_drawdown,
                &result.metrics.win_rate,
                &(result.metrics.trade_count as i32),
                &result.metrics.sharpe,
            ],
        )
        .map_err(|err| Error::new(format!("insert backtest_metrics failed: {err}")))?;

        insert_prices(&mut tx, candles, config)?;
        insert_orders_and_trades(&mut tx, config, &run_id, &result.trades)?;
        insert_positions(&mut tx, config, &result.account, candles)?;
        insert_balances(&mut tx, config, &result.account, candles)?;

        tx.commit()
            .map_err(|err| Error::new(format!("db commit failed: {err}")))?;
        Ok(run_id)
    }

    pub fn persist_paper(
        &self,
        config: &Config,
        candles: &[Candle],
        result: &BacktestResult,
    ) -> Result<()> {
        let mut client = self.connect()?;
        let mut tx = client
            .transaction()
            .map_err(|err| Error::new(format!("db transaction failed: {err}")))?;

        insert_prices(&mut tx, candles, config)?;
        insert_orders_and_trades(&mut tx, config, "paper", &result.trades)?;
        insert_positions(&mut tx, config, &result.account, candles)?;
        insert_balances(&mut tx, config, &result.account, candles)?;

        tx.commit()
            .map_err(|err| Error::new(format!("db commit failed: {err}")))?;
        Ok(())
    }

    fn connect(&self) -> Result<Client> {
        Client::connect(&self.dsn, NoTls)
            .map_err(|err| Error::new(format!("postgres connect failed: {err}")))
    }
}

fn insert_prices(tx: &mut postgres::Transaction<'_>, candles: &[Candle], config: &Config) -> Result<()> {
    if candles.is_empty() {
        return Ok(());
    }
    let stmt = tx
        .prepare(
            "INSERT INTO prices (symbol, interval, time, open, high, low, close, volume) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
             ON CONFLICT DO NOTHING",
        )
        .map_err(|err| Error::new(format!("prepare prices failed: {err}")))?;

    for candle in candles {
        let time = to_timestamp(candle.time)?;
        tx.execute(
            &stmt,
            &[
                &config.symbol,
                &config.data.candle_interval,
                &time,
                &candle.open,
                &candle.high,
                &candle.low,
                &candle.close,
                &candle.volume,
            ],
        )
        .map_err(|err| Error::new(format!("insert prices failed: {err}")))?;
    }
    Ok(())
}

fn insert_orders_and_trades(
    tx: &mut postgres::Transaction<'_>,
    config: &Config,
    run_id: &str,
    trades: &[Trade],
) -> Result<()> {
    if trades.is_empty() {
        return Ok(());
    }

    let order_stmt = tx
        .prepare(
            "INSERT INTO orders (id, time, mode, symbol, side, order_type, price, qty, status, exchange_order_id, client_order_id) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
        )
        .map_err(|err| Error::new(format!("prepare orders failed: {err}")))?;
    let trade_stmt = tx
        .prepare(
            "INSERT INTO trades (id, order_id, time, symbol, side, price, qty, fee, fee_asset) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .map_err(|err| Error::new(format!("prepare trades failed: {err}")))?;

    let mode = config.mode.as_str();
    let order_type = config.orders.order_type.as_str();
    let fee_asset = infer_cash_asset(&config.symbol);

    for (index, trade) in trades.iter().enumerate() {
        let order_id = format!("order-{run_id}-{index}");
        let trade_id = format!("trade-{run_id}-{index}");
        let time = to_timestamp(trade.time)?;
        let side = side_to_str(&trade.side);
        let status = status_to_str(&OrderStatus::Filled);

        tx.execute(
            &order_stmt,
            &[
                &order_id,
                &time,
                &mode,
                &trade.symbol,
                &side,
                &order_type,
                &trade.price,
                &trade.quantity,
                &status,
                &Option::<String>::None,
                &order_id,
            ],
        )
        .map_err(|err| Error::new(format!("insert orders failed: {err}")))?;

        tx.execute(
            &trade_stmt,
            &[
                &trade_id,
                &order_id,
                &time,
                &trade.symbol,
                &side,
                &trade.price,
                &trade.quantity,
                &trade.fee,
                &fee_asset,
            ],
        )
        .map_err(|err| Error::new(format!("insert trades failed: {err}")))?;
    }

    Ok(())
}

fn insert_positions(
    tx: &mut postgres::Transaction<'_>,
    config: &Config,
    account: &Account,
    candles: &[Candle],
) -> Result<()> {
    let snapshot_time = snapshot_time(candles)?;
    let stmt = tx
        .prepare(
            "INSERT INTO positions (time, mode, symbol, qty, avg_price) \
             VALUES ($1, $2, $3, $4, $5) \
             ON CONFLICT (time, mode, symbol) DO UPDATE SET qty = EXCLUDED.qty, avg_price = EXCLUDED.avg_price",
        )
        .map_err(|err| Error::new(format!("prepare positions failed: {err}")))?;

    for position in &account.positions {
        tx.execute(
            &stmt,
            &[
                &snapshot_time,
                &config.mode,
                &position.symbol,
                &position.quantity,
                &position.avg_price,
            ],
        )
        .map_err(|err| Error::new(format!("insert positions failed: {err}")))?;
    }
    Ok(())
}

fn insert_balances(
    tx: &mut postgres::Transaction<'_>,
    config: &Config,
    account: &Account,
    candles: &[Candle],
) -> Result<()> {
    let snapshot_time = snapshot_time(candles)?;
    let cash_asset = infer_cash_asset(&config.symbol);
    let stmt = tx
        .prepare(
            "INSERT INTO balances (time, mode, asset, free, locked) \
             VALUES ($1, $2, $3, $4, $5) \
             ON CONFLICT (time, mode, asset) DO UPDATE SET free = EXCLUDED.free, locked = EXCLUDED.locked",
        )
        .map_err(|err| Error::new(format!("prepare balances failed: {err}")))?;

    let balance = Balance {
        asset: cash_asset.clone(),
        free: account.cash,
        locked: 0.0,
    };
    tx.execute(
        &stmt,
        &[
            &snapshot_time,
            &config.mode,
            &balance.asset,
            &balance.free,
            &balance.locked,
        ],
    )
    .map_err(|err| Error::new(format!("insert balances failed: {err}")))?;
    Ok(())
}

fn build_params_json(config: &Config) -> serde_json::Value {
    json!({
        "mode": config.mode,
        "exchange": config.exchange,
        "symbol": config.symbol,
        "orders": {
            "order_type": config.orders.order_type,
            "limit_price_offset_bps": config.orders.limit_price_offset_bps,
            "fee_rate": config.orders.fee_rate,
            "slippage_bps": config.orders.slippage_bps,
        },
        "triggers": {
            "time_enabled": config.triggers.time_enabled,
            "time_minutes": config.triggers.time_minutes,
            "price_enabled": config.triggers.price_enabled,
            "trigger_mode_any": config.triggers.trigger_mode_any,
            "ma_window": config.triggers.ma_window,
            "buy_threshold": config.triggers.buy_threshold,
            "sell_threshold": config.triggers.sell_threshold,
        },
        "strategy": {
            "buy_cash_ratio": config.strategy.buy_cash_ratio,
            "sell_pos_ratio": config.strategy.sell_pos_ratio,
            "rebuy_cash_ratio": config.strategy.rebuy_cash_ratio,
        },
        "risk": {
            "max_trade_ratio": config.risk.max_trade_ratio,
            "min_cash_reserve_ratio": config.risk.min_cash_reserve_ratio,
            "max_position_value_ratio": config.risk.max_position_value_ratio,
        },
        "data": {
            "source": config.data.source,
            "candle_interval": config.data.candle_interval,
            "exchange_category": config.data.exchange_category,
        }
    })
}

fn split_statements(sql: &str) -> Vec<String> {
    let mut cleaned = String::new();
    for line in sql.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("--") {
            continue;
        }
        cleaned.push_str(line);
        cleaned.push('\n');
    }
    cleaned
        .split(';')
        .map(|statement| statement.trim().to_string())
        .filter(|statement| !statement.is_empty())
        .collect()
}

fn generate_run_id() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    format!("run-{now}")
}

fn to_timestamp(seconds: i64) -> Result<DateTime<Utc>> {
    Utc.timestamp_opt(seconds, 0)
        .single()
        .ok_or_else(|| Error::new("invalid timestamp"))
}

fn snapshot_time(candles: &[Candle]) -> Result<DateTime<Utc>> {
    let last = candles
        .last()
        .ok_or_else(|| Error::new("no candles for snapshot"))?;
    to_timestamp(last.time)
}

fn side_to_str(side: &Side) -> &'static str {
    match side {
        Side::Buy => "buy",
        Side::Sell => "sell",
    }
}

fn status_to_str(status: &OrderStatus) -> &'static str {
    match status {
        OrderStatus::New => "new",
        OrderStatus::PartiallyFilled => "partially_filled",
        OrderStatus::Filled => "filled",
        OrderStatus::Canceled => "canceled",
        OrderStatus::Rejected => "rejected",
    }
}

fn infer_cash_asset(symbol: &str) -> String {
    let candidates = ["USDT", "USDC", "USD", "BUSD", "EUR"];
    if symbol.contains('-') {
        let parts: Vec<&str> = symbol.split('-').collect();
        if parts.len() >= 2 {
            let quote = parts[1];
            if candidates.contains(&quote) {
                return quote.to_string();
            }
            if let Some(last) = parts.last() {
                if candidates.contains(last) {
                    return (*last).to_string();
                }
            }
        }
    }
    for suffix in candidates {
        if symbol.ends_with(suffix) {
            return suffix.to_string();
        }
    }
    "USD".to_string()
}
