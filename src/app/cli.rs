use crate::app::report::write_output;
use crate::backtest::BacktestEngine;
use crate::config::Config;
use crate::core::build_engine_bundle;
use crate::data::csv_loader::{load_candles_from_csv, parse_time};
use crate::data::exchange_loader::load_candles_from_exchange;
use crate::exchange::Exchange;
use crate::exchange::binance::{BinanceConfig, BinanceExchange};
use crate::exchange::bybit::{BybitConfig, BybitExchange};
use crate::exchange::okx::{OkxConfig, OkxExchange};
use crate::exchange::sync::sync_account;
use crate::exchange::CandleRequest;
use crate::core::strategy::Strategy;
use crate::paper::run_paper_with_state;
use crate::app::metrics;
use crate::storage::postgres::PostgresStorage;
use crate::{Error, Result};
use std::sync::OnceLock;
use std::env;
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::{info, warn};

pub fn run() -> Result<()> {
    metrics::init_start_time();
    let args: Vec<String> = env::args().collect();
    let cli = parse_args(&args)?;

    if cli.show_help {
        print_usage();
        return Ok(());
    }

    let mut config = Config::load(&cli.config_path)?;
    if let Some(symbol) = cli.symbol_override {
        config.symbol = symbol;
    }
    if let Some(format) = cli.output_format {
        config.output.format = format;
    }
    if let Some(path) = cli.output_path {
        config.output.path = path;
    }
    if let Some(value) = cli.initial_cash_override {
        config.backtest.initial_cash = value;
    }
    config.validate()?;

    if let Some(value) = cli.pg_enabled_override {
        let _ = CLI_PG_ENABLED_OVERRIDE.set(value);
    }

    if config.mode == "backtest" {
        let mut bundle = build_engine_bundle(&config)?;
        let candles = match config.data.source.as_str() {
            "csv" => {
                let path = config
                    .data
                    .csv_path
                    .as_ref()
                    .ok_or_else(|| Error::new("data.csv_path must be set"))?;
                load_candles_from_csv(path)?
            }
            "exchange" => load_candles_from_exchange(&config)?,
            _ => return Err(Error::new("unknown data source")),
        };

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

        let filtered: Vec<_> = candles
            .into_iter()
            .filter(|candle| candle.time >= start && candle.time <= end)
            .collect();

        let engine = BacktestEngine;
        let result = engine.run_strategy(
            &filtered,
            &config,
            &bundle.trigger_engine,
            &mut bundle.strategy,
            &mut bundle.order_flow,
            config.backtest.initial_cash,
        )?;

        println!("trades: {}", result.metrics.trade_count);
        println!("return_rate: {:.6}", result.metrics.return_rate);
        println!("max_drawdown: {:.6}", result.metrics.max_drawdown);
        println!("win_rate: {:.6}", result.metrics.win_rate);
        println!("sharpe: {:.6}", result.metrics.sharpe);
        let total_fees: f64 = result.trades.iter().map(|trade| trade.fee).sum();
        let average_fee = if result.trades.is_empty() {
            0.0
        } else {
            total_fees / result.trades.len() as f64
        };
        let realized_pnls: Vec<f64> = result.trade_pnls.iter().filter_map(|pnl| *pnl).collect();
        let total_realized_pnl: f64 = realized_pnls.iter().sum();
        let average_realized_pnl = if realized_pnls.is_empty() {
            0.0
        } else {
            total_realized_pnl / realized_pnls.len() as f64
        };

        println!("total_fees: {:.6}", total_fees);
        println!("average_fee: {:.6}", average_fee);
        println!("realized_pnl_total: {:.6}", total_realized_pnl);
        println!("realized_pnl_average: {:.6}", average_realized_pnl);
        println!("realized_pnl_count: {}", realized_pnls.len());

        if !realized_pnls.is_empty() {
            let pnl_list = realized_pnls
                .iter()
                .map(|pnl| format!("{pnl:.6}"))
                .collect::<Vec<String>>()
                .join(", ");
            println!("trade_pnls: [{pnl_list}]");
        }

        if config.output.format != "none" {
            write_output(&config.output.path, &config.output.format, &result)?;
            println!(
                "output_written: {} ({})",
                config.output.path, config.output.format
            );
        }

        metrics::record_backtest(&result.metrics, result.trades.len());
        metrics::write_if_configured()?;

        maybe_persist_backtest(&config, &filtered, &result)?;
    } else if config.mode == "paper" {
        run_paper_mode(&config)?;
    } else if config.mode == "live" {
        run_live(&config, cli.live_execute)?;
    }
    Ok(())
}

