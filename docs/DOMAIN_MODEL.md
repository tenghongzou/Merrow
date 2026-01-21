# Domain Model (DDD)
# 領域模型（DDD）

中文：本文件定義 Merrow 的領域模型、通用語彙、有界上下文與聚合，作為 DDD 開發依據。  
English: This document defines Merrow's domain model, ubiquitous language, bounded contexts, and aggregates for DDD development.

## 1) Ubiquitous Language / 通用語彙
- Candle：K 線資料（OHLCV）。  
  Candle: OHLCV market data.
- Ticker：即時價格摘要。  
  Ticker: real-time price snapshot.
- Trigger：觸發條件（時間或價格）。  
  Trigger: time or price condition.
- Signal：策略輸出（buy/sell/hold）。  
  Signal: strategy output (buy/sell/hold).
- Order：下單請求或交易所訂單。  
  Order: order request or exchange order.
- Trade：成交紀錄。  
  Trade: execution record.
- Position：持倉狀態（數量、均價）。  
  Position: holdings (qty, avg price).
- Balance：資金餘額（free/locked）。  
  Balance: cash balances (free/locked).
- BacktestRun：回測執行與結果集合。  
  BacktestRun: backtest run and metrics.

## 2) Bounded Contexts / 有界上下文
- Market Data Context：Candle/Ticker、資料匯入。  
  Market Data Context: Candle/Ticker and ingestion.
- Strategy Context：Trigger/Signal/Strategy 規則。  
  Strategy Context: trigger/signal/strategy rules.
- Execution Context：Order/Trade/Fill、撮合邏輯。  
  Execution Context: orders, trades, fills, matching logic.
- Risk Context：風控規則與限制。  
  Risk Context: risk rules and limits.
- Portfolio Context：Account/Position/Balance。  
  Portfolio Context: account, positions, balances.
- Backtest Context：回測流程與績效指標。  
  Backtest Context: backtest flow and metrics.
- Exchange Integration Context：交易所介面與一致性。  
  Exchange Integration Context: adapter and exchange consistency.
- Storage Context：Repository 介面與資料存取。  
  Storage Context: repositories and persistence.

## 3) Aggregates / 聚合
- Order Aggregate：Order + Trades，管理狀態轉移與成交一致性。  
  Order aggregate: order + trades, manage state transitions.
- Position Aggregate：Position + PnL，維持數量與均價不變式。  
  Position aggregate: position + PnL, maintain qty/avg invariants.
- BacktestRun Aggregate：回測參數 + 指標。  
  BacktestRun aggregate: params and metrics.

## 4) Domain Services / 領域服務
- TriggerEvaluator：整合 time/price 觸發。  
  TriggerEvaluator: evaluates time/price triggers.
- OrderSizer：依策略比例計算下單大小。  
  OrderSizer: calculates order size per ratios.
- RiskEvaluator：檢查風控限制。  
  RiskEvaluator: enforces risk limits.
- PriceSanityChecker：價格合理性檢查。  
  PriceSanityChecker: validates price sanity.

## 5) Repository Interfaces / Repository 介面
- PricesRepo, OrdersRepo, TradesRepo, PositionsRepo, BacktestRepo  
  Each repository abstracts PGSQL storage.

## 6) Context Mapping / 上下文關係
- Strategy -> Execution：輸出 Signal 並轉成 OrderRequest。  
  Strategy -> Execution: signals become order requests.
- Execution -> Exchange：Live/Paper 經由 Adapter 送單或撮合。  
  Execution -> Exchange: adapter for live/paper execution.
- Backtest -> Execution：重用撮合規則但資料來源不同。  
  Backtest -> Execution: reuse matching logic with historical data.
- Risk -> Execution：所有下單前必須通過風控。  
  Risk -> Execution: risk gates before order placement.

## 7) Invariants / 不變式
- cash >= 0  
- position qty >= 0  
- limit price > 0  
- time_trigger_minutes 為 5 的倍數且 <= 100  
- ratios in [0, 1]
