use crate::{Error, Result};
use serde::Deserialize;
use std::env;
use std::fs;

#[derive(Clone, Debug)]
pub struct TriggerConfig {
    pub time_enabled: bool,
    pub time_minutes: u32,
    pub price_enabled: bool,
    pub trigger_mode_any: bool,
    pub ma_window: u32,
    pub buy_threshold: f64,
    pub sell_threshold: f64,
}

#[derive(Clone, Debug)]
pub struct StrategyConfig {
    pub buy_cash_ratio: f64,
    pub sell_pos_ratio: f64,
    pub rebuy_cash_ratio: f64,
}

#[derive(Clone, Debug)]
pub struct RiskConfig {
    pub max_trade_ratio: f64,
    pub min_cash_reserve_ratio: f64,
    pub max_position_value_ratio: f64,
}

#[derive(Clone, Debug)]
pub struct OrderConfig {
    pub order_type: String,
    pub limit_price_offset_bps: u32,
    pub fee_rate: f64,
    pub slippage_bps: u32,
}

#[derive(Clone, Debug)]
pub struct BacktestConfig {
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub initial_cash: f64,
}

#[derive(Clone, Debug)]
pub struct OutputConfig {
    pub format: String,
    pub path: String,
}

#[derive(Clone, Debug)]
pub struct DataConfig {
    pub source: String,
    pub csv_path: Option<String>,
    pub candle_interval: String,
    pub exchange_base_url: Option<String>,
    pub exchange_limit: Option<u32>,
    pub exchange_category: Option<String>,
}

#[derive(Clone, Debug)]
pub struct StorageConfig {
    pub postgres_dsn: String,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub mode: String,
    pub exchange: String,
    pub symbol: String,
    pub orders: OrderConfig,
    pub triggers: TriggerConfig,
    pub strategy: StrategyConfig,
    pub risk: RiskConfig,
    pub backtest: BacktestConfig,
    pub output: OutputConfig,
    pub data: DataConfig,
    pub storage: StorageConfig,
}

#[derive(Clone, Debug, Deserialize)]
struct TriggerConfigFile {
    time_enabled: Option<bool>,
    time_minutes: Option<u32>,
    price_enabled: Option<bool>,
    trigger_mode_any: Option<bool>,
    ma_window: Option<u32>,
    buy_threshold: Option<f64>,
    sell_threshold: Option<f64>,
}

#[derive(Clone, Debug, Deserialize)]
struct StrategyConfigFile {
    buy_cash_ratio: Option<f64>,
    sell_pos_ratio: Option<f64>,
    rebuy_cash_ratio: Option<f64>,
}

#[derive(Clone, Debug, Deserialize)]
struct RiskConfigFile {
    max_trade_ratio: Option<f64>,
    min_cash_reserve_ratio: Option<f64>,
    max_position_value_ratio: Option<f64>,
}

#[derive(Clone, Debug, Deserialize)]
struct OrderConfigFile {
    order_type: Option<String>,
    limit_price_offset_bps: Option<u32>,
    fee_rate: Option<f64>,
    slippage_bps: Option<u32>,
}

#[derive(Clone, Debug, Deserialize)]
struct BacktestConfigFile {
    start_time: Option<String>,
    end_time: Option<String>,
    initial_cash: Option<f64>,
}

