use merrow::backtest::fill::{fill_limit, ExecutionCosts};
use merrow::models::{Candle, OrderRequest, OrderType, Side};

fn sample_candle() -> Candle {
    Candle {
        time: 1,
        open: 100.0,
        high: 110.0,
        low: 90.0,
        close: 105.0,
        volume: 1.0,
    }
}

#[test]
fn limit_buy_fills_when_low_crosses() {
    let candle = sample_candle();
    let order = OrderRequest {
        client_order_id: "c1".to_string(),
        symbol: "BTCUSDT".to_string(),
        side: Side::Buy,
        order_type: OrderType::Limit { price: 95.0 },
        quantity: 1.0,
    };

    let trade = fill_limit(&order, &candle, ExecutionCosts::zero());
    assert!(trade.is_some());
}

#[test]
fn limit_sell_fills_when_high_crosses() {
    let candle = sample_candle();
    let order = OrderRequest {
        client_order_id: "c2".to_string(),
        symbol: "BTCUSDT".to_string(),
        side: Side::Sell,
        order_type: OrderType::Limit { price: 105.0 },
        quantity: 1.0,
    };

    let trade = fill_limit(&order, &candle, ExecutionCosts::zero());
    assert!(trade.is_some());
}
