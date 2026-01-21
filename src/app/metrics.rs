use crate::backtest::BacktestMetrics;
use crate::{Error, Result};
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

static START_TIME: OnceLock<i64> = OnceLock::new();

static BACKTEST_RUNS_TOTAL: AtomicU64 = AtomicU64::new(0);
static PAPER_RUNS_TOTAL: AtomicU64 = AtomicU64::new(0);
static LIVE_RUNS_TOTAL: AtomicU64 = AtomicU64::new(0);

static TRADES_TOTAL: AtomicU64 = AtomicU64::new(0);
static SIGNALS_TOTAL: AtomicU64 = AtomicU64::new(0);
static ORDERS_PLANNED_TOTAL: AtomicU64 = AtomicU64::new(0);
static LIVE_ORDERS_SENT_TOTAL: AtomicU64 = AtomicU64::new(0);
static TRIGGER_FIRE_TOTAL: AtomicU64 = AtomicU64::new(0);

static LIVE_RETRY_TOTAL: AtomicU64 = AtomicU64::new(0);
static LIVE_RETRY_EXHAUSTED_TOTAL: AtomicU64 = AtomicU64::new(0);
static ERRORS_TOTAL: AtomicU64 = AtomicU64::new(0);

static LAST_RUN_TIMESTAMP: AtomicU64 = AtomicU64::new(0);
static LAST_RUN_MODE_BACKTEST: AtomicU64 = AtomicU64::new(0);
static LAST_RUN_MODE_PAPER: AtomicU64 = AtomicU64::new(0);
static LAST_RUN_MODE_LIVE: AtomicU64 = AtomicU64::new(0);

static LAST_RETURN_RATE_BITS: AtomicU64 = AtomicU64::new(0);
static LAST_MAX_DRAWDOWN_BITS: AtomicU64 = AtomicU64::new(0);
static LAST_WIN_RATE_BITS: AtomicU64 = AtomicU64::new(0);
static LAST_SHARPE_BITS: AtomicU64 = AtomicU64::new(0);

pub fn init_start_time() {
    let _ = START_TIME.set(now_epoch());
}

pub fn record_backtest(metrics: &BacktestMetrics, trade_count: usize) {
    BACKTEST_RUNS_TOTAL.fetch_add(1, Ordering::Relaxed);
    TRADES_TOTAL.fetch_add(trade_count as u64, Ordering::Relaxed);
    set_last_metrics(metrics);
    set_last_run_mode("backtest");
}

pub fn record_paper(metrics: &BacktestMetrics, trade_count: usize) {
    PAPER_RUNS_TOTAL.fetch_add(1, Ordering::Relaxed);
    TRADES_TOTAL.fetch_add(trade_count as u64, Ordering::Relaxed);
    set_last_metrics(metrics);
    set_last_run_mode("paper");
}

pub fn record_live(triggered: bool, signals: usize, orders: usize, sent: usize) {
    LIVE_RUNS_TOTAL.fetch_add(1, Ordering::Relaxed);
    SIGNALS_TOTAL.fetch_add(signals as u64, Ordering::Relaxed);
    ORDERS_PLANNED_TOTAL.fetch_add(orders as u64, Ordering::Relaxed);
    LIVE_ORDERS_SENT_TOTAL.fetch_add(sent as u64, Ordering::Relaxed);
    if triggered {
        TRIGGER_FIRE_TOTAL.fetch_add(1, Ordering::Relaxed);
    }
    set_last_run_mode("live");
}

pub fn inc_live_retry() {
    LIVE_RETRY_TOTAL.fetch_add(1, Ordering::Relaxed);
}

pub fn inc_live_retry_exhausted() {
    LIVE_RETRY_EXHAUSTED_TOTAL.fetch_add(1, Ordering::Relaxed);
}

pub fn inc_error() {
    ERRORS_TOTAL.fetch_add(1, Ordering::Relaxed);
}

pub fn write_if_configured() -> Result<()> {
    let path = match std::env::var("MERROW_METRICS_PATH") {
        Ok(value) if !value.trim().is_empty() => value,
        _ => return Ok(()),
    };
    write_metrics(&path)
}

pub fn write_metrics(path: &str) -> Result<()> {
    let content = render();
    let path = Path::new(path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| Error::new(format!("metrics dir create failed: {err}")))?;
    }
    fs::write(path, content).map_err(|err| Error::new(format!("metrics write failed: {err}")))
}

