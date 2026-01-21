use merrow::app::report::write_output;
use merrow::backtest::{BacktestMetrics, BacktestResult, EquityPoint};
use merrow::models::{Account, Trade};
use std::env;
use std::fs;
use std::path::PathBuf;

fn temp_path(name: &str) -> PathBuf {
    let mut path = env::temp_dir();
    path.push(name);
    path
}

fn sample_result() -> BacktestResult {
    BacktestResult {
        trades: vec![Trade {
            time: 1,
            symbol: "BTCUSDT".to_string(),
            side: merrow::models::Side::Buy,
            price: 100.0,
            quantity: 1.0,
            fee: 0.1,
        }],
        account: Account {
            cash: 900.0,
            positions: Vec::new(),
        },
        metrics: BacktestMetrics {
            return_rate: 0.1,
            max_drawdown: 0.0,
            win_rate: 1.0,
            trade_count: 1,
            sharpe: 1.0,
        },
        equity_curve: vec![EquityPoint {
            time: 1,
            equity: 900.0,
        }],
        trade_pnls: vec![None],
    }
}

#[test]
fn writes_json_report() {
    let path = temp_path("merrow_report.json");
    let result = sample_result();
    write_output(path.to_str().expect("path"), "json", &result).expect("write json");

    let content = fs::read_to_string(&path).expect("read json");
    assert!(content.contains("\"trades\""));
    assert!(content.contains("\"equity_curve\""));
    assert!(content.contains("\"costs\""));

    let _ = fs::remove_file(&path);
}

#[test]
fn writes_csv_report() {
    let path = temp_path("merrow_report.csv");
    let result = sample_result();
    write_output(path.to_str().expect("path"), "csv", &result).expect("write csv");

    let content = fs::read_to_string(&path).expect("read csv");
    let lines: Vec<&str> = content.trim().lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[1].contains("BTCUSDT"));

    let _ = fs::remove_file(&path);
}
