use merrow::exchange::{sync::sync_account, CandleRequest, Exchange};
use merrow::models::{Balance, Candle, OrderAck, OrderRequest, OrderStatus, Position};
use merrow::Result;

struct MockExchange;

impl Exchange for MockExchange {
    fn place_order(&self, _order: &OrderRequest) -> Result<OrderAck> {
        Ok(OrderAck {
            client_order_id: "c1".to_string(),
            exchange_order_id: None,
            status: OrderStatus::New,
        })
    }

    fn cancel_order(&self, _order_id: &str) -> Result<()> {
        Ok(())
    }

    fn fetch_balances(&self) -> Result<Vec<Balance>> {
        Ok(vec![Balance {
            asset: "USDT".to_string(),
            free: 1000.0,
            locked: 0.0,
        }])
    }

    fn fetch_positions(&self) -> Result<Vec<Position>> {
        Ok(vec![Position {
            symbol: "BTCUSDT".to_string(),
            quantity: 1.0,
            avg_price: 100.0,
        }])
    }

    fn fetch_open_orders(&self) -> Result<Vec<OrderAck>> {
        Ok(vec![OrderAck {
            client_order_id: "c2".to_string(),
            exchange_order_id: Some("e2".to_string()),
            status: OrderStatus::New,
        }])
    }

    fn fetch_candles(&self, _req: &CandleRequest) -> Result<Vec<Candle>> {
        Ok(Vec::new())
    }
}

#[test]
fn sync_account_collects_balances_positions_orders() {
    let exchange = MockExchange;
    let snapshot = sync_account(&exchange).expect("sync");
    assert_eq!(snapshot.balances.len(), 1);
    assert_eq!(snapshot.positions.len(), 1);
    assert_eq!(snapshot.open_orders.len(), 1);
}
