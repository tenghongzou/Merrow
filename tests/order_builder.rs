use merrow::config::Config;
use merrow::core::{order_builder::OrderBuilder, StrategyContext};
use merrow::models::{Account, Candle, OrderType, Position, Side, Signal};

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

fn account_with_position(cash: f64, qty: f64) -> Account {
    Account {
        cash,
        positions: vec![Position {
            symbol: "BTCUSDT".to_string(),
            quantity: qty,
            avg_price: 100.0,
        }],
    }
}

#[test]
fn buy_signal_creates_market_order() {
    let mut config = Config::default();
    config.orders.order_type = "market".to_string();

    let account = account_with_position(1000.0, 0.0);
    let candle = candle_with_close(100.0);
    let history = vec![candle.clone()];
    let ctx = StrategyContext {
        candle: &candle,
        history: &history,
        account: &account,
        now: 1,
    };

    let mut builder = OrderBuilder::new();
    let orders = builder
        .build_for_signal(Signal::Buy, &ctx, &config)
        .expect("build orders");

    assert_eq!(orders.len(), 1);
    assert_eq!(orders[0].side, Side::Buy);
    assert!(matches!(orders[0].order_type, OrderType::Market));
    assert_eq!(orders[0].quantity, 1000.0 * 0.5 / 100.0);
}

#[test]
fn limit_order_applies_offset() {
    let mut config = Config::default();
    config.orders.order_type = "limit".to_string();
    config.orders.limit_price_offset_bps = 10;

    let account = account_with_position(1000.0, 0.0);
    let candle = candle_with_close(100.0);
    let history = vec![candle.clone()];
    let ctx = StrategyContext {
        candle: &candle,
        history: &history,
        account: &account,
        now: 1,
    };

    let mut builder = OrderBuilder::new();
    let orders = builder
        .build_for_signal(Signal::Buy, &ctx, &config)
        .expect("build orders");

    match orders[0].order_type {
        OrderType::Limit { price } => {
            let expected = 100.0 * (1.0 - 0.001);
            let delta = (price - expected).abs();
            assert!(delta < 1e-9);
        }
        _ => panic!("expected limit order"),
    }
}

#[test]
fn sell_signal_creates_sell_and_rebuy_orders() {
    let config = Config::default();
    let account = account_with_position(0.0, 10.0);
    let candle = candle_with_close(100.0);
    let history = vec![candle.clone()];
    let ctx = StrategyContext {
        candle: &candle,
        history: &history,
        account: &account,
        now: 1,
    };

    let mut builder = OrderBuilder::new();
    let orders = builder
        .build_for_signal(Signal::Sell, &ctx, &config)
        .expect("build orders");

    assert_eq!(orders.len(), 2);
    assert_eq!(orders[0].side, Side::Sell);
    assert_eq!(orders[0].quantity, 10.0 * 0.2);
    assert_eq!(orders[1].side, Side::Buy);
    assert_eq!(orders[1].quantity, 10.0 * 0.2 * 0.5);
}

#[test]
fn hold_signal_creates_no_orders() {
    let config = Config::default();
    let account = account_with_position(100.0, 1.0);
    let candle = candle_with_close(100.0);
    let history = vec![candle.clone()];
    let ctx = StrategyContext {
        candle: &candle,
        history: &history,
        account: &account,
        now: 1,
    };

    let mut builder = OrderBuilder::new();
    let orders = builder
        .build_for_signal(Signal::Hold, &ctx, &config)
        .expect("build orders");

    assert!(orders.is_empty());
}
