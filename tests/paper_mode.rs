use merrow::config::Config;
use merrow::data::csv_loader::load_candles_from_csv;
use merrow::paper::run_paper;
use std::env;
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("fixtures");
    path.push(name);
    path
}

#[test]
fn paper_runs_on_csv_fixture() {
    let path = fixture_path("candles.csv");
    let candles = load_candles_from_csv(path.to_str().expect("path")).expect("load");

    let mut config = Config::default();
    config.mode = "paper".to_string();
    config.triggers.time_enabled = true;
    config.triggers.price_enabled = false;
    config.triggers.time_minutes = 5;
    config.triggers.ma_window = 1;
    config.triggers.buy_threshold = 0.0;
    config.triggers.sell_threshold = 10.0;
    config.orders.order_type = "market".to_string();

    let result = run_paper(&candles, &config).expect("paper run");
    assert_eq!(result.metrics.trade_count, 1);
    assert_eq!(result.trades.len(), 1);
}
