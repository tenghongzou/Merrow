use merrow::backtest::fill::{fill_limit, fill_market, ExecutionCosts};
use merrow::models::{Candle, OrderRequest, OrderType, Side};

fn candle(open: f64) -> Candle {
    Candle {
        time: 1,
        open,
        high: open,
        low: open,
        close: open,
        volume: 1.0,
    }
}

#[test]
fn market_buy_applies_slippage_and_fee() {
    let order = OrderRequest {
        client_order_id: "c1".to_string(),
        symbol: "BTCUSDT".to_string(),
        side: Side::Buy,
        order_type: OrderType::Market,
        quantity: 2.0,
    };
    let costs = ExecutionCosts {
        fee_rate: 0.001,
        slippage_bps: 10,
    };
    let trade = fill_market(&order, &candle(100.0), costs).expect("trade");

    let expected_price = 100.0 * (1.0 + 0.001);
    let expected_fee = expected_price * 2.0 * 0.001;
    assert!((trade.price - expected_price).abs() < 1e-9);
    assert!((trade.fee - expected_fee).abs() < 1e-9);
}

#[test]
fn market_sell_applies_slippage() {
    let order = OrderRequest {
        client_order_id: "c2".to_string(),
        symbol: "BTCUSDT".to_string(),
        side: Side::Sell,
        order_type: OrderType::Market,
        quantity: 1.0,
    };
    let costs = ExecutionCosts {
        fee_rate: 0.0,
        slippage_bps: 10,
    };
    let trade = fill_market(&order, &candle(100.0), costs).expect("trade");

    let expected_price = 100.0 * (1.0 - 0.001);
    assert!((trade.price - expected_price).abs() < 1e-9);
}

#[test]
fn limit_orders_ignore_slippage() {
    let order = OrderRequest {
        client_order_id: "c3".to_string(),
        symbol: "BTCUSDT".to_string(),
        side: Side::Buy,
        order_type: OrderType::Limit { price: 100.0 },
        quantity: 1.0,
    };
    let costs = ExecutionCosts {
        fee_rate: 0.0,
        slippage_bps: 10,
    };
    let trade = fill_limit(&order, &candle(100.0), costs).expect("trade");
    assert!((trade.price - 100.0).abs() < 1e-9);
}
