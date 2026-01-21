use crate::config::Config;
use crate::data::csv_loader::parse_time;
use crate::models::Candle;
use crate::{Error, Result};
use reqwest::blocking::Client;
use reqwest::StatusCode;
use serde_json::Value;
use std::time::Duration;

pub struct ParsedCandles {
    pub candles: Vec<Candle>,
    pub last_close_ms: Option<i64>,
}

pub fn load_candles_from_exchange(config: &Config) -> Result<Vec<Candle>> {
    let exchange = config.exchange.to_lowercase();
    match exchange.as_str() {
        "binance" => load_binance_candles(config),
        "bybit" => load_bybit_candles(config),
        "okx" => load_okx_candles(config),
        _ => Err(Error::new("exchange data source not implemented")),
    }
}

fn load_binance_candles(config: &Config) -> Result<Vec<Candle>> {
    let start = parse_time(
        config
            .backtest
            .start_time
            .as_ref()
            .ok_or_else(|| Error::new("backtest.start_time must be set"))?,
    )?;
    let end = parse_time(
        config
            .backtest
            .end_time
            .as_ref()
            .ok_or_else(|| Error::new("backtest.end_time must be set"))?,
    )?;

    if start > end {
        return Err(Error::new("backtest.start_time must be <= end_time"));
    }

    let base_url = config
        .data
        .exchange_base_url
        .as_deref()
        .unwrap_or("https://api.binance.com");
    let limit = config.data.exchange_limit.unwrap_or(1000).min(1000);
    if limit == 0 {
        return Err(Error::new("data.exchange_limit must be positive"));
    }

    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|err| Error::new(format!("http client build failed: {err}")))?;

    let mut cursor_ms = start * 1000;
    let end_ms = end * 1000;
    let mut all: Vec<Candle> = Vec::new();

    loop {
        let url = format!("{base_url}/api/v3/klines");
        let query = vec![
            ("symbol".to_string(), config.symbol.clone()),
            ("interval".to_string(), config.data.candle_interval.clone()),
            ("startTime".to_string(), cursor_ms.to_string()),
            ("endTime".to_string(), end_ms.to_string()),
            ("limit".to_string(), limit.to_string()),
        ];
        let text = fetch_text_with_retry(&client, &url, &query)?;
        let parsed = parse_binance_klines(&text)?;
        if parsed.candles.is_empty() {
            break;
        }
        let batch_len = parsed.candles.len();
        all.extend(parsed.candles);

        let Some(last_close_ms) = parsed.last_close_ms else {
            break;
        };
        if last_close_ms >= end_ms {
            break;
        }

        cursor_ms = last_close_ms + 1;
        if batch_len < limit as usize {
            break;
        }
    }

    all.sort_by_key(|candle| candle.time);
    all.dedup_by_key(|candle| candle.time);
    Ok(all)
}

pub struct BybitParsed {
    pub candles: Vec<Candle>,
    pub oldest_start_ms: Option<i64>,
}

fn load_bybit_candles(config: &Config) -> Result<Vec<Candle>> {
    let start = parse_time(
        config
            .backtest
            .start_time
            .as_ref()
            .ok_or_else(|| Error::new("backtest.start_time must be set"))?,
    )?;
    let end = parse_time(
        config
            .backtest
            .end_time
            .as_ref()
            .ok_or_else(|| Error::new("backtest.end_time must be set"))?,
    )?;

    if start > end {
        return Err(Error::new("backtest.start_time must be <= end_time"));
    }

    let base_url = config
        .data
        .exchange_base_url
        .as_deref()
        .unwrap_or("https://api.bybit.com");
    let limit = config.data.exchange_limit.unwrap_or(1000).min(1000);
    if limit == 0 {
        return Err(Error::new("data.exchange_limit must be positive"));
    }

    let category = config
        .data
        .exchange_category
        .as_deref()
        .unwrap_or("spot");
    let interval = map_bybit_interval(&config.data.candle_interval)?;

    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|err| Error::new(format!("http client build failed: {err}")))?;

    let mut end_ms = end * 1000;
    let start_ms = start * 1000;
    let mut all: Vec<Candle> = Vec::new();

    loop {
        let url = format!("{base_url}/v5/market/kline");
        let query = vec![
            ("category".to_string(), category.to_string()),
            ("symbol".to_string(), config.symbol.clone()),
            ("interval".to_string(), interval.clone()),
            ("start".to_string(), start_ms.to_string()),
            ("end".to_string(), end_ms.to_string()),
            ("limit".to_string(), limit.to_string()),
        ];
        let text = fetch_text_with_retry(&client, &url, &query)?;
        let parsed = parse_bybit_klines(&text)?;
        if parsed.candles.is_empty() {
            break;
        }
        let batch_len = parsed.candles.len();
        all.extend(parsed.candles);

        let Some(oldest_start_ms) = parsed.oldest_start_ms else {
            break;
        };
        if oldest_start_ms <= start_ms {
            break;
        }

        end_ms = oldest_start_ms - 1;
        if batch_len < limit as usize {
            break;
        }
    }

    all.sort_by_key(|candle| candle.time);
    all.dedup_by_key(|candle| candle.time);
    Ok(all)
}

