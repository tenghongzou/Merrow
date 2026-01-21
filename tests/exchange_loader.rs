use merrow::data::exchange_loader::{
    map_bybit_interval, map_okx_interval, parse_binance_klines, parse_bybit_klines,
    parse_okx_candles,
};
use std::env;
use std::fs;
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("fixtures");
    path.push(name);
    path
}

#[test]
fn parses_binance_klines() {
    let path = fixture_path("binance_klines.json");
    let content = fs::read_to_string(&path).expect("read fixture");
    let parsed = parse_binance_klines(&content).expect("parse");

    assert_eq!(parsed.candles.len(), 2);
    assert_eq!(parsed.candles[0].close, 105.0);
    assert_eq!(parsed.candles[1].close, 108.0);
    assert_eq!(parsed.last_close_ms, Some(1704067319999));
}

#[test]
fn parses_bybit_klines() {
    let path = fixture_path("bybit_klines.json");
    let content = fs::read_to_string(&path).expect("read fixture");
    let parsed = parse_bybit_klines(&content).expect("parse");

    assert_eq!(parsed.candles.len(), 2);
    assert_eq!(parsed.candles[0].close, 108.0);
    assert_eq!(parsed.candles[1].close, 105.0);
    assert_eq!(parsed.oldest_start_ms, Some(1704067200000));
}

#[test]
fn parses_okx_candles() {
    let path = fixture_path("okx_candles.json");
    let content = fs::read_to_string(&path).expect("read fixture");
    let parsed = parse_okx_candles(&content).expect("parse");

    assert_eq!(parsed.candles.len(), 2);
    assert_eq!(parsed.candles[0].close, 108.0);
    assert_eq!(parsed.candles[1].close, 105.0);
    assert_eq!(parsed.oldest_start_ms, Some(1704067200000));
}

#[test]
fn maps_okx_intervals() {
    assert_eq!(map_okx_interval("1m").expect("1m"), "1m");
    assert_eq!(map_okx_interval("1h").expect("1h"), "1H");
    assert_eq!(map_okx_interval("1D").expect("1D"), "1D");
    assert_eq!(map_okx_interval("1M").expect("1M"), "1M");
    assert!(map_okx_interval("7m").is_err());
}

#[test]
fn maps_bybit_intervals() {
    assert_eq!(map_bybit_interval("1m").expect("1m"), "1");
    assert_eq!(map_bybit_interval("1h").expect("1h"), "60");
    assert_eq!(map_bybit_interval("1D").expect("1D"), "D");
    assert_eq!(map_bybit_interval("1M").expect("1M"), "M");
    assert!(map_bybit_interval("7m").is_err());
}
