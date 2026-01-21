use merrow::config::Config;
use merrow::core::{order_flow::OrderFlow, risk::RiskLimits, risk::RiskManager, StrategyContext};
use merrow::models::{Account, Candle, Position, Signal};

fn candle_with_close(price: f64) -> Candle {
    Candle {
        time: 1,
        open: price,
        high: price,
        low: price,
        close: price,
        volume: 1.0,
    }
}

#[test]
fn risk_blocks_order_exceeding_max_trade_ratio() {
    let mut config = Config::default();
    config.orders.order_type = "market".to_string();
    config.strategy.buy_cash_ratio = 0.5;

    let limits = RiskLimits {
        max_trade_ratio: 0.4,
        min_cash_reserve_ratio: 0.0,
        max_position_value_ratio: 1.0,
    };
    let risk = RiskManager::new(limits).expect("risk manager");
    let mut flow = OrderFlow::new(risk);

    let account = Account {
        cash: 1000.0,
        positions: Vec::new(),
    };
    let candle = candle_with_close(100.0);
    let history = vec![candle.clone()];
    let ctx = StrategyContext {
        candle: &candle,
        history: &history,
        account: &account,
        now: 1,
    };

    let result = flow.plan(Signal::Buy, &ctx, &config);
    assert!(result.is_err());
}

#[test]
fn risk_allows_order_within_limits() {
    let mut config = Config::default();
    config.orders.order_type = "market".to_string();
    config.strategy.buy_cash_ratio = 0.5;

    let limits = RiskLimits {
        max_trade_ratio: 0.6,
        min_cash_reserve_ratio: 0.0,
        max_position_value_ratio: 1.0,
    };
    let risk = RiskManager::new(limits).expect("risk manager");
    let mut flow = OrderFlow::new(risk);

    let account = Account {
        cash: 1000.0,
        positions: Vec::new(),
    };
    let candle = candle_with_close(100.0);
    let history = vec![candle.clone()];
    let ctx = StrategyContext {
        candle: &candle,
        history: &history,
        account: &account,
        now: 1,
    };

    let result = flow.plan(Signal::Buy, &ctx, &config);
    assert!(result.is_ok());
}

#[test]
fn sell_signal_passes_risk_checks() {
    let mut config = Config::default();
    config.orders.order_type = "market".to_string();
    let limits = RiskLimits {
        max_trade_ratio: 1.0,
        min_cash_reserve_ratio: 0.0,
        max_position_value_ratio: 1.0,
    };
    let risk = RiskManager::new(limits).expect("risk manager");
    let mut flow = OrderFlow::new(risk);

    let account = Account {
        cash: 0.0,
        positions: vec![Position {
            symbol: "BTCUSDT".to_string(),
            quantity: 10.0,
            avg_price: 100.0,
        }],
    };
    let candle = candle_with_close(100.0);
    let history = vec![candle.clone()];
    let ctx = StrategyContext {
        candle: &candle,
        history: &history,
        account: &account,
        now: 1,
    };

    let result = flow.plan(Signal::Sell, &ctx, &config);
    assert!(result.is_ok());
}