#[derive(Clone, Debug, Deserialize)]
struct OutputConfigFile {
    format: Option<String>,
    path: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct DataConfigFile {
    source: Option<String>,
    csv_path: Option<String>,
    candle_interval: Option<String>,
    exchange_base_url: Option<String>,
    exchange_limit: Option<u32>,
    exchange_category: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct StorageConfigFile {
    postgres_dsn: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct ConfigFile {
    mode: Option<String>,
    exchange: Option<String>,
    symbol: Option<String>,
    orders: Option<OrderConfigFile>,
    triggers: Option<TriggerConfigFile>,
    strategy: Option<StrategyConfigFile>,
    risk: Option<RiskConfigFile>,
    backtest: Option<BacktestConfigFile>,
    output: Option<OutputConfigFile>,
    data: Option<DataConfigFile>,
    storage: Option<StorageConfigFile>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mode: "backtest".to_string(),
            exchange: "binance".to_string(),
            symbol: "BTCUSDT".to_string(),
            orders: OrderConfig {
                order_type: "limit".to_string(),
                limit_price_offset_bps: 10,
                fee_rate: 0.001,
                slippage_bps: 5,
            },
            triggers: TriggerConfig {
                time_enabled: true,
                time_minutes: 15,
                price_enabled: true,
                trigger_mode_any: true,
                ma_window: 20,
                buy_threshold: 0.02,
                sell_threshold: 0.03,
            },
            strategy: StrategyConfig {
                buy_cash_ratio: 0.5,
                sell_pos_ratio: 0.2,
                rebuy_cash_ratio: 0.5,
            },
            risk: RiskConfig {
                max_trade_ratio: 0.5,
                min_cash_reserve_ratio: 0.05,
                max_position_value_ratio: 0.8,
            },
            backtest: BacktestConfig {
                start_time: Some("2024-01-01T00:00:00Z".to_string()),
                end_time: Some("2024-02-01T00:00:00Z".to_string()),
                initial_cash: 10_000.0,
            },
            output: OutputConfig {
                format: "none".to_string(),
                path: "output/backtest_report.json".to_string(),
            },
            data: DataConfig {
                source: "csv".to_string(),
                csv_path: Some("data/BTCUSDT_1m.csv".to_string()),
                candle_interval: "1m".to_string(),
                exchange_base_url: None,
                exchange_limit: None,
                exchange_category: None,
            },
            storage: StorageConfig {
                postgres_dsn: "postgres://user:pass@localhost:5432/merrow".to_string(),
            },
        }
    }
}

impl Config {
    pub fn load(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)
            .map_err(|err| Error::new(format!("failed to read config: {err}")))?;
        let file: ConfigFile = toml::from_str(&content)
            .map_err(|err| Error::new(format!("failed to parse config: {err}")))?;
        let mut config = Config::from_file(file);
        config.apply_env_overrides()?;
        config.validate()?;
        Ok(config)
    }

