use merrow::backtest::engine::{BacktestEngine, BacktestOrder};
use merrow::models::{Candle, OrderRequest, OrderType, Side};

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
fn market_order_fills_on_next_candle_open() {
    let candles = vec![
        candle(1, 100.0, 105.0, 95.0, 102.0),
        candle(2, 110.0, 115.0, 108.0, 112.0),
    ];
    let order = OrderRequest {
        client_order_id: "o1".to_string(),
        symbol: "BTCUSDT".to_string(),
        side: Side::Buy,
        order_type: OrderType::Market,
        quantity: 1.0,
    };

    let engine = BacktestEngine;
    let trades = engine
        .run(
            &candles,
            vec![BacktestOrder {
                submit_index: 0,
                order,
            }],
        )
        .expect("run backtest");

    assert_eq!(trades.len(), 1);
    assert_eq!(trades[0].price, 110.0);
    assert_eq!(trades[0].time, 2);
}

#[test]
fn limit_order_fills_on_later_candle_when_crosses() {
    let candles = vec![
        candle(1, 100.0, 105.0, 99.0, 102.0),
        candle(2, 103.0, 106.0, 101.0, 104.0),
        candle(3, 100.0, 104.0, 90.0, 92.0),
    ];
    let order = OrderRequest {
        client_order_id: "o2".to_string(),
        symbol: "BTCUSDT".to_string(),
        side: Side::Buy,
        order_type: OrderType::Limit { price: 95.0 },
        quantity: 1.0,
    };

    let engine = BacktestEngine;
    let trades = engine
        .run(
            &candles,
            vec![BacktestOrder {
                submit_index: 0,
                order,
            }],
        )
        .expect("run backtest");

    assert_eq!(trades.len(), 1);
    assert_eq!(trades[0].price, 95.0);
    assert_eq!(trades[0].time, 3);
}

#[test]
fn market_order_on_last_candle_is_not_filled() {
    let candles = vec![candle(1, 100.0, 105.0, 95.0, 102.0)];
    let order = OrderRequest {
        client_order_id: "o3".to_string(),
        symbol: "BTCUSDT".to_string(),
        side: Side::Buy,
        order_type: OrderType::Market,
        quantity: 1.0,
    };

    let engine = BacktestEngine;
    let trades = engine
        .run(
            &candles,
            vec![BacktestOrder {
                submit_index: 0,
                order,
            }],
        )
        .expect("run backtest");

    assert_eq!(trades.len(), 0);
}

#[test]
fn order_submit_index_out_of_range_is_error() {
    let candles = vec![candle(1, 100.0, 105.0, 95.0, 102.0)];
    let order = OrderRequest {
        client_order_id: "o4".to_string(),
        symbol: "BTCUSDT".to_string(),
        side: Side::Buy,
        order_type: OrderType::Market,
        quantity: 1.0,
    };

    let engine = BacktestEngine;
    let result = engine.run(
        &candles,
        vec![BacktestOrder {
            submit_index: 2,
            order,
        }],
    );

    assert!(result.is_err());
}