pub fn render() -> String {
    let uptime = uptime_seconds();
    let last_ts = LAST_RUN_TIMESTAMP.load(Ordering::Relaxed);
    let mut output = String::new();
    push_line(&mut output, "# HELP merrow_up Merrow process up");
    push_line(&mut output, "# TYPE merrow_up gauge");
    push_line(&mut output, "merrow_up 1");
    push_line(&mut output, "# HELP merrow_uptime_seconds Process uptime in seconds");
    push_line(&mut output, "# TYPE merrow_uptime_seconds gauge");
    push_line(&mut output, &format!("merrow_uptime_seconds {}", uptime));
    push_line(&mut output, "# HELP merrow_backtest_runs_total Total backtest runs");
    push_line(&mut output, "# TYPE merrow_backtest_runs_total counter");
    push_line(
        &mut output,
        &format!(
            "merrow_backtest_runs_total {}",
            BACKTEST_RUNS_TOTAL.load(Ordering::Relaxed)
        ),
    );
    push_line(&mut output, "# HELP merrow_paper_runs_total Total paper runs");
    push_line(&mut output, "# TYPE merrow_paper_runs_total counter");
    push_line(
        &mut output,
        &format!(
            "merrow_paper_runs_total {}",
            PAPER_RUNS_TOTAL.load(Ordering::Relaxed)
        ),
    );
    push_line(&mut output, "# HELP merrow_live_runs_total Total live runs");
    push_line(&mut output, "# TYPE merrow_live_runs_total counter");
    push_line(
        &mut output,
        &format!(
            "merrow_live_runs_total {}",
            LIVE_RUNS_TOTAL.load(Ordering::Relaxed)
        ),
    );
    push_line(&mut output, "# HELP merrow_trades_total Total trades");
    push_line(&mut output, "# TYPE merrow_trades_total counter");
    push_line(
        &mut output,
        &format!("merrow_trades_total {}", TRADES_TOTAL.load(Ordering::Relaxed)),
    );
    push_line(&mut output, "# HELP merrow_signals_total Total signals");
    push_line(&mut output, "# TYPE merrow_signals_total counter");
    push_line(
        &mut output,
        &format!("merrow_signals_total {}", SIGNALS_TOTAL.load(Ordering::Relaxed)),
    );
    push_line(&mut output, "# HELP merrow_orders_planned_total Total planned orders");
    push_line(&mut output, "# TYPE merrow_orders_planned_total counter");
    push_line(
        &mut output,
        &format!(
            "merrow_orders_planned_total {}",
            ORDERS_PLANNED_TOTAL.load(Ordering::Relaxed)
        ),
    );
    push_line(&mut output, "# HELP merrow_live_orders_sent_total Total live orders sent");
    push_line(&mut output, "# TYPE merrow_live_orders_sent_total counter");
    push_line(
        &mut output,
        &format!(
            "merrow_live_orders_sent_total {}",
            LIVE_ORDERS_SENT_TOTAL.load(Ordering::Relaxed)
        ),
    );
    push_line(&mut output, "# HELP merrow_trigger_fire_total Trigger fires");
    push_line(&mut output, "# TYPE merrow_trigger_fire_total counter");
    push_line(
        &mut output,
        &format!(
            "merrow_trigger_fire_total {}",
            TRIGGER_FIRE_TOTAL.load(Ordering::Relaxed)
        ),
    );
    push_line(&mut output, "# HELP merrow_live_retry_total Live retries");
    push_line(&mut output, "# TYPE merrow_live_retry_total counter");
    push_line(
        &mut output,
        &format!(
            "merrow_live_retry_total {}",
            LIVE_RETRY_TOTAL.load(Ordering::Relaxed)
        ),
    );
    push_line(
        &mut output,
        "# HELP merrow_live_retry_exhausted_total Live retries exhausted",
    );
    push_line(&mut output, "# TYPE merrow_live_retry_exhausted_total counter");
    push_line(
        &mut output,
        &format!(
            "merrow_live_retry_exhausted_total {}",
            LIVE_RETRY_EXHAUSTED_TOTAL.load(Ordering::Relaxed)
        ),
    );
    push_line(&mut output, "# HELP merrow_errors_total Total errors");
    push_line(&mut output, "# TYPE merrow_errors_total counter");
    push_line(
        &mut output,
        &format!("merrow_errors_total {}", ERRORS_TOTAL.load(Ordering::Relaxed)),
    );
    push_line(
        &mut output,
        "# HELP merrow_last_run_timestamp Last run timestamp (epoch seconds)",
    );
    push_line(&mut output, "# TYPE merrow_last_run_timestamp gauge");
    push_line(
        &mut output,
        &format!("merrow_last_run_timestamp {}", last_ts),
    );
    push_line(
        &mut output,
        "# HELP merrow_last_run_mode_backtest Last run mode backtest (1/0)",
    );
    push_line(&mut output, "# TYPE merrow_last_run_mode_backtest gauge");
    push_line(
        &mut output,
        &format!(
            "merrow_last_run_mode_backtest {}",
            LAST_RUN_MODE_BACKTEST.load(Ordering::Relaxed)
        ),
    );
    push_line(
        &mut output,
        "# HELP merrow_last_run_mode_paper Last run mode paper (1/0)",
    );
    push_line(&mut output, "# TYPE merrow_last_run_mode_paper gauge");
    push_line(
        &mut output,
        &format!(
            "merrow_last_run_mode_paper {}",
            LAST_RUN_MODE_PAPER.load(Ordering::Relaxed)
        ),
    );
    push_line(
        &mut output,
        "# HELP merrow_last_run_mode_live Last run mode live (1/0)",
    );
    push_line(&mut output, "# TYPE merrow_last_run_mode_live gauge");
    push_line(
        &mut output,
        &format!(
            "merrow_last_run_mode_live {}",
            LAST_RUN_MODE_LIVE.load(Ordering::Relaxed)
        ),
    );
    push_line(&mut output, "# HELP merrow_last_return_rate Last return rate");
    push_line(&mut output, "# TYPE merrow_last_return_rate gauge");
    push_line(
        &mut output,
        &format!("merrow_last_return_rate {}", load_f64(&LAST_RETURN_RATE_BITS)),
    );
    push_line(
        &mut output,
        "# HELP merrow_last_max_drawdown Last max drawdown",
    );
    push_line(&mut output, "# TYPE merrow_last_max_drawdown gauge");
    push_line(
        &mut output,
        &format!(
            "merrow_last_max_drawdown {}",
            load_f64(&LAST_MAX_DRAWDOWN_BITS)
        ),
    );
    push_line(&mut output, "# HELP merrow_last_win_rate Last win rate");
    push_line(&mut output, "# TYPE merrow_last_win_rate gauge");
    push_line(
        &mut output,
        &format!("merrow_last_win_rate {}", load_f64(&LAST_WIN_RATE_BITS)),
    );
    push_line(&mut output, "# HELP merrow_last_sharpe Last sharpe");
    push_line(&mut output, "# TYPE merrow_last_sharpe gauge");
    push_line(
        &mut output,
        &format!("merrow_last_sharpe {}", load_f64(&LAST_SHARPE_BITS)),
    );
    output
}