pub struct OkxParsed {
    pub candles: Vec<Candle>,
    pub oldest_start_ms: Option<i64>,
}

fn load_okx_candles(config: &Config) -> Result<Vec<Candle>> {
    let start = parse_time(
        config
            .backtest
            .start_time
            .as_ref()
            .ok_or_else(|| Error::new("backtest.start_time must be set"))?,
    )?;
    let end = parse_time(
        config
            .backtest
            .end_time
            .as_ref()
            .ok_or_else(|| Error::new("backtest.end_time must be set"))?,
    )?;

    if start > end {
        return Err(Error::new("backtest.start_time must be <= end_time"));
    }

    let base_url = config
        .data
        .exchange_base_url
        .as_deref()
        .unwrap_or("https://www.okx.com");
    let limit = config.data.exchange_limit.unwrap_or(100).min(100);
    if limit == 0 {
        return Err(Error::new("data.exchange_limit must be positive"));
    }

    let bar = map_okx_interval(&config.data.candle_interval)?;

    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|err| Error::new(format!("http client build failed: {err}")))?;

    let mut end_ms = end * 1000;
    let start_ms = start * 1000;
    let mut all: Vec<Candle> = Vec::new();

    loop {
        let url = format!("{base_url}/api/v5/market/candles");
        let query = vec![
            ("instId".to_string(), config.symbol.clone()),
            ("bar".to_string(), bar.clone()),
            ("after".to_string(), end_ms.to_string()),
            ("before".to_string(), start_ms.to_string()),
            ("limit".to_string(), limit.to_string()),
        ];
        let text = fetch_text_with_retry(&client, &url, &query)?;
        let parsed = parse_okx_candles(&text)?;
        if parsed.candles.is_empty() {
            break;
        }
        let batch_len = parsed.candles.len();
        all.extend(parsed.candles);

        let Some(oldest_start_ms) = parsed.oldest_start_ms else {
            break;
        };
        if oldest_start_ms <= start_ms {
            break;
        }
        end_ms = oldest_start_ms - 1;
        if batch_len < limit as usize {
            break;
        }
    }

    all.sort_by_key(|candle| candle.time);
    all.dedup_by_key(|candle| candle.time);
    Ok(all)
}

pub fn parse_binance_klines(payload: &str) -> Result<ParsedCandles> {
    let data: Vec<Vec<Value>> = serde_json::from_str(payload)
        .map_err(|err| Error::new(format!("json parse failed: {err}")))?;
    if data.is_empty() {
        return Ok(ParsedCandles {
            candles: Vec::new(),
            last_close_ms: None,
        });
    }

    let mut candles = Vec::with_capacity(data.len());
    let mut last_close_ms = None;

    for row in data {
        if row.len() < 7 {
            return Err(Error::new("kline row has insufficient fields"));
        }
        let open_time_ms = value_to_i64(&row[0])?;
        let open = value_to_f64(&row[1])?;
        let high = value_to_f64(&row[2])?;
        let low = value_to_f64(&row[3])?;
        let close = value_to_f64(&row[4])?;
        let volume = value_to_f64(&row[5])?;
        let close_time_ms = value_to_i64(&row[6])?;
        last_close_ms = Some(close_time_ms);

        candles.push(Candle {
            time: close_time_ms / 1000,
            open,
            high,
            low,
            close,
            volume,
        });

        let _ = open_time_ms;
    }

    Ok(ParsedCandles {
        candles,
        last_close_ms,
    })
}

