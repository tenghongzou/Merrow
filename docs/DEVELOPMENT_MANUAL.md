# Merrow Quant Trading System (Rust) Development Manual
# Merrow 量化交易系統（Rust）開發手冊

中文：本文件提供 Rust 量化交易系統的完整開發規格，包含模組介面、資料結構、回測流程、專案骨架、限制與未來方向。  
English: This document defines the Rust quant trading system specification, including module interfaces, data structures, backtest flow, project skeleton, limits, and roadmap.

中文：重要選型與決策索引見 `DECISIONS.md`。  
English: Key decisions are indexed in `DECISIONS.md`.

## 1) Scope and MVP / 範圍與 MVP
中文：目標是建立支援回測（MVP）、模擬與實盤的單策略、單資產（BTC/ETH）系統，交易所可擴充。  
English: Build a single-strategy, single-asset (BTC/ETH) system with backtest (MVP), paper, and live modes, with exchange-pluggable adapters.

中文：策略規則（MVP）  
English: Strategy rules (MVP)
- 中文：買入使用可用現金 50%。  
  English: Buy using 50% of available cash.
- 中文：賣出使用持倉 20%。  
  English: Sell 20% of position.
- 中文：賣出後，使用賣出所得現金 50% 再買入。  
  English: After sell, re-buy with 50% of sell proceeds.

中文：觸發條件（兩者皆需實作，可同時啟用）  
English: Triggers (both required, can be enabled together)
- 中文：定時觸發（分鐘）：必須為 5 的倍數，最大 100 分鐘，可自定義。  
  English: Time trigger (minutes): must be a multiple of 5, maximum 100, user-configurable.
- 中文：價格門檻觸發（例如 MA 偏離門檻）。  
  English: Price-threshold trigger (e.g., MA deviation).

## 2) Non-Goals / 非目標
中文：MVP 不做多策略組合、多交易所套利、槓桿或衍生品。  
English: MVP excludes multi-strategy portfolios, cross-exchange arbitrage, leverage, or derivatives.

## 3) Architecture Overview / 架構總覽
中文：系統採分層設計，核心與交易所解耦。  
English: The system uses a layered design with core logic decoupled from exchanges.

核心層 Core
- Strategy Engine / 策略引擎
- Trigger Engine / 觸發引擎
- Risk Manager / 風控
- Order Manager / 訂單管理

外部層 External
- Exchange Adapter (REST/WS)
- Market Data / 行情
- Storage (PostgreSQL)

## 4) Module Interfaces (Rust) / 模組介面（Rust）
中文：下列為核心 trait 與模組的最小設計，方便擴充不同交易所與策略。  
English: Minimal core traits to enable exchange and strategy extensibility.

```rust
// src/core/strategy.rs
pub trait Strategy {
    fn on_tick(&mut self, ctx: &StrategyContext) -> Vec<Signal>;
}

// src/core/trigger.rs
pub trait Trigger {
    fn should_fire(&self, ctx: &TriggerContext) -> bool;
}

// src/exchange/mod.rs
pub trait Exchange {
    fn place_order(&self, order: &OrderRequest) -> Result<OrderAck>;
    fn cancel_order(&self, order_id: &str) -> Result<()>;
    fn fetch_balances(&self) -> Result<Vec<Balance>>;
    fn fetch_positions(&self) -> Result<Vec<Position>>;
    fn fetch_open_orders(&self) -> Result<Vec<Order>>;
    fn fetch_candles(&self, req: &CandleRequest) -> Result<Vec<Candle>>;
    fn stream_ticker(&self) -> Result<TickerStream>;
}

// src/core/order_router.rs
pub trait OrderRouter {
    fn route(&self, req: OrderRequest) -> Result<OrderAck>;
}
```

## 5) Data Structures / 資料結構
中文：核心資料模型需簡潔、可序列化（存 DB 或 JSON）。  
English: Core models should be compact and serializable.

