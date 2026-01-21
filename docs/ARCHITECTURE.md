# Merrow System Architecture
# Merrow 系統架構

中文：本文件描述系統模組、資料流與執行流程，作為實作依據。  
English: This document describes modules, data flows, and execution paths for implementation.

## 1) High-Level View / 高層架構
```text
            +------------------+
            |  Config Loader   |
            +--------+---------+
                     |
           +---------v----------+
           |   Scheduler/Timer  |
           +---------+----------+
                     |
     +---------------v---------------+
     |        Trigger Engine         |
     +---------------+---------------+
                     |
             +-------v-------+
             |   Strategy    |
             +-------+-------+
                     |
              +------v------+
              | Risk Manager|
              +------+------+
                     |
              +------v------+
              | Order Router|
              +------+------+
                     |
     +---------------v---------------+
     | Execution (Live/Paper/Backtest)|
     +---------------+---------------+
                     |
            +--------v---------+
            | Storage (PGSQL)  |
            +------------------+
```

## 2) Core Components / 核心模組

### Strategy Engine / 策略引擎
- 中文：依據觸發條件與市場資料產生 Signal。  
  English: Produces signals based on triggers and market data.

### Trigger Engine / 觸發引擎
- 中文：支援 Time Trigger 與 Price Trigger，模式為 `any` 或 `all`。  
  English: Supports time and price triggers with `any` or `all` mode.

### Risk Manager / 風控
- 中文：限制單筆最大交易比例、保留最低現金。  
  English: Enforces max trade ratio and minimum cash reserve.

### Order Router / 訂單路由
- 中文：統一送單入口，對實盤/模擬/回測使用同一介面。  
  English: Unified order entry across live/paper/backtest.

### Execution Layer / 執行層
- Backtest Engine：回放歷史資料，模擬成交。  
  Backtest Engine: replays historical data and simulates fills.
- Paper Executor：用即時行情模擬成交。  
  Paper Executor: simulates fills on live market data.
- Live Executor：對交易所送出實盤訂單。  
  Live Executor: sends real orders to exchange.

### Exchange Adapter / 交易所介面
- 中文：實作 REST/WS，提供統一 API。  
  English: Provides unified REST/WS API.

### Storage / 資料儲存
- 中文：所有行情、訂單、成交、持倉、回測結果存 PGSQL。  
  English: Persist prices, orders, trades, positions, and backtests in PostgreSQL.

## 3) Data Flow / 資料流

### Live/Paper Flow / 實盤/模擬流程
1) Scheduler 觸發評估（Time 或 Price）。  
2) Strategy 產生 Signal。  
3) Risk Manager 檢查。  
4) Order Router -> Executor。  
5) Executor 更新狀態並寫入 PGSQL。

### Backtest Flow / 回測流程
1) 讀取 CSV/交易所歷史資料。  
2) 逐筆回放 Candle。  
3) Trigger/Strategy 評估 -> 產生 Signal。  
4) 模擬成交（市價/限價）。  
5) 更新資金、持倉與績效指標。

## 4) Concurrency Model / 併發模型
- 中文：使用 async runtime（Tokio）管理 WS/REST 與排程器。  
  English: Use async runtime (Tokio) for WS/REST and scheduler.
- 中文：單一策略可維持單執行緒決策，交易所 IO 可多執行緒。  
  English: Strategy decisions can be single-threaded; IO is concurrent.

## 5) Error Handling / 錯誤處理
- 中文：交易所錯誤需重試並記錄。  
  English: Exchange errors should be retried and logged.
- 中文：下單需避免重複送單（幂等機制）。  
  English: Orders must be idempotent to avoid duplicates.

## 6) Extensibility / 可擴充性
- 中文：交換 Adapter 實作 Exchange trait 即可接入。  
  English: Implement the Exchange trait to add new exchanges.
- 中文：策略只需實作 Strategy trait，風控與觸發器可重用。  
  English: Implement Strategy trait; reuse triggers and risk manager.

## 7) Key Interfaces / 主要介面
```rust
pub trait Strategy { fn on_tick(&mut self, ctx: &StrategyContext) -> Vec<Signal>; }
pub trait Trigger { fn should_fire(&self, ctx: &TriggerContext) -> bool; }
pub trait Exchange {
    fn place_order(&self, order: &OrderRequest) -> Result<OrderAck>;
    fn fetch_candles(&self, req: &CandleRequest) -> Result<Vec<Candle>>;
    fn stream_ticker(&self) -> Result<TickerStream>;
}
```

## 8) DDD Alignment / DDD 對齊
- 中文：各模組對應有界上下文（Market Data、Strategy、Execution、Risk、Backtest）。  
  English: Modules align with bounded contexts (Market Data, Strategy, Execution, Risk, Backtest).
- 中文：領域模型與聚合規則請參考 `DOMAIN_MODEL.md`。  
  English: See `DOMAIN_MODEL.md` for domain model and aggregates.
