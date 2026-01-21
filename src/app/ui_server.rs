use crate::config::Config;
use crate::storage::postgres::PostgresStorage;
use crate::{Error, Result};
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Json;
use axum::Router;
use chrono::{DateTime, Utc};
use postgres::{Client, NoTls};
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::task::spawn_blocking;
use tower_http::cors::{Any, CorsLayer};

#[derive(Clone)]
struct UiState {
    config: Config,
    pg_dsn: Option<String>,
    metrics_path: Option<String>,
}

#[derive(Serialize)]
struct Health {
    status: &'static str,
    pg_enabled: bool,
    metrics_path: Option<String>,
    timestamp: i64,
}

#[derive(Serialize)]
struct TradeRow {
    time: String,
    symbol: String,
    side: String,
    price: f64,
    qty: f64,
    fee: f64,
}

#[derive(Serialize)]
struct OrderRow {
    time: String,
    symbol: String,
    side: String,
    order_type: String,
    price: Option<f64>,
    qty: f64,
    status: String,
}

#[derive(Serialize)]
struct PositionRow {
    time: String,
    symbol: String,
    qty: f64,
    avg_price: f64,
}

#[derive(Serialize)]
struct BalanceRow {
    time: String,
    asset: String,
    free: f64,
    locked: f64,
}

#[derive(Serialize)]
struct BacktestLatest {
    run_id: String,
    start_time: String,
    end_time: String,
    return_rate: f64,
    max_drawdown: f64,
    win_rate: f64,
    trade_count: i32,
    sharpe: Option<f64>,
}

#[derive(Serialize)]
struct Summary {
    config: serde_json::Value,
    pg_enabled: bool,
    metrics: HashMap<String, f64>,
    backtest: Option<BacktestLatest>,
    trades: Vec<TradeRow>,
    orders: Vec<OrderRow>,
    positions: Vec<PositionRow>,
    balances: Vec<BalanceRow>,
}

#[derive(Deserialize)]
struct LimitQuery {
    limit: Option<i64>,
}

pub async fn run(addr: &str, config_path: &str, pg_enabled: Option<bool>) -> Result<()> {
    let mut config = Config::load(config_path)?;
    config.validate()?;

    let pg_enabled = pg_enabled.unwrap_or_else(pg_env_enabled);
    let pg_dsn = if pg_enabled {
        Some(config.storage.postgres_dsn.clone())
    } else {
        None
    };
    let metrics_path = std::env::var("MERROW_METRICS_PATH").ok();

    let state = UiState {
        config,
        pg_dsn,
        metrics_path,
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/summary", get(summary))
        .route("/api/metrics", get(metrics))
        .route("/api/trades", get(trades))
        .route("/api/orders", get(orders))
        .route("/api/positions", get(positions))
        .route("/api/balances", get(balances))
        .route("/api/backtest/latest", get(backtest_latest))
        .with_state(Arc::new(state))
        .layer(cors);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|err| Error::new(format!("bind failed: {err}")))?;
    axum::serve(listener, app)
        .await
        .map_err(|err| Error::new(format!("server error: {err}")))?;
    Ok(())
}

async fn health(State(state): State<Arc<UiState>>) -> impl IntoResponse {
    let response = Health {
        status: "ok",
        pg_enabled: state.pg_dsn.is_some(),
        metrics_path: state.metrics_path.clone(),
        timestamp: now_epoch(),
    };
    Json(response)
}

async fn summary(State(state): State<Arc<UiState>>) -> impl IntoResponse {
    match build_summary(state).await {
        Ok(summary) => Json(summary).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.message).into_response(),
    }
}

async fn metrics(State(state): State<Arc<UiState>>) -> impl IntoResponse {
    match read_metrics_map(state.metrics_path.as_deref()) {
        Ok(map) => Json(map).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.message).into_response(),
    }
}

async fn trades(
    State(state): State<Arc<UiState>>,
    Query(query): Query<LimitQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(100).clamp(1, 500);
    match fetch_trades(state, limit).await {
        Ok(rows) => Json(rows).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.message).into_response(),
    }
}

async fn orders(
    State(state): State<Arc<UiState>>,
    Query(query): Query<LimitQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(100).clamp(1, 500);
    match fetch_orders(state, limit).await {
        Ok(rows) => Json(rows).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.message).into_response(),
    }
}

async fn positions(State(state): State<Arc<UiState>>) -> impl IntoResponse {
    match fetch_positions(state).await {
        Ok(rows) => Json(rows).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.message).into_response(),
    }
}