```rust
#[derive(Clone)]
pub struct Candle {
    pub time: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

pub enum Side { Buy, Sell }

pub enum OrderType {
    Market,
    Limit { price: f64 },
}

pub struct OrderRequest {
    pub symbol: String,
    pub side: Side,
    pub order_type: OrderType,
    pub quantity: f64,
}

pub enum OrderStatus { New, PartiallyFilled, Filled, Canceled, Rejected }

pub struct Position {
    pub symbol: String,
    pub quantity: f64,
    pub avg_price: f64,
}

pub struct Account {
    pub cash: f64,
    pub positions: Vec<Position>,
}
```

## 6) Strategy Logic / 策略邏輯
中文：策略在每次觸發後評估，若滿足條件則產生 Signal。  
English: Strategy evaluates on each trigger; generates signals when conditions hold.

中文：觸發整合策略  
English: Trigger combination strategy
- 中文：`trigger_mode = any` 表示任一觸發滿足即計算訊號（預設）。  
  English: `trigger_mode = any` evaluates when any trigger fires (default).
- 中文：`trigger_mode = all` 表示兩者皆觸發才計算訊號。  
  English: `trigger_mode = all` evaluates only when both triggers fire.

```text
On each evaluation:
  if entry_condition:
    buy_amount = cash * 0.5
  if exit_condition:
    sell_qty = position_qty * 0.2
    rebuy_amount = sell_proceeds * 0.5
```

中文：定時與價格觸發必須皆可獨立啟用。  
English: Time and price triggers must be independently enableable.

## 7) Backtest Engine / 回測引擎
中文：回測為 MVP，需支援 CSV 或交易所歷史資料。  
English: Backtest is MVP; support CSV or exchange historical data.

流程 Flow
1) 讀取歷史資料（CSV / Exchange），轉換為 Candle/Trade 序列。  
2) 時序回放，將每根 Candle 餵給策略與觸發器。  
3) 產生 Signal -> 轉為 OrderRequest -> 模擬撮合。  
4) 更新資金、持倉與績效指標。

Limit Order 模擬（建議）
- 中文：買單：若 `low <= limit_price` 則成交。  
  English: Buy limit: fill if `low <= limit_price`.
- 中文：賣單：若 `high >= limit_price` 則成交。  
  English: Sell limit: fill if `high >= limit_price`.
- 中文：成交價格採 limit_price，並計入手續費與滑點。  
  English: Fill at limit_price and apply fee/slippage.

輸出指標 Metrics
- Return, Max Drawdown, Win Rate, Trade Count, Sharpe (optional)

## 8) Order Management / 訂單管理
中文：需支援市價與限價單（MVP 必做）。  
English: Support market and limit orders (required for MVP).

中文：實盤與模擬都使用同一 OrderRequest 結構，統一介面。  
English: Use the same OrderRequest for live and paper to keep a unified interface.

## 9) PostgreSQL Schema / 資料庫 Schema
中文：以下為必要資料表（可擴充）。  
English: Required tables (extend as needed).

```sql
-- prices
time BIGINT, symbol TEXT, open DOUBLE, high DOUBLE, low DOUBLE, close DOUBLE, volume DOUBLE

-- orders
id TEXT, time BIGINT, symbol TEXT, side TEXT, order_type TEXT, price DOUBLE, qty DOUBLE, status TEXT

-- trades
id TEXT, order_id TEXT, time BIGINT, price DOUBLE, qty DOUBLE, fee DOUBLE

-- positions
time BIGINT, symbol TEXT, qty DOUBLE, avg_price DOUBLE

-- backtest_runs
id TEXT, start_time BIGINT, end_time BIGINT, params JSONB

-- backtest_metrics
run_id TEXT, return DOUBLE, max_drawdown DOUBLE, win_rate DOUBLE, trade_count INT
```

## 10) Config Spec / 配置規格
中文：所有參數使用 `config.toml` 或環境變數覆寫。  
English: Use `config.toml` with optional env overrides.

中文：開發環境可使用 `.env` 或 `.env.local` 載入環境變數（請勿提交到版本庫）。  
English: In dev, use `.env` or `.env.local` to load env vars (do not commit).

中文：日誌使用 `MERROW_LOG`（或 `RUST_LOG`）設定層級，例如 `info`、`debug`。  
English: Logging uses `MERROW_LOG` (or `RUST_LOG`) to set levels, e.g., `info`, `debug`.

