use merrow::config::Config;

#[test]
fn time_trigger_requires_multiple_of_five() {
    let mut config = Config::default();
    config.triggers.time_minutes = 7;
    let result = config.validate();
    assert!(result.is_err());
}

#[test]
fn time_trigger_requires_upper_bound() {
    let mut config = Config::default();
    config.triggers.time_minutes = 105;
    let result = config.validate();
    assert!(result.is_err());
}

#[test]
fn must_enable_at_least_one_trigger() {
    let mut config = Config::default();
    config.triggers.time_enabled = false;
    config.triggers.price_enabled = false;
    let result = config.validate();
    assert!(result.is_err());
}

#[test]
fn ratios_accept_valid_range() {
    let mut config = Config::default();
    config.strategy.buy_cash_ratio = 0.1;
    config.strategy.sell_pos_ratio = 0.2;
    config.strategy.rebuy_cash_ratio = 0.3;
    let result = config.validate();
    assert!(result.is_ok());
}

#[test]
fn csv_source_requires_path() {
    let mut config = Config::default();
    config.data.source = "csv".to_string();
    config.data.csv_path = None;
    let result = config.validate();
    assert!(result.is_err());
}

#[test]
fn data_source_must_be_known() {
    let mut config = Config::default();
    config.data.source = "unknown".to_string();
    let result = config.validate();
    assert!(result.is_err());
}

#[test]
fn order_type_must_be_known() {
    let mut config = Config::default();
    config.orders.order_type = "unknown".to_string();
    let result = config.validate();
    assert!(result.is_err());
}

#[test]
fn backtest_requires_time_range() {
    let mut config = Config::default();
    config.mode = "backtest".to_string();
    config.backtest.start_time = None;
    config.backtest.end_time = None;
    let result = config.validate();
    assert!(result.is_err());
}

#[test]
fn engine_bundle_requires_valid_risk() {
    let mut config = Config::default();
    config.risk.max_trade_ratio = 2.0;
    let result = merrow::core::build_engine_bundle(&config);
    assert!(result.is_err());
}

#[test]
fn backtest_initial_cash_must_be_non_negative() {
    let mut config = Config::default();
    config.backtest.initial_cash = -1.0;
    let result = config.validate();
    assert!(result.is_err());
}

#[test]
fn output_format_must_be_known() {
    let mut config = Config::default();
    config.output.format = "xml".to_string();
    let result = config.validate();
    assert!(result.is_err());
}

#[test]
fn output_path_required_when_enabled() {
    let mut config = Config::default();
    config.output.format = "json".to_string();
    config.output.path = "   ".to_string();
    let result = config.validate();
    assert!(result.is_err());
}

#[test]
fn exchange_limit_must_be_positive() {
    let mut config = Config::default();
    config.data.source = "exchange".to_string();
    config.data.exchange_limit = Some(0);
    let result = config.validate();
    assert!(result.is_err());
}
