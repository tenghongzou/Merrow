pub mod order_router;
pub mod order_builder;
pub mod order_flow;
pub mod risk;
pub mod strategy;
pub mod trigger;
pub mod triggers;
pub mod strategies;

use crate::config::Config;
use crate::core::order_flow::OrderFlow;
use crate::core::risk::{RiskLimits, RiskManager};
use crate::core::strategies::ThresholdStrategy;
use crate::core::triggers::{PriceTrigger, TimeTrigger, TriggerEngine};
use crate::Result;

use crate::models::{Account, Candle};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TriggerMode {
    Any,
    All,
}

pub struct StrategyContext<'a> {
    pub candle: &'a Candle,
    pub history: &'a [Candle],
    pub account: &'a Account,
    pub now: i64,
}

pub struct TriggerContext<'a> {
    pub candle: &'a Candle,
    pub history: &'a [Candle],
    pub now: i64,
}

pub struct EngineBundle {
    pub trigger_engine: TriggerEngine,
    pub strategy: ThresholdStrategy,
    pub order_flow: OrderFlow,
}

pub fn build_engine_bundle(config: &Config) -> Result<EngineBundle> {
    let mut triggers: Vec<Box<dyn trigger::Trigger>> = Vec::new();
    if config.triggers.time_enabled {
        triggers.push(Box::new(TimeTrigger::new(config.triggers.time_minutes)));
    }
    if config.triggers.price_enabled {
        triggers.push(Box::new(PriceTrigger::new(
            config.triggers.ma_window as usize,
            config.triggers.buy_threshold,
            config.triggers.sell_threshold,
        )));
    }

    let mode = if config.triggers.trigger_mode_any {
        TriggerMode::Any
    } else {
        TriggerMode::All
    };
    let trigger_engine = TriggerEngine::new(mode, triggers);

    let strategy = ThresholdStrategy::new(
        config.triggers.ma_window as usize,
        config.triggers.buy_threshold,
        config.triggers.sell_threshold,
    );

    let limits = RiskLimits {
        max_trade_ratio: config.risk.max_trade_ratio,
        min_cash_reserve_ratio: config.risk.min_cash_reserve_ratio,
        max_position_value_ratio: config.risk.max_position_value_ratio,
    };
    let risk = RiskManager::new(limits)?;
    let order_flow = OrderFlow::new(risk);

    Ok(EngineBundle {
        trigger_engine,
        strategy,
        order_flow,
    })
}
