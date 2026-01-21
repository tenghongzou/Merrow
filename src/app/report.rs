use crate::backtest::{BacktestMetrics, BacktestResult, EquityPoint};
use crate::{Error, Result};
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Serialize)]
struct TradeReport {
    time: i64,
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
    fee: f64,
    pnl: Option<f64>,
}

#[derive(Serialize)]
struct MetricsReport {
    return_rate: f64,
    max_drawdown: f64,
    win_rate: f64,
    trade_count: usize,
    sharpe: f64,
}

#[derive(Serialize)]
struct BacktestReport {
    metrics: MetricsReport,
    ending_cash: f64,
    trades: Vec<TradeReport>,
    equity_curve: Vec<EquityReport>,
    costs: CostsReport,
}

#[derive(Serialize)]
struct EquityReport {
    time: i64,
    equity: f64,
}

#[derive(Serialize)]
struct CostsReport {
    total_fees: f64,
    average_fee: f64,
}

pub fn write_output(path: &str, format: &str, result: &BacktestResult) -> Result<()> {
    match format {
        "json" => write_json(path, result),
        "csv" => write_csv(path, result),
        "none" => Ok(()),
        _ => Err(Error::new("output.format must be none, json, or csv")),
    }
}

fn write_json(path: &str, result: &BacktestResult) -> Result<()> {
    ensure_parent_dir(path)?;
    let report = build_report(result);
    let payload = serde_json::to_string_pretty(&report)
        .map_err(|err| Error::new(format!("json serialization failed: {err}")))?;
    fs::write(path, payload).map_err(|err| Error::new(format!("write failed: {err}")))?;
    Ok(())
}

fn write_csv(path: &str, result: &BacktestResult) -> Result<()> {
    ensure_parent_dir(path)?;
    let mut writer = csv::Writer::from_path(path)
        .map_err(|err| Error::new(format!("csv open failed: {err}")))?;
    for (index, trade) in result.trades.iter().enumerate() {
        let row = TradeReport {
            time: trade.time,
            symbol: trade.symbol.clone(),
            side: side_label(&trade.side),
            price: trade.price,
            quantity: trade.quantity,
            fee: trade.fee,
            pnl: result.trade_pnls.get(index).copied().flatten(),
        };
        writer
            .serialize(row)
            .map_err(|err| Error::new(format!("csv write failed: {err}")))?;
    }
    writer
        .flush()
        .map_err(|err| Error::new(format!("csv flush failed: {err}")))?;
    Ok(())
}

fn build_report(result: &BacktestResult) -> BacktestReport {
    let total_fees = result.trades.iter().map(|trade| trade.fee).sum::<f64>();
    let average_fee = if result.trades.is_empty() {
        0.0
    } else {
        total_fees / result.trades.len() as f64
    };

    BacktestReport {
        metrics: to_metrics_report(&result.metrics),
        ending_cash: result.account.cash,
        trades: result
            .trades
            .iter()
            .enumerate()
            .map(|(index, trade)| TradeReport {
                time: trade.time,
                symbol: trade.symbol.clone(),
                side: side_label(&trade.side),
                price: trade.price,
                quantity: trade.quantity,
                fee: trade.fee,
                pnl: result.trade_pnls.get(index).copied().flatten(),
            })
            .collect(),
        equity_curve: to_equity_report(&result.equity_curve),
        costs: CostsReport {
            total_fees,
            average_fee,
        },
    }
}

fn to_metrics_report(metrics: &BacktestMetrics) -> MetricsReport {
    MetricsReport {
        return_rate: metrics.return_rate,
        max_drawdown: metrics.max_drawdown,
        win_rate: metrics.win_rate,
        trade_count: metrics.trade_count,
        sharpe: metrics.sharpe,
    }
}

fn to_equity_report(curve: &[EquityPoint]) -> Vec<EquityReport> {
    curve
        .iter()
        .map(|point| EquityReport {
            time: point.time,
            equity: point.equity,
        })
        .collect()
}

fn side_label(side: &crate::models::Side) -> String {
    match side {
        crate::models::Side::Buy => "buy".to_string(),
        crate::models::Side::Sell => "sell".to_string(),
    }
}

fn ensure_parent_dir(path: &str) -> Result<()> {
    let parent = Path::new(path).parent();
    if let Some(parent) = parent {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .map_err(|err| Error::new(format!("create dir failed: {err}")))?;
        }
    }
    Ok(())
}
