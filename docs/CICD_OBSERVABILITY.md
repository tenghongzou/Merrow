# CI/CD and Observability Spec
# CI/CD 與可觀測性規格

中文：本文件定義 CI/CD 流程、版本控制與監控規範。  
English: This document defines CI/CD workflow, release/versioning, and observability standards.

## 1) CI Pipeline / CI 流程
中文：每次 PR 或 push 需執行以下步驟。  
English: Each PR/push runs:
- `cargo fmt -- --check`  
- `cargo clippy -- -D warnings`  
- `cargo test`  
- `cargo build --release`

## 2) CD Pipeline / CD 流程
- 中文：主分支通過後產出 release build。  
  English: Main branch builds release artifacts.
- 中文：release 需附上版本與變更摘要。  
  English: Releases include version and changelog.

## 3) Versioning / 版本策略
- 中文：採用 SemVer（MAJOR.MINOR.PATCH）。  
  English: Use SemVer (MAJOR.MINOR.PATCH).
- 中文：策略變更與風控規則修改視為 MINOR 或 MAJOR。  
  English: Strategy/risk changes are MINOR or MAJOR.

## 4) Environments / 環境
- 中文：至少三個環境：dev、paper、live。  
  English: At least dev, paper, live environments.
- 中文：不同環境使用不同 API Key 與資料庫。  
  English: Separate API keys and DB per environment.

## 5) Secrets / 機密管理
- 中文：API Key 只能透過環境變數或秘密管理工具。  
  English: API keys only via env vars or secret manager.
- 中文：CI 不得輸出敏感資訊。  
  English: CI logs must not leak secrets.

## 6) Logging / 日誌
- 中文：使用結構化日誌（JSON），包含 order_id、symbol、mode。  
  English: Use structured logs (JSON) with order_id, symbol, mode.
- 中文：錯誤需包含 error_code 與可重試標記。  
  English: Errors include error_code and retryable flag.

## 7) Metrics / 指標
- 交易成功率 / Order success rate  
- 平均成交延遲 / Avg fill latency  
- 失敗率 / Failure rate  
- 觸發次數 / Trigger count  
- 回測耗時 / Backtest duration

## 8) Alerts / 告警
- 中文：API 失敗率 > 5% 持續 5 分鐘。  
  English: API failure rate > 5% for 5 minutes.
- 中文：下單失敗連續 N 次。  
  English: N consecutive order failures.

## 9) Audit / 稽核
- 中文：訂單、成交、持倉變化需可追溯。  
  English: Orders, trades, and positions must be traceable.
