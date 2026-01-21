use merrow::data::csv_loader::{load_candles_from_csv, parse_time};
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
fn loads_fixture_sorted_and_deduped() {
    let path = fixture_path("candles.csv");
    let candles = load_candles_from_csv(path.to_str().expect("path")).expect("load");

    assert_eq!(candles.len(), 3);
    assert!(candles[0].time < candles[1].time);
    assert!(candles[1].time < candles[2].time);
    assert_eq!(candles[2].close, 107.0);
    assert_eq!(candles[0].time, 1_704_067_200);
}

#[test]
fn parse_time_accepts_epoch_and_rfc3339() {
    let epoch = parse_time("1704067200").expect("epoch");
    let rfc = parse_time("2024-01-01T00:00:00Z").expect("rfc3339");
    assert_eq!(epoch, rfc);
}

#[test]
fn rejects_invalid_row() {
    let mut path = env::temp_dir();
    path.push("merrow_invalid.csv");
    let content = "time,open,high,low,close,volume\n1704067200,100,90,95,98,1\n";
    fs::write(&path, content).expect("write temp");

    let result = load_candles_from_csv(path.to_str().expect("path"));
    assert!(result.is_err());

    let _ = fs::remove_file(&path);
}
