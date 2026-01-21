# Merrow Test Plan
# Merrow 測試計畫

中文：本文件定義測試範圍、測試類型、案例與驗收標準。  
English: This document defines test scope, test types, cases, and acceptance criteria.

## 0) TDD Workflow / TDD 流程
- 中文：先寫失敗測試（Red）-> 實作功能（Green）-> 重構（Refactor）。  
  English: Write failing tests (Red) -> implement (Green) -> refactor.
- 中文：每個新功能需先建立對應測試案例，再進行實作。  
  English: Each new feature must have tests before implementation.
- 中文：DDD 的不變式需具體化為測試（例如持倉與資金不可為負）。  
  English: DDD invariants must be expressed as tests (e.g., non-negative cash/position).

## 1) Test Scope / 測試範圍
- 中文：策略邏輯、觸發器、回測引擎、訂單撮合、交易所介面、PGSQL 儲存。  
  English: Strategy logic, triggers, backtest engine, order fills, exchange interface, PGSQL storage.

## 2) Test Types / 測試類型
### Unit Tests / 單元測試
- 觸發器：time trigger 的倍數與上限檢查。  
  Time trigger validation (multiple of 5, <= 100).
- 觸發模式：`any` 與 `all` 行為。  
  Trigger modes: `any` vs `all`.
- 策略規則：買 50%、賣 20%、再買 50%。  
  Strategy rules: buy/sell/re-buy ratios.
- 風控限制：最小現金保留、最大下單比例、最大持倉比例。  
  Risk limits: min cash reserve, max trade ratio, max position ratio.
- 成本：手續費與滑點計算。  
  Fee and slippage calculations.

### Integration Tests / 整合測試
- PGSQL 寫入與讀取一致性。  
  PostgreSQL read/write correctness.
- CSV 匯入資料解析與時間排序。  
  CSV parsing and time ordering.
- Exchange Adapter 模擬（使用假資料）。  
  Exchange adapter mock integration.

### Backtest Validation / 回測驗證
- 固定資料集可重現結果。  
  Reproducible results on fixed dataset.
- 限價單跨越 candle high/low 才成交。  
  Limit orders fill only when candle crosses.
- 市價單在下一根 candle 成交或同根成交（依規格）。  
  Market order fill rule as specified.

### Failure Injection / 失敗注入
- 交易所 API 超時 -> 重試與降級。  
  Exchange timeout -> retry and degrade.
- 資料缺失 -> 跳過並記錄。  
  Missing data -> skip and log.

## 3) Test Data / 測試資料
- CSV：OHLCV 時間序列，確保時間遞增。  
  CSV OHLCV with monotonic timestamps.
- 模擬行情：極端波動、平盤、跳空。  
  Simulated price regimes: volatility, flat, gaps.

## 4) Acceptance Criteria / 驗收標準
- 同一份資料與參數，回測結果一致。  
  Backtest results are deterministic.
- 觸發器行為符合規格（time + price）。  
  Triggers behave as specified.
- 市價/限價單在回測與模擬中一致。  
  Market/limit orders behave consistently across backtest and paper.
- PGSQL 存取完成且無錯誤。  
  PostgreSQL operations succeed without errors.

## 5) Manual Checks / 人工檢查
- 日誌包含下單與成交紀錄。  
  Logs include order and trade events.
- 參數驗證錯誤會明確提示。  
  Config validation errors are clear.
