use crate::backtest::{BacktestEngine, BacktestResult};
use crate::config::Config;
use crate::core::build_engine_bundle;
use crate::models::{Account, Candle, Position};
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn run_paper(candles: &[Candle], config: &Config) -> Result<BacktestResult> {
    if candles.is_empty() {
        return Err(Error::new("paper mode requires candle data"));
    }

    let mut bundle = build_engine_bundle(config)?;
    let engine = BacktestEngine;
    engine.run_strategy(
        candles,
        config,
        &bundle.trigger_engine,
        &mut bundle.strategy,
        &mut bundle.order_flow,
        config.backtest.initial_cash,
    )
}

pub fn run_paper_with_state(
    candles: &[Candle],
    config: &Config,
    state_path: &str,
) -> Result<BacktestResult> {
    if candles.is_empty() {
        return Err(Error::new("paper mode requires candle data"));
    }

    let starting_account = match load_state(state_path)? {
        Some(state) => state.to_account()?,
        None => Account {
            cash: config.backtest.initial_cash,
            positions: Vec::new(),
        },
    };

    let mut bundle = build_engine_bundle(config)?;
    let engine = BacktestEngine;
    let result = engine.run_strategy_with_account(
        candles,
        config,
        &bundle.trigger_engine,
        &mut bundle.strategy,
        &mut bundle.order_flow,
        starting_account,
    )?;

    save_state(state_path, &result.account)?;
    Ok(result)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PaperPosition {
    symbol: String,
    quantity: f64,
    avg_price: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PaperAccount {
    cash: f64,
    positions: Vec<PaperPosition>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PaperState {
    account: PaperAccount,
    updated_at: i64,
}

impl PaperState {
    fn to_account(&self) -> Result<Account> {
        let mut positions = Vec::new();
        for pos in &self.account.positions {
            positions.push(Position::new(&pos.symbol, pos.quantity, pos.avg_price)?);
        }
        Account::new(self.account.cash, positions)
    }

    fn from_account(account: &Account) -> Self {
        let positions = account
            .positions
            .iter()
            .map(|pos| PaperPosition {
                symbol: pos.symbol.clone(),
                quantity: pos.quantity,
                avg_price: pos.avg_price,
            })
            .collect::<Vec<_>>();
        let account = PaperAccount {
            cash: account.cash,
            positions,
        };
        PaperState {
            account,
            updated_at: now_epoch(),
        }
    }
}

fn load_state(path: &str) -> Result<Option<PaperState>> {
    let path = Path::new(path);
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(path)
        .map_err(|err| Error::new(format!("paper state read failed: {err}")))?;
    let state = serde_json::from_str::<PaperState>(&content)
        .map_err(|err| Error::new(format!("paper state parse failed: {err}")))?;
    Ok(Some(state))
}

fn save_state(path: &str, account: &Account) -> Result<()> {
    let state = PaperState::from_account(account);
    let content = serde_json::to_string_pretty(&state)
        .map_err(|err| Error::new(format!("paper state serialize failed: {err}")))?;
    let path = Path::new(path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| Error::new(format!("paper state dir create failed: {err}")))?;
    }
    fs::write(path, content).map_err(|err| Error::new(format!("paper state write failed: {err}")))
}

fn now_epoch() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}
