# Exchange Contract Specification
# 交易所介面契約規格

中文：本文件定義交易所 Adapter 的介面、狀態機、錯誤處理與一致性要求，確保不同交易所可插拔且行為一致。  
English: This document defines the exchange adapter interface, state machine, error handling, and consistency rules for pluggable exchanges.

## 1) Scope / 範圍
- 中文：適用於 Live 與 Paper 模式；回測使用內建撮合器。  
  English: Applies to Live and Paper modes; backtest uses internal matcher.
- 中文：交易所介面必須提供 REST 下單與查詢，並可選擇性支援 WS 行情。  
  English: Adapter must provide REST for orders/queries and optional WS market data.

## 2) Core Interface / 核心介面
```rust
pub trait Exchange {
    fn place_order(&self, order: &OrderRequest) -> Result<OrderAck>;
    fn cancel_order(&self, order_id: &str) -> Result<()>;
    fn fetch_balances(&self) -> Result<Vec<Balance>>;
    fn fetch_positions(&self) -> Result<Vec<Position>>;
    fn fetch_open_orders(&self) -> Result<Vec<Order>>;
    fn fetch_candles(&self, req: &CandleRequest) -> Result<Vec<Candle>>;
    fn stream_ticker(&self) -> Result<TickerStream>;
}
```

## 3) Order Model / 訂單模型
- 中文：OrderRequest 必須包含 `client_order_id`（幂等）。  
  English: OrderRequest must include `client_order_id` for idempotency.
- 中文：OrderAck 需回傳 `exchange_order_id` 與初始狀態。  
  English: OrderAck must return `exchange_order_id` and initial status.
- 中文：狀態機：`new -> partially_filled -> filled` 或 `new -> canceled` 或 `new -> rejected`。  
  English: State machine: `new -> partially_filled -> filled` or `new -> canceled` or `new -> rejected`.

## 4) Idempotency / 幂等規範
- 中文：`client_order_id` 全局唯一（每次策略下單生成）。  
  English: `client_order_id` must be globally unique per order.
- 中文：當網路錯誤或超時發生，重試需使用相同 `client_order_id`。  
  English: Retries after timeout must reuse the same `client_order_id`.
- 中文：若交易所不支援幂等，必須在本地儲存並去重。  
  English: If exchange lacks idempotency, local de-duplication is required.

## 5) Error Handling / 錯誤處理

### Error Classes / 錯誤分類
- Network: timeout, DNS, connection reset  
- Auth: invalid key, permission denied  
- Rate Limit: too many requests  
- Exchange: order rejected, invalid params, system busy

### Retry Policy / 重試策略
- 中文：Network 或 Rate Limit 錯誤可重試，使用指數退避。  
  English: Retry on network or rate-limit errors with exponential backoff.
- 中文：Auth 或參數錯誤不可重試，直接失敗。  
  English: Do not retry auth/parameter errors.
- 中文：超時下單需查詢訂單狀態後再決定是否重試。  
  English: On order timeout, query order status before retrying.

## 6) Rate Limits / 限速
- 中文：Adapter 必須在內部實作節流，避免達到交易所限制。  
  English: Adapter must throttle internally to avoid exchange limits.
- 中文：若收到 429 或類似錯誤，需降低請求速率。  
  English: On 429-like responses, reduce request rate.

## 7) Time Sync / 時間同步
- 中文：需支援與交易所時間同步（偏差超過 1s 需校正）。  
  English: Sync with exchange time; correct drift above 1s.
- 中文：所有訂單與成交時間統一以 UTC 存儲。  
  English: Store order/trade timestamps in UTC.

## 8) Partial Fills / 部分成交
- 中文：Live 模式必須能處理部分成交並更新持倉。  
  English: Live mode must handle partial fills and update positions.
- 中文：Paper 模式可配置是否允許部分成交，預設允許。  
  English: Paper mode may allow partial fills, default enabled.

## 9) Order Size Constraints / 交易大小限制
- 中文：Adapter 必須檢查交易所的最小數量與最小名目價值。  
  English: Adapter must enforce min qty and min notional rules.
- 中文：不符合限制時，訂單需被拒絕並記錄原因。  
  English: Reject and log if constraints are violated.

## 10) WebSocket / WS 行情
- 中文：支援心跳與自動重連。  
  English: Implement heartbeat and auto-reconnect.
- 中文：若有序號，必須處理缺失序號並重建快照。  
  English: Handle sequence gaps and resubscribe or rebuild snapshots.

## 11) Consistency / 一致性
- 中文：下單後必須能查詢訂單狀態並比對本地紀錄。  
  English: After placing orders, reconcile with local records.
- 中文：若交易所回傳狀態與本地不一致，需觸發警報。  
  English: Alert on mismatches between exchange and local states.