中文：可用 `MERROW_LOG_FORMAT=json` 輸出 JSON 格式日誌（預設 `plain`）。  
English: Use `MERROW_LOG_FORMAT=json` for JSON logs (default `plain`).

中文：設定 `MERROW_METRICS_PATH` 可輸出 Prometheus textfile 指標。  
English: Set `MERROW_METRICS_PATH` to emit Prometheus textfile metrics.

## 16) UI Dashboard (Svelte) / 前端儀表板
中文：本機可使用 Svelte UI 搭配 `merrow_ui` 服務查看資料。  
English: Run the local Svelte UI with the `merrow_ui` API service.

Backend / 後端
```text
cargo run --bin merrow_ui -- --config config.toml --addr 127.0.0.1:8088
```

Frontend / 前端
```text
cd ui
npm install
npm run dev
```

中文：前端會呼叫 `VITE_API_BASE`（預設 `http://127.0.0.1:8088`）。  
English: Frontend uses `VITE_API_BASE` (default `http://127.0.0.1:8088`).

Decision / 決策
- 中文：UI 框架 SWOT 與選型請見 `UI_SWOT.md`（V1 採 Svelte）。  
  English: See `UI_SWOT.md` for SWOT and framework decision (V1 uses Svelte).

Constraints / 參數限制
- 中文：`time_trigger_minutes % 5 == 0` 且 `<= 100`。  
  English: `time_trigger_minutes % 5 == 0` and `<= 100`.
- 中文：`buy_cash_ratio`, `sell_pos_ratio`, `rebuy_cash_ratio` 範圍為 0~1。  
  English: Ratios must be between 0 and 1.
- 中文：至少啟用一種觸發器。  
  English: At least one trigger must be enabled.

## 11) Project Skeleton / 專案骨架
中文：以下為建議專案結構（MVP）。  
English: Suggested project structure (MVP).

```text
merrow/
  src/
    main.rs
    app/
      cli.rs
    core/
      mod.rs
      strategy.rs
      trigger.rs
      triggers.rs
      strategies.rs
      risk.rs
      order_router.rs
      order_builder.rs
      order_flow.rs
    backtest/
      mod.rs
      engine.rs
      fill.rs
    paper/
      mod.rs
    exchange/
      mod.rs
      adapter.rs
      binance.rs
      rest.rs
      sync.rs
    data/
      mod.rs
      market_data.rs
      csv_loader.rs
      exchange_loader.rs
    storage/
      mod.rs
      postgres.rs
    models/
      mod.rs
      types.rs
    config.rs
  tests/
    backtest_costs.rs
    backtest_fill.rs
    backtest_integration.rs
    backtest_metrics.rs
    binance_signing.rs
    csv_loader.rs
    exchange_loader.rs
    exchange_sync.rs
    report_output.rs
    config_validation.rs
    domain_invariants.rs
  config.example.toml
  DEVELOPMENT_MANUAL.md
```

## 12) Testing / 測試
中文：策略與回測需單元測試；交易所介面做整合測試。  
English: Unit-test strategy and backtest; integration-test exchange adapters.

建議測試案例 / Suggested Tests
- 觸發器邏輯（time/price, any/all）  
- 限價單撮合（跨越 candle high/low）  
- 手續費與滑點是否正確影響盈虧  

## 12.1) CLI Usage / CLI 使用方式
中文：CLI 參數可覆寫設定檔內容。  
English: CLI flags can override config values.

```text
merrow --config <path> --symbol <SYMBOL> --output-format <fmt> --output-path <path> --initial-cash <amount> --live-execute
```

Flags / 參數
- `-c, --config`：設定檔路徑（預設 `config.toml`）  
  Config path (default: `config.toml`)
- `-s, --symbol`：覆寫交易對  
  Override symbol
- `-f, --output-format`：覆寫輸出格式（`none|json|csv`）  
  Override output format (`none|json|csv`)
- `-o, --output-path`：覆寫輸出路徑  
  Override output path
- `-i, --initial-cash`：覆寫回測初始資金  
  Override backtest initial cash
