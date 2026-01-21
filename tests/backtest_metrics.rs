use merrow::backtest::BacktestEngine;
use merrow::config::Config;
use merrow::core::order_flow::OrderFlow;
use merrow::core::risk::{RiskLimits, RiskManager};
use merrow::core::strategies::ThresholdStrategy;
use merrow::core::triggers::{TimeTrigger, TriggerEngine};
use merrow::core::TriggerMode;
use merrow::models::Candle;

fn candle(time: i64, close: f64) -> Candle {
    Candle {
        time,
        open: close,
        high: close,
        low: close,
        close,
        volume: 1.0,
    }
}

#[test]
fn metrics_are_zero_when_no_trades() {
    let mut config = Config::default();
    config.orders.order_type = "market".to_string();
    config.triggers.time_enabled = true;
    config.triggers.time_minutes = 5;
    config.triggers.price_enabled = false;
    config.triggers.buy_threshold = 0.5;
    config.triggers.sell_threshold = 0.5;
    config.orders.fee_rate = 0.0;
    config.orders.slippage_bps = 0;

    let candles = vec![candle(300, 100.0), candle(600, 100.0)];
    let trigger = TimeTrigger::new(5);
    let trigger_engine = TriggerEngine::new(TriggerMode::Any, vec![Box::new(trigger)]);
    let mut strategy = ThresholdStrategy::new(2, 0.5, 0.5);

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

    assert_eq!(result.trades.len(), 0);
    assert_eq!(result.metrics.trade_count, 0);
    assert_eq!(result.metrics.return_rate, 0.0);
    assert_eq!(result.metrics.max_drawdown, 0.0);
    assert_eq!(result.metrics.win_rate, 0.0);
}
