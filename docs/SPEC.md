# Merrow Quant Trading System Specification
# Merrow 量化交易系統規格書

中文：本文件定義 Merrow 量化交易系統的功能與非功能需求，作為設計、實作與驗收依據。  
English: This document defines functional and non-functional requirements for the Merrow quant trading system as the basis for design, implementation, and acceptance.

## 1) Objectives / 目標
- 中文：提供單策略、單資產（BTC/ETH）的回測、模擬與實盤交易能力。  
  English: Provide backtest, paper, and live trading for a single strategy and single asset (BTC/ETH).
- 中文：支援「定時觸發」與「價格門檻」觸發，皆可啟用。  
  English: Support both time-based and price-threshold triggers; both can be enabled.
- 中文：交易所介面可插拔，資料庫使用 PostgreSQL。  
  English: Exchange interface is pluggable; PostgreSQL is the storage backend.

## 2) Scope / 範圍
包含 / In scope
- 回測引擎（MVP 必須）。  
  Backtest engine (required for MVP).
- 模擬撮合（Paper Trading）。  
  Paper trading execution simulation.
- 實盤下單（Live Trading）。  
  Live trading execution.
- 市價與限價單。  
  Market and limit orders.
- CSV 匯入與交易所歷史資料下載（至少支援其一，MVP 支援兩者）。  
  CSV import and exchange historical fetch (MVP supports both).

不包含 / Out of scope (MVP)
- 多策略組合、跨交易所套利、槓桿/衍生品。  
  Multi-strategy portfolios, cross-exchange arbitrage, leverage/derivatives.

## 3) Assumptions / 假設
- 中文：先支援單一交易所實作，後續再擴充。  
  English: Implement one exchange first, extend later.
- 中文：行情以 K 線為主，必要時可改用成交資料。  
  English: Candles are the primary data source; trades optional later.

## 4) Functional Requirements / 功能需求

### 4.1 Modes / 模式
- Backtest：讀取歷史資料，輸出績效指標。  
  Replay historical data and output metrics.
- Paper：模擬成交與資金變化，不送實盤。  
  Simulate fills and balances without real orders.
- Live：實盤下單，含限價與市價。  
  Live order placement for market and limit orders.

### 4.2 Strategy Rules / 策略規則
- 買入：可用現金 * 0.5。  
  Buy: available cash * 0.5.
- 賣出：持倉數量 * 0.2。  
  Sell: position quantity * 0.2.
- 再買入：賣出所得現金 * 0.5。  
  Re-buy: sell proceeds * 0.5.

### 4.3 Triggers / 觸發條件
- 定時觸發：`time_trigger_minutes` 必須為 5 的倍數且 <= 100，可自定義。  
  Time trigger: must be a multiple of 5 and <= 100 minutes; user-configurable.
- 價格門檻：基於移動平均 MA，支援買入/賣出門檻。  
  Price threshold: MA-based buy/sell thresholds.
- 觸發模式：`any` 或 `all`。  
  Trigger mode: `any` or `all`.

### 4.4 Orders / 訂單
- 支援市價單與限價單。  
  Support market and limit orders.
- 交易成本：手續費與滑點可配置。  
  Configurable fees and slippage.

### 4.5 Data Sources / 資料來源
- CSV 匯入：需定義欄位（time, open, high, low, close, volume）。  
  CSV import with defined schema (time, open, high, low, close, volume).
- 交易所歷史資料：使用 REST 下載。  
  Exchange historical data via REST.

### 4.6 Storage / 儲存
- PostgreSQL 儲存行情、訂單、成交、持倉、回測結果。  
  PostgreSQL stores prices, orders, trades, positions, and backtest results.

## 5) Non-Functional Requirements / 非功能需求
- 稳定性：回測可重現、結果一致。  
  Determinism: backtest results should be reproducible.
- 可靠性：實盤需重試、限速控制、斷線恢復。  
  Reliability: retries, rate-limit handling, and reconnect for live trading.
- 安全性：API 金鑰不落地明文；支援環境變數。  
  Security: avoid plain-text keys on disk; allow env vars.
- 可觀測性：日誌分級，保留關鍵事件。  
  Observability: structured logs with critical events.

## 6) Constraints / 約束
- `time_trigger_minutes % 5 == 0` 且 `<= 100`。  
  `time_trigger_minutes % 5 == 0` and `<= 100`.
- `buy_cash_ratio`, `sell_pos_ratio`, `rebuy_cash_ratio` 在 0~1。  
  Ratios must be in [0, 1].
- 至少啟用一種觸發器。  
  At least one trigger must be enabled.

## 7) CSV Schema / CSV 規格
必填欄位 / Required fields
- `time` (RFC3339 或 epoch seconds)  
- `open`, `high`, `low`, `close`, `volume`

## 8) Acceptance Criteria / 驗收標準
- 回測結果可重現（相同資料與參數）。  
  Backtest is reproducible with same inputs.
- 觸發規則正確執行（time + price）。  
  Triggers fire correctly for time and price.
- 支援限價與市價（回測、模擬與實盤一致介面）。  
  Market and limit orders supported across all modes.
- PGSQL 正確寫入與查詢。  
  PostgreSQL writes/reads successfully.

## 9) Development Approach / 開發模式
- 中文：採 TDD（先測試再實作）與 DDD（領域模型驅動）。  
  English: Use TDD (tests first) and DDD (domain-driven design).
- 中文：需求、設計與測試需可追蹤，參考 `TRACEABILITY.md`。  
  English: Requirements, design, and tests must be traceable; see `TRACEABILITY.md`.