static CLI_PG_ENABLED_OVERRIDE: OnceLock<bool> = OnceLock::new();

struct CliArgs {
    config_path: String,
    symbol_override: Option<String>,
    output_format: Option<String>,
    output_path: Option<String>,
    initial_cash_override: Option<f64>,
    live_execute: bool,
    pg_enabled_override: Option<bool>,
    show_help: bool,
}

fn parse_args(args: &[String]) -> Result<CliArgs> {
    let mut config_path = "config.toml".to_string();
    let mut symbol_override = None;
    let mut output_format = None;
    let mut output_path = None;
    let mut initial_cash_override = None;
    let mut live_execute = false;
    let mut pg_enabled_override = None;
    let mut show_help = false;

    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--help" | "-h" => {
                show_help = true;
                index += 1;
            }
            "--config" | "-c" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| Error::new("missing value for --config"))?;
                config_path = value.to_string();
                index += 2;
            }
            "--symbol" | "-s" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| Error::new("missing value for --symbol"))?;
                symbol_override = Some(value.to_string());
                index += 2;
            }
            "--output-format" | "-f" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| Error::new("missing value for --output-format"))?;
                output_format = Some(value.to_string());
                index += 2;
            }
            "--output-path" | "-o" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| Error::new("missing value for --output-path"))?;
                output_path = Some(value.to_string());
                index += 2;
            }
            "--initial-cash" | "-i" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| Error::new("missing value for --initial-cash"))?;
                let parsed = value
                    .parse::<f64>()
                    .map_err(|_| Error::new("invalid value for --initial-cash"))?;
                initial_cash_override = Some(parsed);
                index += 2;
            }
            "--live-execute" => {
                live_execute = true;
                index += 1;
            }
            "--pg-enabled" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| Error::new("missing value for --pg-enabled"))?;
                pg_enabled_override = Some(parse_bool(value, "--pg-enabled")?);
                index += 2;
            }
            unknown => {
                return Err(Error::new(format!("unknown argument: {unknown}")));
            }
        }
    }

    Ok(CliArgs {
        config_path,
        symbol_override,
        output_format,
        output_path,
        initial_cash_override,
        live_execute,
        pg_enabled_override,
        show_help,
    })
}

fn print_usage() {
    println!("usage: merrow [--config <path>] [--symbol <SYMBOL>] [--output-format <fmt>] [--output-path <path>] [--initial-cash <amount>] [--live-execute] [--pg-enabled <bool>]");
    println!("  -c, --config   Path to config.toml (default: config.toml)");
    println!("  -s, --symbol   Override symbol from config");
    println!("  -f, --output-format   Override output format (none|json|csv)");
    println!("  -o, --output-path     Override output path");
    println!("  -i, --initial-cash    Override backtest initial cash");
    println!("      --live-execute    Execute live orders (default: dry-run)");
    println!("      --pg-enabled      Enable PGSQL persistence (true/false)");
    println!("  -h, --help     Show this help");
}

fn parse_bool(value: &str, flag: &str) -> Result<bool> {
    match value.trim().to_lowercase().as_str() {
        "1" | "true" | "yes" => Ok(true),
        "0" | "false" | "no" => Ok(false),
        _ => Err(Error::new(format!("{flag} must be true/false"))),
    }
}