async fn balances(State(state): State<Arc<UiState>>) -> impl IntoResponse {
    match fetch_balances(state).await {
        Ok(rows) => Json(rows).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.message).into_response(),
    }
}

async fn backtest_latest(State(state): State<Arc<UiState>>) -> impl IntoResponse {
    match fetch_backtest_latest(state).await {
        Ok(row) => Json(row).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.message).into_response(),
    }
}

async fn build_summary(state: Arc<UiState>) -> Result<Summary> {
    let metrics = read_metrics_map(state.metrics_path.as_deref())?;
    let config = config_view(&state.config);
    let backtest = fetch_backtest_latest(state.clone()).await.ok();
    let trades = fetch_trades(state.clone(), 50).await.unwrap_or_default();
    let orders = fetch_orders(state.clone(), 50).await.unwrap_or_default();
    let positions = fetch_positions(state.clone()).await.unwrap_or_default();
    let balances = fetch_balances(state.clone()).await.unwrap_or_default();

    Ok(Summary {
        config,
        pg_enabled: state.pg_dsn.is_some(),
        metrics,
        backtest,
        trades,
        orders,
        positions,
        balances,
    })
}

fn config_view(config: &Config) -> serde_json::Value {
    json!({
        "mode": config.mode,
        "exchange": config.exchange,
        "symbol": config.symbol,
        "data": {
            "source": config.data.source,
            "candle_interval": config.data.candle_interval,
            "exchange_category": config.data.exchange_category,
        },
        "orders": {
            "order_type": config.orders.order_type,
            "fee_rate": config.orders.fee_rate,
            "slippage_bps": config.orders.slippage_bps,
        },
        "strategy": {
            "buy_cash_ratio": config.strategy.buy_cash_ratio,
            "sell_pos_ratio": config.strategy.sell_pos_ratio,
            "rebuy_cash_ratio": config.strategy.rebuy_cash_ratio,
        },
        "risk": {
            "max_trade_ratio": config.risk.max_trade_ratio,
            "min_cash_reserve_ratio": config.risk.min_cash_reserve_ratio,
            "max_position_value_ratio": config.risk.max_position_value_ratio,
        },
        "triggers": {
            "time_enabled": config.triggers.time_enabled,
            "time_minutes": config.triggers.time_minutes,
            "price_enabled": config.triggers.price_enabled,
            "trigger_mode_any": config.triggers.trigger_mode_any,
            "ma_window": config.triggers.ma_window,
            "buy_threshold": config.triggers.buy_threshold,
            "sell_threshold": config.triggers.sell_threshold,
        },
    })
}

fn pg_env_enabled() -> bool {
    match std::env::var("MERROW_PG_ENABLED") {
        Ok(value) => matches!(value.to_lowercase().as_str(), "1" | "true" | "yes"),
        Err(_) => false,
    }
}

fn read_metrics_map(path: Option<&str>) -> Result<HashMap<String, f64>> {
    let Some(path) = path else {
        return Ok(HashMap::new());
    };
    let text =
        std::fs::read_to_string(path).map_err(|err| Error::new(format!("metrics read failed: {err}")))?;
    Ok(parse_metrics(&text))
}

fn parse_metrics(text: &str) -> HashMap<String, f64> {
    let mut map = HashMap::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let mut parts = trimmed.split_whitespace();
        if let (Some(name), Some(value)) = (parts.next(), parts.next()) {
            if let Ok(parsed) = value.parse::<f64>() {
                map.insert(name.to_string(), parsed);
            }
        }
    }
    map
}

async fn fetch_trades(state: Arc<UiState>, limit: i64) -> Result<Vec<TradeRow>> {
    let Some(dsn) = state.pg_dsn.clone() else {
        return Ok(Vec::new());
    };
    spawn_blocking(move || {
        let mut client = connect(&dsn)?;
        let rows = client
            .query(
                "SELECT time, symbol, side, price, qty, fee FROM trades ORDER BY time DESC LIMIT $1",
                &[&limit],
            )
            .map_err(|err| Error::new(format!("query trades failed: {err}")))?;
        Ok(rows
            .into_iter()
            .map(|row| TradeRow {
                time: row.get::<_, DateTime<Utc>>(0).to_rfc3339(),
                symbol: row.get(1),
                side: row.get(2),
                price: row.get(3),
                qty: row.get(4),
                fee: row.get(5),
            })
            .collect())
    })
    .await
    .map_err(|err| Error::new(format!("join failed: {err}")))?
}

