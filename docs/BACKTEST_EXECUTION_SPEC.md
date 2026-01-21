# Backtest Execution Specification
# 回測撮合規格

中文：本文件定義回測的時間序、撮合規則、成交價與成本模型，確保結果可重現。  
English: This document defines backtest time flow, matching rules, fill prices, and cost models to ensure reproducibility.

## 1) Time Model / 時間模型
- 中文：Candle `time` 為該根 K 線的收盤時間（UTC）。  
  English: Candle `time` is the close time (UTC).
- 中文：策略只使用已收盤的 K 線資料。  
  English: Strategy uses only closed candles.

## 2) Evaluation Order / 執行順序
對每根 Candle `t`：
1) 更新市場狀態（close、MA）。  
2) 觸發器評估（time + price）。  
3) 策略輸出 Signal。  
4) 產生 OrderRequest（進入待撮合佇列）。  
5) 在下一根 Candle `t+1` 進行成交判定。

English: For each candle `t`:
1) Update market state (close, MA).  
2) Evaluate triggers (time + price).  
3) Strategy emits signals.  
4) Build OrderRequest (enqueue).  
5) Execute fills on next candle `t+1`.

## 3) Market Orders / 市價單
- 中文：市價單在下一根 Candle 的開盤價成交。  
  English: Market orders fill at the next candle open.
- 中文：若為最後一根 Candle，未成交則取消並記錄。  
  English: On the final candle, unfilled market orders are canceled and logged.

## 4) Limit Orders / 限價單
- 中文：限價單為 GTC（直到成交或手動取消）。  
  English: Limit orders are GTC until filled or canceled.
- 中文：買單：若 `low <= limit_price` 則成交。  
  English: Buy limit fills if `low <= limit_price`.
- 中文：賣單：若 `high >= limit_price` 則成交。  
  English: Sell limit fills if `high >= limit_price`.
- 中文：成交價為 `limit_price`。  
  English: Fill price is `limit_price`.

## 5) Order Priority / 訂單優先序
- 中文：同一時間戳的訂單，依產生順序先後撮合。  
  English: Orders with the same timestamp fill in creation order.
- 中文：資金不足時，該訂單被拒絕並記錄。  
  English: Reject and log if insufficient funds.

## 6) Fees and Slippage / 手續費與滑點
- 中文：市價單成交價套用滑點。  
  English: Market orders apply slippage.
- 中文：限價單預設不套用滑點（可配置）。  
  English: Limit orders do not apply slippage by default (configurable).

計算方式 / Calculation
- Buy: `fill_price * (1 + slippage_bps/10000)`  
- Sell: `fill_price * (1 - slippage_bps/10000)`  
- Fee: `fill_value * fee_rate`

## 7) Data Gaps / 缺資料處理
- 中文：若 Candle 缺失，該時間不評估策略。  
  English: Missing candles are skipped.
- 中文：可選擇填補（前值複製，volume=0），但需明確標記。  
  English: Optional gap-fill (forward close, zero volume) must be flagged.

## 8) Determinism / 可重現性
- 中文：相同資料與參數必須產生一致結果。  
  English: Same data and params yield identical results.
- 中文：任何隨機元素需固定 seed。  
  English: Any randomness must be seeded.