pub fn parse_bybit_klines(payload: &str) -> Result<BybitParsed> {
    let data: Value = serde_json::from_str(payload)
        .map_err(|err| Error::new(format!("json parse failed: {err}")))?;
    let ret_code = data
        .get("retCode")
        .and_then(|value| value.as_i64())
        .unwrap_or(-1);
    if ret_code != 0 {
        return Err(Error::new("bybit retCode is not 0"));
    }

    let list = data
        .get("result")
        .and_then(|value| value.get("list"))
        .and_then(|value| value.as_array())
        .ok_or_else(|| Error::new("bybit result.list missing"))?;

    if list.is_empty() {
        return Ok(BybitParsed {
            candles: Vec::new(),
            oldest_start_ms: None,
        });
    }

    let mut candles = Vec::with_capacity(list.len());
    let mut oldest_start_ms = None;

    for row in list {
        let row = row
            .as_array()
            .ok_or_else(|| Error::new("bybit kline row is not array"))?;
        if row.len() < 6 {
            return Err(Error::new("bybit kline row has insufficient fields"));
        }

        let start_time_ms = value_to_i64(&row[0])?;
        let open = value_to_f64(&row[1])?;
        let high = value_to_f64(&row[2])?;
        let low = value_to_f64(&row[3])?;
        let close = value_to_f64(&row[4])?;
        let volume = value_to_f64(&row[5])?;

        oldest_start_ms = Some(start_time_ms);
        candles.push(Candle {
            time: start_time_ms / 1000,
            open,
            high,
            low,
            close,
            volume,
        });
    }

    Ok(BybitParsed {
        candles,
        oldest_start_ms,
    })
}

pub fn parse_okx_candles(payload: &str) -> Result<OkxParsed> {
    let data: Value = serde_json::from_str(payload)
        .map_err(|err| Error::new(format!("json parse failed: {err}")))?;
    let code = data
        .get("code")
        .and_then(|value| value.as_str())
        .unwrap_or("1");
    if code != "0" {
        return Err(Error::new("okx code is not 0"));
    }

    let list = data
        .get("data")
        .and_then(|value| value.as_array())
        .ok_or_else(|| Error::new("okx data missing"))?;

    if list.is_empty() {
        return Ok(OkxParsed {
            candles: Vec::new(),
            oldest_start_ms: None,
        });
    }

    let mut candles = Vec::with_capacity(list.len());
    let mut oldest_start_ms = None;

    for row in list {
        let row = row
            .as_array()
            .ok_or_else(|| Error::new("okx candle row is not array"))?;
        if row.len() < 6 {
            return Err(Error::new("okx candle row has insufficient fields"));
        }
        let ts_ms = value_to_i64(&row[0])?;
        let open = value_to_f64(&row[1])?;
        let high = value_to_f64(&row[2])?;
        let low = value_to_f64(&row[3])?;
        let close = value_to_f64(&row[4])?;
        let volume = value_to_f64(&row[5])?;
        oldest_start_ms = Some(ts_ms);
        candles.push(Candle {
            time: ts_ms / 1000,
            open,
            high,
            low,
            close,
            volume,
        });
    }

    Ok(OkxParsed {
        candles,
        oldest_start_ms,
    })
}

pub fn map_okx_interval(interval: &str) -> Result<String> {
    let trimmed = interval.trim();
    if trimmed.is_empty() {
        return Err(Error::new("okx interval must be non-empty"));
    }
    if trimmed == "1M" {
        return Ok("1M".to_string());
    }

    match trimmed.to_lowercase().as_str() {
        "1m" => Ok("1m".to_string()),
        "3m" => Ok("3m".to_string()),
        "5m" => Ok("5m".to_string()),
        "15m" => Ok("15m".to_string()),
        "30m" => Ok("30m".to_string()),
        "1h" => Ok("1H".to_string()),
        "2h" => Ok("2H".to_string()),
        "4h" => Ok("4H".to_string()),
        "6h" => Ok("6H".to_string()),
        "12h" => Ok("12H".to_string()),
        "1d" => Ok("1D".to_string()),
        "1w" => Ok("1W".to_string()),
        _ => Err(Error::new("unsupported okx interval")),
    }
}

