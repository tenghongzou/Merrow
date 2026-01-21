use crate::models::Signal;

use super::StrategyContext;

pub trait Strategy {
    fn on_tick(&mut self, ctx: &StrategyContext) -> Vec<Signal>;
}
