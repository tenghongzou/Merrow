use crate::models::Candle;

use super::trigger::Trigger;
use super::{TriggerContext, TriggerMode};

pub struct TimeTrigger {
    interval_minutes: u32,
}

impl TimeTrigger {
    pub fn new(interval_minutes: u32) -> Self {
        Self { interval_minutes }
    }
}

impl Trigger for TimeTrigger {
    fn should_fire(&self, ctx: &TriggerContext) -> bool {
        if self.interval_minutes == 0 {
            return false;
        }
        let interval_seconds = i64::from(self.interval_minutes) * 60;
        ctx.now % interval_seconds == 0
    }
}

pub struct PriceTrigger {
    ma_window: usize,
    buy_threshold: f64,
    sell_threshold: f64,
}

impl PriceTrigger {
    pub fn new(ma_window: usize, buy_threshold: f64, sell_threshold: f64) -> Self {
        Self {
            ma_window,
            buy_threshold,
            sell_threshold,
        }
    }

    fn moving_average(&self, history: &[Candle]) -> Option<f64> {
        if self.ma_window == 0 || history.len() < self.ma_window {
            return None;
        }
        let start = history.len() - self.ma_window;
        let sum: f64 = history[start..].iter().map(|candle| candle.close).sum();
        Some(sum / self.ma_window as f64)
    }
}

impl Trigger for PriceTrigger {
    fn should_fire(&self, ctx: &TriggerContext) -> bool {
        let price = ctx.candle.close;
        if price <= 0.0 {
            return false;
        }
        let ma = match self.moving_average(ctx.history) {
            Some(value) if value > 0.0 => value,
            _ => return false,
        };
        let buy_level = ma * (1.0 - self.buy_threshold);
        let sell_level = ma * (1.0 + self.sell_threshold);
        price <= buy_level || price >= sell_level
    }
}

pub struct TriggerEngine {
    mode: TriggerMode,
    triggers: Vec<Box<dyn Trigger>>,
}

impl TriggerEngine {
    pub fn new(mode: TriggerMode, triggers: Vec<Box<dyn Trigger>>) -> Self {
        Self { mode, triggers }
    }

    pub fn should_fire(&self, ctx: &TriggerContext) -> bool {
        if self.triggers.is_empty() {
            return false;
        }
        match self.mode {
            TriggerMode::Any => self.triggers.iter().any(|trigger| trigger.should_fire(ctx)),
            TriggerMode::All => self.triggers.iter().all(|trigger| trigger.should_fire(ctx)),
        }
    }
}