fn set_last_metrics(metrics: &BacktestMetrics) {
    store_f64(&LAST_RETURN_RATE_BITS, metrics.return_rate);
    store_f64(&LAST_MAX_DRAWDOWN_BITS, metrics.max_drawdown);
    store_f64(&LAST_WIN_RATE_BITS, metrics.win_rate);
    store_f64(&LAST_SHARPE_BITS, metrics.sharpe);
}

fn set_last_run_mode(mode: &str) {
    let now = now_epoch() as u64;
    LAST_RUN_TIMESTAMP.store(now, Ordering::Relaxed);
    match mode {
        "backtest" => {
            LAST_RUN_MODE_BACKTEST.store(1, Ordering::Relaxed);
            LAST_RUN_MODE_PAPER.store(0, Ordering::Relaxed);
            LAST_RUN_MODE_LIVE.store(0, Ordering::Relaxed);
        }
        "paper" => {
            LAST_RUN_MODE_BACKTEST.store(0, Ordering::Relaxed);
            LAST_RUN_MODE_PAPER.store(1, Ordering::Relaxed);
            LAST_RUN_MODE_LIVE.store(0, Ordering::Relaxed);
        }
        "live" => {
            LAST_RUN_MODE_BACKTEST.store(0, Ordering::Relaxed);
            LAST_RUN_MODE_PAPER.store(0, Ordering::Relaxed);
            LAST_RUN_MODE_LIVE.store(1, Ordering::Relaxed);
        }
        _ => {}
    }
}

fn now_epoch() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

fn uptime_seconds() -> i64 {
    let start = START_TIME.get().copied().unwrap_or_else(now_epoch);
    now_epoch().saturating_sub(start)
}

fn store_f64(target: &AtomicU64, value: f64) {
    target.store(value.to_bits(), Ordering::Relaxed);
}

fn load_f64(source: &AtomicU64) -> f64 {
    f64::from_bits(source.load(Ordering::Relaxed))
}

fn push_line(target: &mut String, line: &str) {
    target.push_str(line);
    target.push('\n');
}