fn retry_with_backoff<T, F>(label: &str, mut action: F) -> Result<T>
where
    F: FnMut() -> Result<T>,
{
    let (max_retries, base_delay_ms) = live_retry_settings();
    let mut attempt = 0u32;
    loop {
        match action() {
            Ok(value) => return Ok(value),
            Err(err) => {
                let should_retry = attempt < max_retries && is_transient_error(&err.message);
                if !should_retry {
                    metrics::inc_error();
                    if attempt >= max_retries {
                        metrics::inc_live_retry_exhausted();
                    }
                    return Err(err);
                }
                metrics::inc_live_retry();
                let delay = backoff_delay_ms(base_delay_ms, attempt);
                warn!(
                    label = %label,
                    attempt = attempt + 1,
                    max_retries,
                    delay_ms = delay,
                    error = %err.message,
                    "live retry"
                );
                sleep(Duration::from_millis(delay));
                attempt += 1;
            }
        }
    }
}

fn live_retry_settings() -> (u32, u64) {
    let max_retries = env::var("MERROW_LIVE_RETRY_MAX")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(3);
    let base_delay_ms = env::var("MERROW_LIVE_RETRY_BASE_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(500);
    (max_retries, base_delay_ms)
}

fn backoff_delay_ms(base: u64, attempt: u32) -> u64 {
    let shift = if attempt >= 63 { u64::MAX } else { 1u64 << attempt };
    let max_delay_ms = env::var("MERROW_LIVE_RETRY_MAX_DELAY_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(8_000);
    let jitter_pct = env::var("MERROW_LIVE_RETRY_JITTER_PCT")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(20)
        .min(100);
    let base_delay = base.saturating_mul(shift).min(max_delay_ms);
    let jitter = jitter_amount(base_delay, jitter_pct);
    base_delay.saturating_add(jitter).min(max_delay_ms)
}

fn jitter_amount(base_delay: u64, jitter_pct: u32) -> u64 {
    if jitter_pct == 0 || base_delay == 0 {
        return 0;
    }
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.subsec_nanos() as u64)
        .unwrap_or(0);
    let bucket = (nanos % 10_000) as u64;
    let jitter_max = base_delay.saturating_mul(jitter_pct as u64) / 100;
    if jitter_max == 0 {
        0
    } else {
        bucket % (jitter_max + 1)
    }
}

fn is_transient_error(message: &str) -> bool {
    let lowered = message.to_lowercase();
    lowered.contains("429")
        || lowered.contains("too many requests")
        || lowered.contains("timed out")
        || lowered.contains("timeout")
        || lowered.contains("connection reset")
        || lowered.contains("broken pipe")
        || lowered.contains("http request failed")
        || lowered.contains("response status: 5")
        || lowered.contains("response status: 502")
        || lowered.contains("response status: 503")
        || lowered.contains("response status: 504")
}

#[cfg(test)]
mod tests {
    use super::{backoff_delay_ms, is_transient_error, parse_args};

    #[test]
    fn parses_defaults() {
        let args = vec!["merrow".to_string()];
        let parsed = parse_args(&args).expect("parse");
        assert_eq!(parsed.config_path, "config.toml");
        assert!(parsed.symbol_override.is_none());
        assert!(!parsed.show_help);
    }

    #[test]
    fn parses_overrides() {
        let args = vec![
            "merrow".to_string(),
            "--config".to_string(),
            "custom.toml".to_string(),
            "--symbol".to_string(),
            "ETHUSDT".to_string(),
            "--output-format".to_string(),
            "json".to_string(),
            "--output-path".to_string(),
            "out/report.json".to_string(),
            "--initial-cash".to_string(),
            "5000".to_string(),
            "--live-execute".to_string(),
            "--pg-enabled".to_string(),
            "true".to_string(),
        ];
        let parsed = parse_args(&args).expect("parse");
        assert_eq!(parsed.config_path, "custom.toml");
        assert_eq!(parsed.symbol_override.as_deref(), Some("ETHUSDT"));
        assert_eq!(parsed.output_format.as_deref(), Some("json"));
        assert_eq!(parsed.output_path.as_deref(), Some("out/report.json"));
        assert_eq!(parsed.initial_cash_override, Some(5000.0));
        assert!(parsed.live_execute);
        assert_eq!(parsed.pg_enabled_override, Some(true));
    }

    #[test]
    fn detects_transient_errors() {
        assert!(is_transient_error("binance response status: 429"));
        assert!(is_transient_error("http request failed: timeout"));
        assert!(is_transient_error("response status: 503"));
        assert!(!is_transient_error("invalid api key"));
    }

    #[test]
    fn backoff_clamps_with_defaults() {
        let delay = backoff_delay_ms(500, 10);
        assert!(delay <= 8_000);
    }
}

fn run_live(config: &Config, live_execute: bool) -> Result<()> {
    if config.data.source != "exchange" {
        return Err(Error::new("live mode requires data.source=exchange"));
    }
    let exchange = config.exchange.to_lowercase();
    match exchange.as_str() {
        "binance" => run_live_binance(config, live_execute),
        "bybit" => run_live_bybit(config, live_execute),
        "okx" => run_live_okx(config, live_execute),
        _ => Err(Error::new("live mode only supports binance/bybit/okx currently")),
    }
}

fn run_paper_mode(config: &Config) -> Result<()> {
    let lookback = (config.triggers.ma_window as usize).max(1) + 2;
    let state_path =
        env::var("MERROW_PAPER_STATE_PATH").unwrap_or_else(|_| "output/paper_state.json".to_string());
    let candles = match config.data.source.as_str() {
        "csv" => {
            let path = config
                .data
                .csv_path
                .as_ref()
                .ok_or_else(|| Error::new("data.csv_path must be set"))?;
            let mut all = load_candles_from_csv(path)?;
            if all.len() > lookback {
                all = all[all.len() - lookback..].to_vec();
            }
            all
        }
        "exchange" => {
            let interval_secs = parse_interval_seconds(&config.data.candle_interval)? as i64;
            let end_sec = now_ms()? / 1000;
            let start_sec = end_sec.saturating_sub(interval_secs * lookback as i64);
            let mut temp = config.clone();
            temp.backtest.start_time = Some(start_sec.to_string());
            temp.backtest.end_time = Some(end_sec.to_string());
            load_candles_from_exchange(&temp)?
        }
        _ => return Err(Error::new("unknown data source")),
    };

    let result = run_paper_with_state(&candles, config, &state_path)?;
    println!("paper_trades: {}", result.metrics.trade_count);
    println!("paper_return_rate: {:.6}", result.metrics.return_rate);
    println!("paper_max_drawdown: {:.6}", result.metrics.max_drawdown);
    println!("paper_win_rate: {:.6}", result.metrics.win_rate);
    println!("paper_sharpe: {:.6}", result.metrics.sharpe);
    println!("paper_cash: {:.6}", result.account.cash);
    println!("paper_positions: {:?}", result.account.positions);
    println!("paper_state_path: {}", state_path);
    if config.output.format != "none" {
        write_output(&config.output.path, &config.output.format, &result)?;
        println!(
            "paper_output_written: {} ({})",
            config.output.path, config.output.format
        );
    }
    metrics::record_paper(&result.metrics, result.trades.len());
    metrics::write_if_configured()?;
    maybe_persist_paper(config, &candles, &result)?;
    Ok(())
}

fn pg_enabled() -> bool {
    if let Some(value) = CLI_PG_ENABLED_OVERRIDE.get() {
        return *value;
    }
    match env::var("MERROW_PG_ENABLED") {
        Ok(value) => matches!(value.to_lowercase().as_str(), "1" | "true" | "yes"),
        Err(_) => false,
    }
}

fn pg_init_schema() -> bool {
    match env::var("MERROW_PG_INIT_SCHEMA") {
        Ok(value) => !matches!(value.to_lowercase().as_str(), "0" | "false" | "no"),
        Err(_) => true,
    }
}

fn maybe_persist_backtest(
    config: &Config,
    candles: &[crate::models::Candle],
    result: &crate::backtest::BacktestResult,
) -> Result<()> {
    if !pg_enabled() {
        return Ok(());
    }
    let storage = PostgresStorage::new(&config.storage.postgres_dsn);
    if pg_init_schema() {
        storage.ensure_schema()?;
    }
    let run_id = storage.persist_backtest(config, candles, result)?;
    println!("pg_backtest_run_id: {}", run_id);
    info!(run_id = %run_id, "pg_backtest_saved");
    Ok(())
}

fn maybe_persist_paper(
    config: &Config,
    candles: &[crate::models::Candle],
    result: &crate::backtest::BacktestResult,
) -> Result<()> {
    if !pg_enabled() {
        return Ok(());
    }
    let storage = PostgresStorage::new(&config.storage.postgres_dsn);
    if pg_init_schema() {
        storage.ensure_schema()?;
    }
    storage.persist_paper(config, candles, result)?;
    println!("pg_paper_saved: true");
    info!("pg_paper_saved");
    Ok(())
}

fn run_live_binance(config: &Config, live_execute: bool) -> Result<()> {
    let api_key = env::var("MERROW_BINANCE_API_KEY")
        .map_err(|_| Error::new("MERROW_BINANCE_API_KEY must be set"))?;
    let api_secret = env::var("MERROW_BINANCE_API_SECRET")
        .map_err(|_| Error::new("MERROW_BINANCE_API_SECRET must be set"))?;
    let base_url = env::var("MERROW_BINANCE_BASE_URL")
        .ok()
        .or_else(|| config.data.exchange_base_url.clone())
        .unwrap_or_else(|| "https://api.binance.com".to_string());
    let recv_window = env::var("MERROW_BINANCE_RECV_WINDOW")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(5000);

    let exchange = BinanceExchange::new(BinanceConfig {
        base_url,
        api_key,
        api_secret,
        recv_window,
        timeout_secs: 30,
        default_symbol: Some(config.symbol.clone()),
    })?;

    let cash_asset = env::var("MERROW_CASH_ASSET")
        .ok()
        .or_else(|| infer_cash_asset(&config.symbol))
        .ok_or_else(|| Error::new("cash asset not found; set MERROW_CASH_ASSET"))?;

    run_live_with_exchange(config, live_execute, &exchange, &cash_asset)
}

fn run_live_bybit(config: &Config, live_execute: bool) -> Result<()> {
    let api_key = env::var("MERROW_BYBIT_API_KEY")
        .map_err(|_| Error::new("MERROW_BYBIT_API_KEY must be set"))?;
    let api_secret = env::var("MERROW_BYBIT_API_SECRET")
        .map_err(|_| Error::new("MERROW_BYBIT_API_SECRET must be set"))?;
    let base_url = env::var("MERROW_BYBIT_BASE_URL")
        .ok()
        .or_else(|| config.data.exchange_base_url.clone())
        .unwrap_or_else(|| "https://api.bybit.com".to_string());
    let recv_window = env::var("MERROW_BYBIT_RECV_WINDOW")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(5000);
    let account_type = env::var("MERROW_BYBIT_ACCOUNT_TYPE").unwrap_or_else(|_| "UNIFIED".to_string());
    let category = config
        .data
        .exchange_category
        .as_deref()
        .unwrap_or("spot")
        .to_string();

    let exchange = BybitExchange::new(BybitConfig {
        base_url,
        api_key,
        api_secret,
        recv_window,
        timeout_secs: 30,
        category,
        account_type,
        default_symbol: Some(config.symbol.clone()),
    })?;

    let cash_asset = env::var("MERROW_CASH_ASSET")
        .ok()
        .or_else(|| infer_cash_asset(&config.symbol))
        .ok_or_else(|| Error::new("cash asset not found; set MERROW_CASH_ASSET"))?;

    run_live_with_exchange(config, live_execute, &exchange, &cash_asset)
}

fn run_live_okx(config: &Config, live_execute: bool) -> Result<()> {
    let api_key =
        env::var("MERROW_OKX_API_KEY").map_err(|_| Error::new("MERROW_OKX_API_KEY must be set"))?;
    let api_secret = env::var("MERROW_OKX_API_SECRET")
        .map_err(|_| Error::new("MERROW_OKX_API_SECRET must be set"))?;
    let passphrase = env::var("MERROW_OKX_PASSPHRASE")
        .map_err(|_| Error::new("MERROW_OKX_PASSPHRASE must be set"))?;
    let base_url = env::var("MERROW_OKX_BASE_URL")
        .ok()
        .or_else(|| config.data.exchange_base_url.clone())
        .unwrap_or_else(|| "https://www.okx.com".to_string());

    let cash_asset = env::var("MERROW_CASH_ASSET")
        .ok()
        .or_else(|| infer_cash_asset(&config.symbol))
        .ok_or_else(|| Error::new("cash asset not found; set MERROW_CASH_ASSET"))?;
    let okx_symbol = normalize_okx_symbol(&config.symbol, &cash_asset);
    let mut live_config = config.clone();
    live_config.symbol = okx_symbol;

    let exchange = OkxExchange::new(OkxConfig {
        base_url,
        api_key,
        api_secret,
        passphrase,
        timeout_secs: 30,
        default_symbol: Some(live_config.symbol.clone()),
    })?;

    run_live_with_exchange(&live_config, live_execute, &exchange, &cash_asset)
}

fn run_live_with_exchange<E: Exchange>(
    config: &Config,
    live_execute: bool,
    exchange: &E,
    cash_asset: &str,
) -> Result<()> {
    let mut triggered = false;
    let mut signals_count = 0usize;
    let mut orders_count = 0usize;
    let mut orders_sent = 0usize;
    let snapshot = retry_with_backoff("sync_account", || sync_account(exchange))?;
    let account = account_from_snapshot(&snapshot, &config.symbol, cash_asset);

    let now_ms = now_ms()?;
    let interval_secs = parse_interval_seconds(&config.data.candle_interval)? as i64;
    let lookback = config.triggers.ma_window as i64 + 2;
    let start_ms = now_ms - interval_secs * 1000 * lookback;

    let candle_request = CandleRequest {
        symbol: config.symbol.clone(),
        interval: config.data.candle_interval.clone(),
        start_time: start_ms,
        end_time: now_ms,
    };
    let candles = retry_with_backoff("fetch_candles", || exchange.fetch_candles(&candle_request))?;

    if candles.is_empty() {
        return Err(Error::new("no candles returned for live evaluation"));
    }

    let mut bundle = build_engine_bundle(config)?;
    let last = candles.last().ok_or_else(|| Error::new("no candle data"))?;
    let trigger_ctx = crate::core::TriggerContext {
        candle: last,
        history: &candles,
        now: last.time,
    };

    if !bundle.trigger_engine.should_fire(&trigger_ctx) {
        info!("live: trigger not fired");
        metrics::record_live(triggered, signals_count, orders_count, orders_sent);
        metrics::write_if_configured()?;
        return Ok(());
    }
    triggered = true;

    let strategy_ctx = crate::core::StrategyContext {
        candle: last,
        history: &candles,
        account: &account,
        now: last.time,
    };
    let signals = bundle.strategy.on_tick(&strategy_ctx);
    signals_count = signals.len();
    let mut orders = Vec::new();
    for signal in signals {
        orders.extend(bundle.order_flow.plan(signal, &strategy_ctx, config)?);
    }
    orders_count = orders.len();

    if orders.is_empty() {
        info!("live: no orders to place");
        metrics::record_live(triggered, signals_count, orders_count, orders_sent);
        metrics::write_if_configured()?;
        return Ok(());
    }

    if live_execute {
        info!("live: executing {} order(s)", orders.len());
        for order in orders {
            let ack = retry_with_backoff("place_order", || exchange.place_order(&order))?;
            info!(
                client_id = %ack.client_order_id,
                status = ?ack.status,
                "order_ack"
            );
            orders_sent += 1;
        }
    } else {
        info!("live: dry-run mode (no orders sent)");
        for order in &orders {
            info!(
                symbol = %order.symbol,
                side = ?order.side,
                qty = order.quantity,
                "dry_order"
            );
        }
    }

    metrics::record_live(triggered, signals_count, orders_count, orders_sent);
    metrics::write_if_configured()?;
    Ok(())
}

fn account_from_snapshot(
    snapshot: &crate::exchange::sync::AccountSnapshot,
    symbol: &str,
    cash_asset: &str,
) -> crate::models::Account {
    let cash = snapshot
        .balances
        .iter()
        .find(|balance| balance.asset == cash_asset)
        .map(|balance| balance.free)
        .unwrap_or(0.0);
    let base_asset = if let Some(base) = symbol.split('-').next() {
        if symbol.contains('-') {
            base.to_string()
        } else {
            symbol
                .strip_suffix(cash_asset)
                .map(|value| value.to_string())
                .unwrap_or_else(|| symbol.to_string())
        }
    } else {
        symbol
            .strip_suffix(cash_asset)
            .map(|value| value.to_string())
            .unwrap_or_else(|| symbol.to_string())
    };
    let base_qty = snapshot
        .balances
        .iter()
        .find(|balance| balance.asset == base_asset)
        .map(|balance| balance.free + balance.locked)
        .unwrap_or(0.0);
    let mut positions = Vec::new();
    if base_qty > 0.0 {
        positions.push(crate::models::Position {
            symbol: symbol.to_string(),
            quantity: base_qty,
            avg_price: 0.0,
        });
    }
    crate::models::Account { cash, positions }
}

fn infer_cash_asset(symbol: &str) -> Option<String> {
    let candidates = ["USDT", "USDC", "USD", "BUSD", "EUR"];
    if symbol.contains('-') {
        let parts: Vec<&str> = symbol.split('-').collect();
        if parts.len() >= 2 {
            let quote = parts[1];
            if candidates.contains(&quote) {
                return Some(quote.to_string());
            }
            if let Some(last) = parts.last() {
                if candidates.contains(last) {
                    return Some((*last).to_string());
                }
            }
        }
    }
    for suffix in candidates {
        if symbol.ends_with(suffix) {
            return Some(suffix.to_string());
        }
    }
    None
}

fn parse_interval_seconds(interval: &str) -> Result<u64> {
    let trimmed = interval.trim();
    match trimmed {
        "1" => return Ok(60),
        "3" => return Ok(180),
        "5" => return Ok(300),
        "15" => return Ok(900),
        "30" => return Ok(1800),
        "60" => return Ok(3600),
        "120" => return Ok(7200),
        "240" => return Ok(14400),
        "360" => return Ok(21600),
        "720" => return Ok(43200),
        "D" | "d" => return Ok(86400),
        "W" | "w" => return Ok(604800),
        "M" => return Ok(2592000),
        _ => {}
    }

    match trimmed.to_lowercase().as_str() {
        "1m" => Ok(60),
        "3m" => Ok(180),
        "5m" => Ok(300),
        "15m" => Ok(900),
        "30m" => Ok(1800),
        "1h" => Ok(3600),
        "2h" => Ok(7200),
        "4h" => Ok(14400),
        "6h" => Ok(21600),
        "12h" => Ok(43200),
        "1d" => Ok(86400),
        "1w" => Ok(604800),
        _ => Err(Error::new("unsupported interval for live mode")),
    }
}

fn normalize_okx_symbol(symbol: &str, cash_asset: &str) -> String {
    if symbol.contains('-') {
        return symbol.to_string();
    }
    if let Some(base) = symbol.strip_suffix(cash_asset) {
        return format!("{base}-{cash_asset}");
    }
    symbol.to_string()
}

fn now_ms() -> Result<i64> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| Error::new("system time before unix epoch"))?;
    Ok(now.as_millis() as i64)
}
