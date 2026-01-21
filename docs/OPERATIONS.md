# Merrow Operations Guide
# Merrow 運維指南

中文：本文件描述部署、設定、監控、安全與應變流程。  
English: This document describes deployment, configuration, monitoring, security, and incident response.

## 1) Deployment / 部署
- 中文：MVP 以 CLI 形式執行，建議由 systemd 或排程工具管理。  
  English: MVP runs as a CLI service managed by systemd or a scheduler.
- 中文：可選擇 Docker 化，避免環境差異。  
  English: Dockerize optionally to avoid environment drift.

## 2) Configuration / 設定
- 中文：使用 `config.toml` + 環境變數覆寫。  
  English: Use `config.toml` with env overrides.
- 中文：設定分離：交易參數與運維參數（資料庫、日誌）。  
  English: Separate trading params from ops params (DB/logging).

## 3) Secrets / 金鑰與安全
- 中文：API Key 使用環境變數或祕密管理工具。  
  English: Use env vars or secret manager for API keys.
- 中文：避免將私鑰/金鑰寫入檔案。  
  English: Do not store secrets in files.
- 中文：本機可用 `.env.local` 管理敏感資訊，禁止提交到版本庫。  
  English: Use `.env.local` for local secrets; never commit it.

Live trading env vars / 實盤環境變數
- `MERROW_BINANCE_API_KEY`, `MERROW_BINANCE_API_SECRET`  
- `MERROW_BINANCE_BASE_URL` (optional), `MERROW_BINANCE_RECV_WINDOW` (optional)  
- `MERROW_CASH_ASSET` (optional; required if quote asset cannot be inferred)
- `MERROW_PAPER_STATE_PATH` (optional; paper mode state file path)
- `MERROW_PG_ENABLED` (optional; enable PGSQL persistence)
- `MERROW_PG_INIT_SCHEMA` (optional; create schema on startup)

Env var table / 環境變數表
```text
Name                          Required  Default                     Notes
MERROW_BINANCE_API_KEY        yes       -                           Binance API key
MERROW_BINANCE_API_SECRET     yes       -                           Binance API secret
MERROW_BINANCE_BASE_URL       no        https://api.binance.com      Override REST base URL
MERROW_BINANCE_RECV_WINDOW    no        5000                        recvWindow in ms
MERROW_BYBIT_API_KEY          yes       -                           Bybit API key
MERROW_BYBIT_API_SECRET       yes       -                           Bybit API secret
MERROW_BYBIT_BASE_URL         no        https://api.bybit.com        Override REST base URL
MERROW_BYBIT_RECV_WINDOW      no        5000                        recvWindow in ms
MERROW_BYBIT_ACCOUNT_TYPE     no        UNIFIED                     Wallet balance account type
MERROW_OKX_API_KEY            yes       -                           OKX API key
MERROW_OKX_API_SECRET         yes       -                           OKX API secret
MERROW_OKX_PASSPHRASE         yes       -                           OKX API passphrase
MERROW_OKX_BASE_URL           no        https://www.okx.com          Override REST base URL
MERROW_CASH_ASSET             no        -                           Set when quote asset can't be inferred
MERROW_PAPER_STATE_PATH       no        output/paper_state.json      Paper account state path
MERROW_PG_ENABLED             no        0                           Enable PGSQL persistence
MERROW_PG_INIT_SCHEMA         no        1                           Auto-create schema
MERROW_LIVE_RETRY_MAX          no        3                           Live retry count
MERROW_LIVE_RETRY_BASE_MS      no        500                         Live retry base delay (ms)
MERROW_LIVE_RETRY_MAX_DELAY_MS no        8000                        Live retry max delay (ms)
MERROW_LIVE_RETRY_JITTER_PCT   no        20                          Live retry jitter percent
MERROW_LOG                     no        info                        Log level (info/debug)
MERROW_LOG_FORMAT              no        plain                       Log format (plain/json)
MERROW_METRICS_PATH            no        output/metrics.prom          Prometheus textfile metrics path
MERROW_UI_ADDR                 no        127.0.0.1:8088               UI API bind address
```

## 4) Logging / 日誌
- 中文：日誌分級：info/warn/error。  
  English: Log levels: info/warn/error.
- 中文：關鍵事件：下單、成交、錯誤、重試。  
  English: Key events: order placement, fills, errors, retries.

## 5) Monitoring & Alerts / 監控與告警
- 中文：監控策略狀態、資金、下單失敗率。  
  English: Monitor strategy status, balances, and order failure rates.
- 中文：告警條件：重試超過 N 次、API 失敗率高。  
  English: Alerts for retry bursts and high API error rate.

## 6) Backup & Retention / 備份與保存
- 中文：PGSQL 每日備份，保留 30 天（可調）。  
  English: Daily PGSQL backup, retain 30 days (configurable).
- 中文：回測結果可長期保存作為策略比較基準。  
  English: Keep backtest results for benchmarking.

## 7) Incident Response / 事件應變
- 中文：交易所異常 -> 暫停交易，切換至 Paper。  
  English: Exchange outage -> pause trading, switch to paper.
- 中文：資料庫失效 -> 快速切換備援或暫停。  
  English: DB failure -> switch to backup or pause.

## 8) Upgrade Process / 升級流程
- 中文：先在回測與模擬模式驗證，再進實盤。  
  English: Validate in backtest and paper before live.
- 中文：保留舊版本配置可回滾。  
  English: Keep previous configs for rollback.

## 9) Safe Live Rollout / 安全上線清單
- 中文：先啟用 dry-run，觀察日誌與訊號。  
  English: Start with dry-run; observe logs and signals.
- 中文：選小資金或小交易對測試。  
  English: Test with small capital or low-risk symbol.
- 中文：限制觸發頻率，避免連續下單。  
  English: Limit trigger frequency to avoid burst orders.
- 中文：設定告警（錯誤率、下單失敗）。  
  English: Configure alerts (error rate, order failures).
- 中文：首次實盤需人工監控。  
  English: Supervise the first live session manually.

## 10) Dry-run Monitoring Example / Dry-run 監控範例
中文：以下為 dry-run 常見的 log 與建議監控欄位。  
English: Example dry-run logs and suggested counters.

Example log lines / 範例日誌
```text
live: dry-run mode (no orders sent)
dry_order: symbol=BTCUSDT side=Buy qty=0.00500000
```

Suggested counters / 建議監控指標
- `signals_total`：策略訊號數量  
  Strategy signal count
- `orders_planned_total`：預計下單數量  
  Planned orders count
- `orders_blocked_risk_total`：風控擋單數  
  Orders blocked by risk rules
- `trigger_fire_total`：觸發器啟動次數  
  Trigger fire count
- `errors_total`：錯誤總數  
  Error count

## 11) Live-mode Risk/Stop Conditions / 實盤風控與停機條件
- 中文：連續 N 次下單失敗 -> 立即停止交易。  
  English: Stop trading after N consecutive order failures.
- 中文：API 連續 `5xx` 或 `429` 超過門檻 -> 降頻或停機。  
  English: Reduce frequency or stop after too many `5xx`/`429`.
- 中文：資金異常變動（> X%）-> 停機並人工檢查。  
  English: Stop and investigate on abnormal balance drift (> X%).
- 中文：價格跳空或 K 線異常 -> 暫停策略。  
  English: Pause strategy on abnormal price gaps.
- 中文：風控擋單比例過高 -> 檢查參數或停機。  
  English: Investigate or stop if risk blocks too many orders.