- `--live-execute`：實盤送單（預設為 dry-run）  
  Execute live orders (default: dry-run)

## 12.2) Live Mode / 實盤模式
中文：實盤模式需啟用 `config.mode = "live"`，並使用 `data.source = "exchange"`。  
English: Live mode requires `config.mode = "live"` and `data.source = "exchange"`.

中文：目前支援 `exchange = "binance" | "bybit" | "okx"`。  
English: Supported: `exchange = "binance" | "bybit" | "okx"`.

中文：支援的 `data.candle_interval`：`1m|3m|5m|15m|30m|1h|2h|4h|6h|12h|1d|1w`。  
English: Supported `data.candle_interval`: `1m|3m|5m|15m|30m|1h|2h|4h|6h|12h|1d|1w`.

Environment / 環境變數
- `MERROW_BINANCE_API_KEY`：必填  
  Required
- `MERROW_BINANCE_API_SECRET`：必填  
  Required
- `MERROW_BINANCE_BASE_URL`：可選（預設 `https://api.binance.com`）  
  Optional (default: `https://api.binance.com`)
- `MERROW_BINANCE_RECV_WINDOW`：可選（毫秒，預設 `5000`）  
  Optional (milliseconds, default: `5000`)

- `MERROW_BYBIT_API_KEY`：必填  
  Required
- `MERROW_BYBIT_API_SECRET`：必填  
  Required
- `MERROW_BYBIT_BASE_URL`：可選（預設 `https://api.bybit.com`）  
  Optional (default: `https://api.bybit.com`)
- `MERROW_BYBIT_RECV_WINDOW`：可選（毫秒，預設 `5000`）  
  Optional (milliseconds, default: `5000`)
- `MERROW_BYBIT_ACCOUNT_TYPE`：可選（預設 `UNIFIED`）  
  Optional (default: `UNIFIED`)

- `MERROW_OKX_API_KEY`：必填  
  Required
- `MERROW_OKX_API_SECRET`：必填  
  Required
- `MERROW_OKX_PASSPHRASE`：必填  
  Required
- `MERROW_OKX_BASE_URL`：可選（預設 `https://www.okx.com`）  
  Optional (default: `https://www.okx.com`)

- `MERROW_CASH_ASSET`：可選（若交易對無法推斷報價幣時必填）  
  Optional; required when quote asset cannot be inferred

Example (PowerShell) / 範例（PowerShell）
```text
$env:MERROW_BINANCE_API_KEY="your_key"
$env:MERROW_BINANCE_API_SECRET="your_secret"
$env:MERROW_CASH_ASSET="USDT"
merrow --config config.toml --symbol BTCUSDT --live-execute
```

Dry-run example (PowerShell) / 乾跑範例（PowerShell）
```text
$env:MERROW_BINANCE_API_KEY="your_key"
$env:MERROW_BINANCE_API_SECRET="your_secret"
$env:MERROW_CASH_ASSET="USDT"
merrow --config config.toml --symbol BTCUSDT
```

Sample live config / 實盤設定範例
```toml
mode = "live"
exchange = "binance"
symbol = "BTCUSDT"

[data]
source = "exchange"
candle_interval = "5m"
exchange_limit = 200
```

Notes / 備註
- 中文：OKX 建議使用 `BTC-USDT` 形式（instId）。  
  English: OKX uses `BTC-USDT` (instId) format.

Live-mode checklist / 實盤前檢查清單
- 確認 `config.mode = "live"` 且 `data.source = "exchange"`  
  Confirm `config.mode = "live"` and `data.source = "exchange"`.
- 設定必要環境變數（API Key/Secret）。  
  Set required env vars (API Key/Secret).
- 先在回測與 dry-run 驗證策略。  
  Validate strategy in backtest and dry-run first.
- 確認 `symbol` 與 `candle_interval` 正確。  
  Verify `symbol` and `candle_interval`.
- 檢查資金與風控限制是否符合預期。  
  Check balances and risk limits.

Common live errors / 常見實盤錯誤
- 中文：`MERROW_BINANCE_API_KEY must be set` -> 未設定 API Key。  
  English: API key env var not set.