    fn from_file(file: ConfigFile) -> Self {
        let mut config = Config::default();

        if let Some(mode) = file.mode {
            config.mode = mode;
        }
        if let Some(exchange) = file.exchange {
            config.exchange = exchange;
        }
        if let Some(symbol) = file.symbol {
            config.symbol = symbol;
        }

        if let Some(orders) = file.orders {
            if let Some(value) = orders.order_type {
                config.orders.order_type = value;
            }
            if let Some(value) = orders.limit_price_offset_bps {
                config.orders.limit_price_offset_bps = value;
            }
            if let Some(value) = orders.fee_rate {
                config.orders.fee_rate = value;
            }
            if let Some(value) = orders.slippage_bps {
                config.orders.slippage_bps = value;
            }
        }

        if let Some(triggers) = file.triggers {
            if let Some(value) = triggers.time_enabled {
                config.triggers.time_enabled = value;
            }
            if let Some(value) = triggers.time_minutes {
                config.triggers.time_minutes = value;
            }
            if let Some(value) = triggers.price_enabled {
                config.triggers.price_enabled = value;
            }
            if let Some(value) = triggers.trigger_mode_any {
                config.triggers.trigger_mode_any = value;
            }
            if let Some(value) = triggers.ma_window {
                config.triggers.ma_window = value;
            }
            if let Some(value) = triggers.buy_threshold {
                config.triggers.buy_threshold = value;
            }
            if let Some(value) = triggers.sell_threshold {
                config.triggers.sell_threshold = value;
            }
        }

        if let Some(strategy) = file.strategy {
            if let Some(value) = strategy.buy_cash_ratio {
                config.strategy.buy_cash_ratio = value;
            }
            if let Some(value) = strategy.sell_pos_ratio {
                config.strategy.sell_pos_ratio = value;
            }
            if let Some(value) = strategy.rebuy_cash_ratio {
                config.strategy.rebuy_cash_ratio = value;
            }
        }

        if let Some(risk) = file.risk {
            if let Some(value) = risk.max_trade_ratio {
                config.risk.max_trade_ratio = value;
            }
            if let Some(value) = risk.min_cash_reserve_ratio {
                config.risk.min_cash_reserve_ratio = value;
            }
            if let Some(value) = risk.max_position_value_ratio {
                config.risk.max_position_value_ratio = value;
            }
        }

        if let Some(backtest) = file.backtest {
            if let Some(value) = backtest.start_time {
                config.backtest.start_time = Some(value);
            }
            if let Some(value) = backtest.end_time {
                config.backtest.end_time = Some(value);
            }
            if let Some(value) = backtest.initial_cash {
                config.backtest.initial_cash = value;
            }
        }

        if let Some(output) = file.output {
            if let Some(value) = output.format {
                config.output.format = value;
            }
            if let Some(value) = output.path {
                config.output.path = value;
            }
        }

        if let Some(data) = file.data {
            if let Some(value) = data.source {
                config.data.source = value;
            }
            if let Some(value) = data.csv_path {
                config.data.csv_path = Some(value);
            }
            if let Some(value) = data.candle_interval {
                config.data.candle_interval = value;
            }
            if let Some(value) = data.exchange_base_url {
                config.data.exchange_base_url = Some(value);
            }
            if let Some(value) = data.exchange_limit {
                config.data.exchange_limit = Some(value);
            }
            if let Some(value) = data.exchange_category {
                config.data.exchange_category = Some(value);
            }
        }

        if let Some(storage) = file.storage {
            if let Some(value) = storage.postgres_dsn {
                config.storage.postgres_dsn = value;
            }
        }

        config
    }

