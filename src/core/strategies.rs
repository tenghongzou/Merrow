use crate::models::{Candle, Signal};

use super::{strategy::Strategy, StrategyContext};

pub struct ThresholdStrategy {
    ma_window: usize,
    buy_threshold: f64,
    sell_threshold: f64,
}

impl ThresholdStrategy {
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

impl Strategy for ThresholdStrategy {
    fn on_tick(&mut self, ctx: &StrategyContext) -> Vec<Signal> {
        let price = ctx.candle.close;
        if price <= 0.0 {
            return vec![Signal::Hold];
        }
        let ma = match self.moving_average(ctx.history) {
            Some(value) if value > 0.0 => value,
            _ => return vec![Signal::Hold],
        };
        let buy_level = ma * (1.0 - self.buy_threshold);
        let sell_level = ma * (1.0 + self.sell_threshold);

        if price <= buy_level {
            vec![Signal::Buy]
        } else if price >= sell_level {
            vec![Signal::Sell]
        } else {
            vec![Signal::Hold]
        }
    }
}
