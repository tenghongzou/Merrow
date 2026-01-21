use merrow::config::Config;
use std::env;
use std::fs;
use std::path::PathBuf;

fn temp_config_path(name: &str) -> PathBuf {
    let mut path = env::temp_dir();
    path.push(format!("merrow_{name}.toml"));
    path
}

#[test]
fn loads_config_and_applies_env_overrides() {
    let path = temp_config_path("config_loader");
    let content = r#"
mode = "backtest"
symbol = "BTCUSDT"

[orders]
order_type = "limit"
limit_price_offset_bps = 10
fee_rate = 0.001
slippage_bps = 5

[triggers]
time_enabled = true
time_minutes = 15
price_enabled = true
trigger_mode_any = true
ma_window = 20
buy_threshold = 0.02
sell_threshold = 0.03

[strategy]
buy_cash_ratio = 0.5
sell_pos_ratio = 0.2
rebuy_cash_ratio = 0.5

[risk]
max_trade_ratio = 0.5
min_cash_reserve_ratio = 0.05
max_position_value_ratio = 0.8

[backtest]
start_time = "2024-01-01T00:00:00Z"
end_time = "2024-02-01T00:00:00Z"
initial_cash = 10000.0

[output]
format = "json"
path = "output/backtest_report.json"

[data]
source = "csv"
csv_path = "data/BTCUSDT_1m.csv"
candle_interval = "1m"
exchange_base_url = "https://api.binance.com"
exchange_limit = 1000
exchange_category = "spot"

[storage]
postgres_dsn = "postgres://user:pass@localhost:5432/merrow"
"#;

    fs::write(&path, content).expect("write temp config");
    env::set_var("MERROW_SYMBOL", "ETHUSDT");
    env::set_var("MERROW_TIME_TRIGGER_MINUTES", "20");

    let config = Config::load(path.to_str().expect("path")).expect("load config");

    assert_eq!(config.symbol, "ETHUSDT");
    assert_eq!(config.triggers.time_minutes, 20);

    env::remove_var("MERROW_SYMBOL");
    env::remove_var("MERROW_TIME_TRIGGER_MINUTES");
    let _ = fs::remove_file(&path);
}
