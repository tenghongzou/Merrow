use merrow::core::trigger::Trigger;
use merrow::core::{triggers::PriceTrigger, triggers::TimeTrigger, triggers::TriggerEngine};
use merrow::core::{TriggerContext, TriggerMode};
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
fn time_trigger_fires_on_interval() {
    let trigger = TimeTrigger::new(10);
    let candles = vec![candle(600, 100.0)];
    let ctx = TriggerContext {
        candle: &candles[0],
        history: &candles,
        now: 600,
    };
    assert!(trigger.should_fire(&ctx));
}

#[test]
fn time_trigger_does_not_fire_off_interval() {
    let trigger = TimeTrigger::new(10);
    let candles = vec![candle(601, 100.0)];
    let ctx = TriggerContext {
        candle: &candles[0],
        history: &candles,
        now: 601,
    };
    assert!(!trigger.should_fire(&ctx));
}

#[test]
fn price_trigger_fires_on_buy_threshold() {
    let trigger = PriceTrigger::new(5, 0.05, 0.05);
    let mut candles = Vec::new();
    for time in 1..=5 {
        candles.push(candle(time, 100.0));
    }
    candles.push(candle(6, 90.0));
    let ctx = TriggerContext {
        candle: &candles[5],
        history: &candles,
        now: 6,
    };
    assert!(trigger.should_fire(&ctx));
}

#[test]
fn price_trigger_fires_on_sell_threshold() {
    let trigger = PriceTrigger::new(5, 0.05, 0.05);
    let mut candles = Vec::new();
    for time in 1..=5 {
        candles.push(candle(time, 100.0));
    }
    candles.push(candle(6, 110.0));
    let ctx = TriggerContext {
        candle: &candles[5],
        history: &candles,
        now: 6,
    };
    assert!(trigger.should_fire(&ctx));
}

#[test]
fn price_trigger_requires_history() {
    let trigger = PriceTrigger::new(5, 0.05, 0.05);
    let candles = vec![candle(1, 100.0), candle(2, 99.0)];
    let ctx = TriggerContext {
        candle: &candles[1],
        history: &candles,
        now: 2,
    };
    assert!(!trigger.should_fire(&ctx));
}

#[test]
fn trigger_engine_any_mode() {
    let time_trigger = Box::new(TimeTrigger::new(10));
    let price_trigger = Box::new(PriceTrigger::new(5, 0.05, 0.05));
    let engine = TriggerEngine::new(TriggerMode::Any, vec![time_trigger, price_trigger]);

    let mut candles = Vec::new();
    for time in 1..=5 {
        candles.push(candle(time, 100.0));
    }
    candles.push(candle(6, 90.0));
    let ctx = TriggerContext {
        candle: &candles[5],
        history: &candles,
        now: 601,
    };

    assert!(engine.should_fire(&ctx));
}

#[test]
fn trigger_engine_all_mode() {
    let time_trigger = Box::new(TimeTrigger::new(10));
    let price_trigger = Box::new(PriceTrigger::new(5, 0.05, 0.05));
    let engine = TriggerEngine::new(TriggerMode::All, vec![time_trigger, price_trigger]);

    let mut candles = Vec::new();
    for time in 1..=5 {
        candles.push(candle(time, 100.0));
    }
    candles.push(candle(6, 90.0));
    let ctx = TriggerContext {
        candle: &candles[5],
        history: &candles,
        now: 601,
    };

    assert!(!engine.should_fire(&ctx));
}