async fn fetch_orders(state: Arc<UiState>, limit: i64) -> Result<Vec<OrderRow>> {
    let Some(dsn) = state.pg_dsn.clone() else {
        return Ok(Vec::new());
    };
    spawn_blocking(move || {
        let mut client = connect(&dsn)?;
        let rows = client
            .query(
                "SELECT time, symbol, side, order_type, price, qty, status FROM orders ORDER BY time DESC LIMIT $1",
                &[&limit],
            )
            .map_err(|err| Error::new(format!("query orders failed: {err}")))?;
        Ok(rows
            .into_iter()
            .map(|row| OrderRow {
                time: row.get::<_, DateTime<Utc>>(0).to_rfc3339(),
                symbol: row.get(1),
                side: row.get(2),
                order_type: row.get(3),
                price: row.get(4),
                qty: row.get(5),
                status: row.get(6),
            })
            .collect())
    })
    .await
    .map_err(|err| Error::new(format!("join failed: {err}")))?
}

async fn fetch_positions(state: Arc<UiState>) -> Result<Vec<PositionRow>> {
    let Some(dsn) = state.pg_dsn.clone() else {
        return Ok(Vec::new());
    };
    spawn_blocking(move || {
        let mut client = connect(&dsn)?;
        let rows = client
            .query(
                "SELECT DISTINCT ON (symbol) time, symbol, qty, avg_price FROM positions ORDER BY symbol, time DESC",
                &[],
            )
            .map_err(|err| Error::new(format!("query positions failed: {err}")))?;
        Ok(rows
            .into_iter()
            .map(|row| PositionRow {
                time: row.get::<_, DateTime<Utc>>(0).to_rfc3339(),
                symbol: row.get(1),
                qty: row.get(2),
                avg_price: row.get(3),
            })
            .collect())
    })
    .await
    .map_err(|err| Error::new(format!("join failed: {err}")))?
}

async fn fetch_balances(state: Arc<UiState>) -> Result<Vec<BalanceRow>> {
    let Some(dsn) = state.pg_dsn.clone() else {
        return Ok(Vec::new());
    };
    spawn_blocking(move || {
        let mut client = connect(&dsn)?;
        let rows = client
            .query(
                "SELECT DISTINCT ON (asset) time, asset, free, locked FROM balances ORDER BY asset, time DESC",
                &[],
            )
            .map_err(|err| Error::new(format!("query balances failed: {err}")))?;
        Ok(rows
            .into_iter()
            .map(|row| BalanceRow {
                time: row.get::<_, DateTime<Utc>>(0).to_rfc3339(),
                asset: row.get(1),
                free: row.get(2),
                locked: row.get(3),
            })
            .collect())
    })
    .await
    .map_err(|err| Error::new(format!("join failed: {err}")))?
}

async fn fetch_backtest_latest(state: Arc<UiState>) -> Result<BacktestLatest> {
    let Some(dsn) = state.pg_dsn.clone() else {
        return Err(Error::new("pg disabled"));
    };
    spawn_blocking(move || {
        let mut client = connect(&dsn)?;
        let rows = client
            .query(
                "SELECT r.id, r.start_time, r.end_time, m.return, m.max_drawdown, m.win_rate, m.trade_count, m.sharpe \
                 FROM backtest_runs r JOIN backtest_metrics m ON r.id = m.run_id \
                 ORDER BY r.created_at DESC LIMIT 1",
                &[],
            )
            .map_err(|err| Error::new(format!("query backtest failed: {err}")))?;
        let row = rows
            .into_iter()
            .next()
            .ok_or_else(|| Error::new("no backtest rows"))?;
        Ok(BacktestLatest {
            run_id: row.get(0),
            start_time: row.get::<_, DateTime<Utc>>(1).to_rfc3339(),
            end_time: row.get::<_, DateTime<Utc>>(2).to_rfc3339(),
            return_rate: row.get(3),
            max_drawdown: row.get(4),
            win_rate: row.get(5),
            trade_count: row.get(6),
            sharpe: row.get(7),
        })
    })
    .await
    .map_err(|err| Error::new(format!("join failed: {err}")))?
}

fn connect(dsn: &str) -> Result<Client> {
    Client::connect(dsn, NoTls).map_err(|err| Error::new(format!("postgres connect failed: {err}")))
}

fn now_epoch() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}