    pub fn apply_env_overrides(&mut self) -> Result<()> {
        if let Some(value) = env::var("MERROW_MODE").ok() {
            self.mode = value;
        }
        if let Some(value) = env::var("MERROW_EXCHANGE").ok() {
            self.exchange = value;
        }
        if let Some(value) = env::var("MERROW_SYMBOL").ok() {
            self.symbol = value;
        }

        if let Some(value) = read_string_env("MERROW_ORDER_TYPE")? {
            self.orders.order_type = value;
        }
        if let Some(value) = read_u32_env("MERROW_LIMIT_PRICE_OFFSET_BPS")? {
            self.orders.limit_price_offset_bps = value;
        }
        if let Some(value) = read_f64_env("MERROW_FEE_RATE")? {
            self.orders.fee_rate = value;
        }
        if let Some(value) = read_u32_env("MERROW_SLIPPAGE_BPS")? {
            self.orders.slippage_bps = value;
        }

        if let Some(value) = read_bool_env("MERROW_TIME_TRIGGER_ENABLED")? {
            self.triggers.time_enabled = value;
        }
        if let Some(value) = read_u32_env("MERROW_TIME_TRIGGER_MINUTES")? {
            self.triggers.time_minutes = value;
        }
        if let Some(value) = read_bool_env("MERROW_PRICE_TRIGGER_ENABLED")? {
            self.triggers.price_enabled = value;
        }
        if let Some(value) = read_string_env("MERROW_TRIGGER_MODE")? {
            self.triggers.trigger_mode_any = match value.as_str() {
                "any" => true,
                "all" => false,
                _ => {
                    return Err(Error::new(
                        "MERROW_TRIGGER_MODE must be 'any' or 'all'",
                    ))
                }
            };
        }
        if let Some(value) = read_u32_env("MERROW_MA_WINDOW")? {
            self.triggers.ma_window = value;
        }
        if let Some(value) = read_f64_env("MERROW_BUY_THRESHOLD")? {
            self.triggers.buy_threshold = value;
        }
        if let Some(value) = read_f64_env("MERROW_SELL_THRESHOLD")? {
            self.triggers.sell_threshold = value;
        }

        if let Some(value) = read_f64_env("MERROW_BUY_CASH_RATIO")? {
            self.strategy.buy_cash_ratio = value;
        }
        if let Some(value) = read_f64_env("MERROW_SELL_POS_RATIO")? {
            self.strategy.sell_pos_ratio = value;
        }
        if let Some(value) = read_f64_env("MERROW_REBUY_CASH_RATIO")? {
            self.strategy.rebuy_cash_ratio = value;
        }

        if let Some(value) = read_f64_env("MERROW_RISK_MAX_TRADE_RATIO")? {
            self.risk.max_trade_ratio = value;
        }
        if let Some(value) = read_f64_env("MERROW_RISK_MIN_CASH_RESERVE_RATIO")? {
            self.risk.min_cash_reserve_ratio = value;
        }
        if let Some(value) = read_f64_env("MERROW_RISK_MAX_POSITION_VALUE_RATIO")? {
            self.risk.max_position_value_ratio = value;
        }

        if let Some(value) = read_string_env("MERROW_BACKTEST_START_TIME")? {
            self.backtest.start_time = Some(value);
        }
        if let Some(value) = read_string_env("MERROW_BACKTEST_END_TIME")? {
            self.backtest.end_time = Some(value);
        }
        if let Some(value) = read_f64_env("MERROW_BACKTEST_INITIAL_CASH")? {
            self.backtest.initial_cash = value;
        }

        if let Some(value) = read_string_env("MERROW_OUTPUT_FORMAT")? {
            self.output.format = value;
        }
        if let Some(value) = read_string_env("MERROW_OUTPUT_PATH")? {
            self.output.path = value;
        }

        if let Some(value) = read_string_env("MERROW_DATA_SOURCE")? {
            self.data.source = value;
        }
        if let Some(value) = read_string_env("MERROW_CSV_PATH")? {
            self.data.csv_path = Some(value);
        }
        if let Some(value) = read_string_env("MERROW_CANDLE_INTERVAL")? {
            self.data.candle_interval = value;
        }
        if let Some(value) = read_string_env("MERROW_EXCHANGE_BASE_URL")? {
            self.data.exchange_base_url = Some(value);
        }
        if let Some(value) = read_u32_env("MERROW_EXCHANGE_LIMIT")? {
            self.data.exchange_limit = Some(value);
        }
        if let Some(value) = read_string_env("MERROW_EXCHANGE_CATEGORY")? {
            self.data.exchange_category = Some(value);
        }
        if let Some(value) = read_string_env("MERROW_POSTGRES_DSN")? {
            self.storage.postgres_dsn = value;
        }

        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        if !matches!(self.mode.as_str(), "backtest" | "paper" | "live") {
            return Err(Error::new("mode must be backtest, paper, or live"));
        }
        if self.exchange.trim().is_empty() {
            return Err(Error::new("exchange must be set"));
        }

        match self.orders.order_type.as_str() {
            "market" | "limit" => {}
            _ => return Err(Error::new("orders.order_type must be market or limit")),
        }
        if self.orders.fee_rate < 0.0 {
            return Err(Error::new("orders.fee_rate must be non-negative"));
        }

        let time_minutes = self.triggers.time_minutes;
        if self.triggers.time_enabled {
            if time_minutes == 0 || time_minutes > 100 || time_minutes % 5 != 0 {
                return Err(Error::new(
                    "time_minutes must be a multiple of 5 and <= 100",
                ));
            }
        }

        if !self.triggers.time_enabled && !self.triggers.price_enabled {
            return Err(Error::new("at least one trigger must be enabled"));
        }

        for (name, value) in [
            ("buy_cash_ratio", self.strategy.buy_cash_ratio),
            ("sell_pos_ratio", self.strategy.sell_pos_ratio),
            ("rebuy_cash_ratio", self.strategy.rebuy_cash_ratio),
        ] {
            if !(0.0..=1.0).contains(&value) {
                return Err(Error::new(format!("{name} must be in [0, 1]")));
            }
        }

        for (name, value) in [
            ("max_trade_ratio", self.risk.max_trade_ratio),
            ("min_cash_reserve_ratio", self.risk.min_cash_reserve_ratio),
            ("max_position_value_ratio", self.risk.max_position_value_ratio),
        ] {
            if !(0.0..=1.0).contains(&value) {
                return Err(Error::new(format!("{name} must be in [0, 1]")));
            }
        }

        if self.mode == "backtest" {
            if self.backtest.start_time.is_none() || self.backtest.end_time.is_none() {
                return Err(Error::new("backtest.start_time and backtest.end_time must be set"));
            }
            if self.backtest.initial_cash < 0.0 {
                return Err(Error::new("backtest.initial_cash must be non-negative"));
            }
        }

        match self.output.format.as_str() {
            "none" | "json" | "csv" => {}
            _ => return Err(Error::new("output.format must be none, json, or csv")),
        }
        if self.output.format != "none" && self.output.path.trim().is_empty() {
            return Err(Error::new("output.path must be set"));
        }

        let source = self.data.source.as_str();
        if source != "csv" && source != "exchange" {
            return Err(Error::new("data.source must be csv or exchange"));
        }
        if self.data.candle_interval.trim().is_empty() {
            return Err(Error::new("data.candle_interval must be set"));
        }
        if source == "csv" {
            match &self.data.csv_path {
                Some(path) if !path.trim().is_empty() => {}
                _ => return Err(Error::new("data.csv_path must be set for csv source")),
            }
        }
        if source == "exchange" {
            if self.exchange.trim().is_empty() {
                return Err(Error::new("exchange must be set for exchange source"));
            }
            if let Some(limit) = self.data.exchange_limit {
                if limit == 0 {
                    return Err(Error::new("data.exchange_limit must be positive"));
                }
            }
            if let Some(url) = &self.data.exchange_base_url {
                if url.trim().is_empty() {
                    return Err(Error::new("data.exchange_base_url must be non-empty"));
                }
            }
            if let Some(category) = &self.data.exchange_category {
                if category.trim().is_empty() {
                    return Err(Error::new("data.exchange_category must be non-empty"));
                }
            }
        }
        if self.storage.postgres_dsn.trim().is_empty() {
            return Err(Error::new("storage.postgres_dsn must be set"));
        }

        Ok(())
    }
}