- 中文：`MERROW_BINANCE_API_SECRET must be set` -> 未設定 API Secret。  
  English: API secret env var not set.
- 中文：`cash asset not found; set MERROW_CASH_ASSET` -> 無法推斷報價幣。  
  English: Quote asset not inferred; set `MERROW_CASH_ASSET`.
- 中文：`unsupported interval for live mode` -> 不支援的 `data.candle_interval`。  
  English: Unsupported `data.candle_interval`.
- 中文：`live mode only supports binance currently` -> 交易所尚未支援。  
  English: Only Binance is supported for live mode now.

Troubleshooting Binance HTTP errors / Binance HTTP 錯誤排查
- 中文：`429` (rate limit) -> 降低請求頻率或加大 `data.candle_interval`。  
  English: `429` (rate limit) -> reduce request rate or increase `data.candle_interval`.
- 中文：`5xx` -> 交易所暫時異常，稍後重試。  
  English: `5xx` -> exchange outage; retry later.
- 中文：若持續發生，確認 `exchange_limit` 是否過大。  
  English: If persistent, verify `exchange_limit` is not too large.

Live retry / 實盤重試設定
- `MERROW_LIVE_RETRY_MAX`：可選（預設 `3`）  
  Optional (default: `3`)
- `MERROW_LIVE_RETRY_BASE_MS`：可選（預設 `500` 毫秒）  
  Optional (default: `500` ms)
- `MERROW_LIVE_RETRY_MAX_DELAY_MS`：可選（預設 `8000` 毫秒）  
  Optional (default: `8000` ms)
- `MERROW_LIVE_RETRY_JITTER_PCT`：可選（預設 `20`）  
  Optional (default: `20`)

## 12.3) Paper Mode / 模擬模式
中文：模擬模式需啟用 `config.mode = "paper"`，不會送出實盤單。  
English: Paper mode uses `config.mode = "paper"` and does not place real orders.

中文：資料來源可用 CSV 或交易所，資金使用 `backtest.initial_cash`。  
English: Data can come from CSV or exchange; starting cash uses `backtest.initial_cash`.

中文：模擬帳戶會寫入狀態檔，預設 `output/paper_state.json`。  
English: Paper account state is persisted to `output/paper_state.json` by default.

- `MERROW_PAPER_STATE_PATH`：可選（指定狀態檔路徑）  
  Optional (override paper state path)

中文：模擬模式輸出可使用 `output.format` 與 `output.path`（與回測相同）。  
English: Paper output uses `output.format` and `output.path` (same as backtest).

## 12.4) PostgreSQL Persistence / PGSQL 寫入
中文：設定 `MERROW_PG_ENABLED=1` 可將回測與模擬結果寫入 PGSQL。  
English: Set `MERROW_PG_ENABLED=1` to persist backtest/paper results into PGSQL.

中文：預設會建立/更新資料表結構（可設 `MERROW_PG_INIT_SCHEMA=0` 關閉）。  
English: Schema is created by default (set `MERROW_PG_INIT_SCHEMA=0` to disable).

中文：CLI 亦可使用 `--pg-enabled true|false` 覆寫（優先於環境變數）。  
English: CLI flag `--pg-enabled true|false` overrides env (takes precedence).

## 13) Limits & Risks / 限制與風險
中文：API 延遲、行情落後、限價單長時間不成交。  
English: API latency, stale market data, unfilled limit orders.

## 14) Roadmap / 未來發展方向
中文：多資產、多策略組合、動態風控、Dashboard。  
English: Multi-asset portfolios, adaptive risk, and dashboards.

## 15) TDD + DDD Approach / 開發模式
中文：專案採 TDD 與 DDD。先寫測試（Red），再實作（Green），最後重構。  
English: The project follows TDD and DDD: write tests first (Red), implement (Green), then refactor.

中文：領域模型與通用語彙請參考 `DOMAIN_MODEL.md`；測試流程請參考 `TEST_PLAN.md`。  
English: See `DOMAIN_MODEL.md` for the domain model and `TEST_PLAN.md` for the TDD test workflow.
