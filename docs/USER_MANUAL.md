# Merrow Quant Trading System User Manual
# Merrow 量化交易系統使用手冊

中文：本手冊面向使用者，說明如何安裝、設定、回測、模擬與實盤操作，以及 UI 儀表板使用方式。  
English: This manual is for users. It covers installation, configuration, backtest, paper, live trading, and the UI dashboard.

> 風險提示 / Risk Notice  
> 中文：量化交易存在虧損風險。請先在回測/模擬環境驗證策略，再評估風險後投入實盤。  
> English: Quant trading involves loss risk. Validate in backtest/paper before going live.

---

## 1) Quick Start / 快速開始

Prerequisites / 先決條件
- Rust (stable) + Cargo
- Node.js + pnpm (for UI)
- PostgreSQL (optional)

Build / 編譯
```bash
cargo build
```

Run backtest / 執行回測
```bash
cargo run --bin merrow -- backtest --config config.toml
```

Run paper / 執行模擬
```bash
cargo run --bin merrow -- paper --config config.toml
```

Run live / 執行實盤（需交易所 API）
```bash
cargo run --bin merrow -- live --config config.toml
```

Run UI API / 啟動 UI API
```bash
cargo run --bin merrow_ui -- --config config.toml --addr 127.0.0.1:8088
```

Run UI frontend / 啟動 UI 前端
```bash
cd ui
pnpm install
pnpm dev
```

中文：前端預設呼叫 `http://127.0.0.1:8088`。  
English: Frontend defaults to `http://127.0.0.1:8088`.

---

## 2) Configuration / 設定

Config file / 設定檔
- `config.toml` (see `config.example.toml`)

Env override / 環境變數覆寫
- 開發環境可使用 `.env.local` 或 `.env`（不提交版本庫）  
- `.env.example` 提供完整範例

Core parameters / 核心參數
- `symbol` (BTC/ETH)
- `time_trigger_minutes`  
  中文：必須為 5 的倍數，且 <= 100  
  English: Must be multiple of 5 and <= 100
- `price_trigger_enabled` / `time_trigger_enabled`
- `buy_cash_ratio` / `sell_pos_ratio` / `rebuy_cash_ratio`（0~1）
- `order_type` (market/limit) + `limit_price` (if limit)

---

## 3) Modes / 模式

### A) Backtest / 回測（MVP 必備）
中文：使用歷史資料回放，產出績效指標。  
English: Replay historical data and output metrics.

Inputs / 輸入來源
- CSV
- 交易所歷史資料（由 adapter 載入）

Output / 輸出
- Summary + metrics（Return, Max Drawdown, Win Rate, Trade Count 等）

### B) Paper / 模擬
中文：不下真單，但會模擬撮合與資金變化，可持久化狀態。  
English: No real orders; simulates fills and account state with optional persistence.

State persistence / 狀態保存
- `MERROW_PAPER_STATE_PATH` (default: `output/paper_state.json`)

### C) Live / 實盤
中文：需交易所 API Key/Secret，會下真單。  
English: Requires exchange API keys; places real orders.

Retry / 重試
- `MERROW_LIVE_RETRY_MAX`
- `MERROW_LIVE_RETRY_BASE_MS`
- `MERROW_LIVE_RETRY_MAX_DELAY_MS`
- `MERROW_LIVE_RETRY_JITTER_PCT`

---

## 4) Exchanges / 交易所

中文：交易所可插拔，V1 支援多交易所 adapter。  
English: Pluggable exchanges; V1 supports multiple adapters.

常見參數 / Common settings
- `exchange = "binance" | "bybit" | "okx"`
- API key / secret / passphrase (if needed)

---

## 5) Database (PostgreSQL) / 資料庫

中文：可選擇啟用 PostgreSQL 以保存回測/交易記錄。  
English: PostgreSQL is optional for storing backtests, orders, trades, balances, positions.

Enable / 啟用
```text
MERROW_PG_ENABLED=true
MERROW_PG_INIT_SCHEMA=true
```

Connection / 連線
- `MERROW_PG_URL=postgres://user:pass@localhost:5432/merrow`

---

## 6) Logging / 日誌

中文：使用 `MERROW_LOG` 設定等級（或 `RUST_LOG`）。  
English: Use `MERROW_LOG` (or `RUST_LOG`) for log level.

Format / 格式
- `MERROW_LOG_FORMAT=plain|json`

---

## 7) Metrics / 指標

中文：可輸出 Prometheus textfile。  
English: Prometheus textfile metrics supported.

```text
MERROW_METRICS_PATH=output/metrics.prom
```

---

## 8) UI Dashboard / UI 儀表板

中文：UI 透過 `merrow_ui` 讀取資料庫與 metrics。  
English: UI uses `merrow_ui` to read from DB and metrics.

常見問題 / FAQ
- Q: UI 無資料？  
  A: 檢查 `MERROW_PG_ENABLED`、資料庫連線、或是否有回測/模擬資料。

---

## 9) Safety Checklist / 安全檢查清單

Before live / 實盤前
- 確認 API 權限最小化（僅交易，不可提現）  
- 先回測，再模擬  
- 先使用小額  
- 確認觸發器與限價設定符合預期  

---

## 10) Troubleshooting / 排錯

Common issues / 常見問題
- UI/CLI 無法連線：確認埠號與 `MERROW_UI_ADDR`  
- 沒有回測資料：確認資料來源與 symbol  
- 交易所錯誤：確認 API key/secret/passphrase

---

## 11) References / 參考文件
- `DEVELOPMENT_MANUAL.md`
- `OPERATIONS.md`
- `ARCHITECTURE.md`
- `UI_SWOT.md`
- `DECISIONS.md`