fn read_string_env(key: &str) -> Result<Option<String>> {
    match env::var(key) {
        Ok(value) => Ok(Some(value)),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(err) => Err(Error::new(format!("failed to read {key}: {err}"))),
    }
}

fn read_bool_env(key: &str) -> Result<Option<bool>> {
    match env::var(key) {
        Ok(value) => match value.to_lowercase().as_str() {
            "true" | "1" | "yes" => Ok(Some(true)),
            "false" | "0" | "no" => Ok(Some(false)),
            _ => Err(Error::new(format!("{key} must be a boolean"))),
        },
        Err(env::VarError::NotPresent) => Ok(None),
        Err(err) => Err(Error::new(format!("failed to read {key}: {err}"))),
    }
}

fn read_u32_env(key: &str) -> Result<Option<u32>> {
    match env::var(key) {
        Ok(value) => value
            .parse::<u32>()
            .map(Some)
            .map_err(|err| Error::new(format!("{key} must be u32: {err}"))),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(err) => Err(Error::new(format!("failed to read {key}: {err}"))),
    }
}

fn read_f64_env(key: &str) -> Result<Option<f64>> {
    match env::var(key) {
        Ok(value) => value
            .parse::<f64>()
            .map(Some)
            .map_err(|err| Error::new(format!("{key} must be f64: {err}"))),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(err) => Err(Error::new(format!("failed to read {key}: {err}"))),
    }
}