pub fn map_bybit_interval(interval: &str) -> Result<String> {
    let trimmed = interval.trim();
    if trimmed.is_empty() {
        return Err(Error::new("bybit interval must be non-empty"));
    }

    if trimmed.chars().all(|ch| ch.is_ascii_digit()) {
        return Ok(trimmed.to_string());
    }

    if matches!(trimmed, "D" | "W" | "M") {
        return Ok(trimmed.to_string());
    }

    if trimmed == "1M" {
        return Ok("M".to_string());
    }

    match trimmed.to_lowercase().as_str() {
        "1m" => Ok("1".to_string()),
        "3m" => Ok("3".to_string()),
        "5m" => Ok("5".to_string()),
        "15m" => Ok("15".to_string()),
        "30m" => Ok("30".to_string()),
        "1h" => Ok("60".to_string()),
        "2h" => Ok("120".to_string()),
        "4h" => Ok("240".to_string()),
        "6h" => Ok("360".to_string()),
        "12h" => Ok("720".to_string()),
        "1d" => Ok("D".to_string()),
        "1w" => Ok("W".to_string()),
        _ => Err(Error::new("unsupported bybit interval")),
    }
}

fn fetch_text_with_retry(
    client: &Client,
    url: &str,
    query: &[(String, String)],
) -> Result<String> {
    const MAX_RETRIES: u32 = 3;
    const BASE_DELAY_MS: u64 = 500;

    let mut attempt = 0;
    loop {
        let response = client.get(url).query(query).send();
        match response {
            Ok(response) => {
                let status = response.status();
                if status.is_success() {
                    return response
                        .text()
                        .map_err(|err| Error::new(format!("http read failed: {err}")));
                }

                if should_retry(status) && attempt < MAX_RETRIES {
                    let delay = retry_delay_ms(&response, attempt, BASE_DELAY_MS);
                    std::thread::sleep(Duration::from_millis(delay));
                    attempt += 1;
                    continue;
                }

                return Err(Error::new(format!("exchange response status: {status}")));
            }
            Err(err) => {
                if attempt < MAX_RETRIES {
                    let delay = BASE_DELAY_MS * (1_u64 << attempt);
                    std::thread::sleep(Duration::from_millis(delay));
                    attempt += 1;
                    continue;
                }
                return Err(Error::new(format!("http request failed: {err}")));
            }
        }
    }
}

fn should_retry(status: StatusCode) -> bool {
    status == StatusCode::TOO_MANY_REQUESTS || status.is_server_error()
}

fn retry_delay_ms(response: &reqwest::blocking::Response, attempt: u32, base_ms: u64) -> u64 {
    if let Some(value) = response.headers().get("retry-after") {
        if let Ok(text) = value.to_str() {
            if let Ok(seconds) = text.parse::<u64>() {
                return seconds.saturating_mul(1000);
            }
        }
    }
    base_ms * (1_u64 << attempt)
}

fn value_to_i64(value: &Value) -> Result<i64> {
    match value {
        Value::Number(number) => number
            .as_i64()
            .ok_or_else(|| Error::new("number is not i64")),
        Value::String(text) => text
            .parse::<i64>()
            .map_err(|err| Error::new(format!("invalid i64: {err}"))),
        _ => Err(Error::new("unexpected value type for i64")),
    }
}

fn value_to_f64(value: &Value) -> Result<f64> {
    match value {
        Value::Number(number) => number
            .as_f64()
            .ok_or_else(|| Error::new("number is not f64")),
        Value::String(text) => text
            .parse::<f64>()
            .map_err(|err| Error::new(format!("invalid f64: {err}"))),
        _ => Err(Error::new("unexpected value type for f64")),
    }
}
