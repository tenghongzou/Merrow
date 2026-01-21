use merrow::backtest::BacktestEngine;
use merrow::config::Config;
use merrow::core::order_flow::OrderFlow;
use merrow::core::risk::{RiskLimits, RiskManager};
use merrow::core::strategies::ThresholdStrategy;
use merrow::core::triggers::{PriceTrigger, TriggerEngine};
use merrow::core::TriggerMode;
use merrow::models::Candle;

fn candle(time: i64, open: f64, high: f64, low: f64, close: f64) -> Candle {
    Candle {
        time,
        open,
        high,
        low,
        close,
        volume: 1.0,
    }
}

#[test]
fn trigger_strategy_order_flow_creates_trade() {
    let mut config = Config::default();
    config.orders.order_type = "market".to_string();
    config.orders.slippage_bps = 0;
    config.orders.fee_rate = 0.0;
    config.triggers.price_enabled = true;
    config.triggers.time_enabled = false;

    let candles = vec![
        candle(1, 100.0, 100.0, 100.0, 100.0),
        candle(2, 100.0, 100.0, 100.0, 100.0),
        candle(3, 100.0, 100.0, 100.0, 100.0),
        candle(4, 90.0, 95.0, 89.0, 90.0),
        candle(5, 95.0, 97.0, 94.0, 96.0),
    ];

    let trigger = PriceTrigger::new(3, 0.05, 0.05);
    let trigger_engine = TriggerEngine::new(TriggerMode::Any, vec![Box::new(trigger)]);
    let mut strategy = ThresholdStrategy::new(3, 0.05, 0.05);

    let limits = RiskLimits {
        max_trade_ratio: 1.0,
        min_cash_reserve_ratio: 0.0,
        max_position_value_ratio: 1.0,
    };
    let risk = RiskManager::new(limits).expect("risk manager");
    let mut order_flow = OrderFlow::new(risk);

    let engine = BacktestEngine;
    let result = engine
        .run_strategy(
            &candles,
            &config,
            &trigger_engine,
            &mut strategy,
            &mut order_flow,
            1000.0,
        )
        .expect("run strategy");

    assert_eq!(result.trades.len(), 1);
    assert_eq!(result.trades[0].price, 95.0);
    assert!(result.account.cash < 1000.0);
}
