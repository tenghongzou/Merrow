use crate::models::Candle;
use crate::{Error, Result};
use chrono::DateTime;
use csv::ReaderBuilder;
use std::fs::File;

#[derive(serde::Deserialize)]
struct CandleRow {
    time: String,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

pub fn load_candles_from_csv(path: &str) -> Result<Vec<Candle>> {
    let file = File::open(path).map_err(|err| Error::new(format!("csv open failed: {err}")))?;
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(file);

    let mut rows: Vec<(i64, usize, Candle)> = Vec::new();
    for (index, result) in reader.deserialize::<CandleRow>().enumerate() {
        let row = result.map_err(|err| Error::new(format!("csv parse failed: {err}")))?;
        let time = parse_time(&row.time)?;
        validate_row(&row)?;
        rows.push((
            time,
            index,
            Candle {
                time,
                open: row.open,
                high: row.high,
                low: row.low,
                close: row.close,
                volume: row.volume,
            },
        ));
    }

    rows.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

    let mut deduped: Vec<Candle> = Vec::new();
    for (_, _, candle) in rows {
        if let Some(last) = deduped.last_mut() {
            if last.time == candle.time {
                *last = candle;
                continue;
            }
        }
        deduped.push(candle);
    }

    Ok(deduped)
}

fn validate_row(row: &CandleRow) -> Result<()> {
    if row.open <= 0.0 || row.high <= 0.0 || row.low <= 0.0 || row.close <= 0.0 {
        return Err(Error::new("prices must be positive"));
    }
    if row.volume < 0.0 {
        return Err(Error::new("volume must be non-negative"));
    }
    let max_oc = row.open.max(row.close);
    let min_oc = row.open.min(row.close);
    if row.high < max_oc {
        return Err(Error::new("high must be >= max(open, close)"));
    }
    if row.low > min_oc {
        return Err(Error::new("low must be <= min(open, close)"));
    }
    Ok(())
}

pub fn parse_time(value: &str) -> Result<i64> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(Error::new("time value is empty"));
    }
    if let Ok(epoch) = trimmed.parse::<i64>() {
        return Ok(epoch);
    }
    let parsed = DateTime::parse_from_rfc3339(trimmed)
        .map_err(|err| Error::new(format!("invalid time format: {err}")))?;
    Ok(parsed.timestamp())
}
