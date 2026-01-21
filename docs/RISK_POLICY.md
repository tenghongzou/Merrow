# Risk Policy and Limits
# 風控政策與限制

中文：本文件定義風控規則與限制，避免策略失控與資金過度暴露。  
English: This document defines risk limits to prevent excessive exposure and loss.

## 1) Core Limits / 核心限制
- max_trade_ratio: 單筆最大交易比例（預設 0.5）。  
  max_trade_ratio: max fraction per order (default 0.5).
- min_cash_reserve_ratio: 最低現金保留比例（預設 0.05）。  
  min_cash_reserve_ratio: minimum cash reserve (default 0.05).
- max_position_value_ratio: 最大持倉價值佔比（預設 0.8）。  
  max_position_value_ratio: max position value share (default 0.8).

## 2) Exposure Control / 風險暴露
- 中文：若持倉價值超過上限，禁止再買入。  
  English: Disallow further buys when exposure exceeds limit.
- 中文：若現金低於保留比例，禁止買入。  
  English: Block buys when cash below reserve threshold.

## 3) Order Frequency / 下單頻率
- 中文：限制每小時最大下單次數（預設 20）。  
  English: Cap max orders per hour (default 20).
- 中文：限制連續下單，若連續失敗 N 次則停機。  
  English: Pause trading after N consecutive failures.

## 4) Drawdown Guard / 回撤保護
- 中文：若最大回撤超過門檻（如 10%），自動暫停交易。  
  English: Pause trading if max drawdown exceeds threshold (e.g., 10%).

## 5) Price Sanity / 價格合理性
- 中文：若當前價格與最近價格偏離超過 X%，拒絕下單。  
  English: Reject orders if price deviates beyond X% from last price.
- 中文：若 spread 或波動率超過門檻，暫停交易。  
  English: Pause trading on excessive spread or volatility.

## 6) Exchange Constraints / 交易所限制
- 中文：遵守最小數量、最小名目價值與價格精度。  
  English: Enforce min qty, min notional, and price precision.
- 中文：訂單不符時直接拒絕並記錄。  
  English: Reject invalid orders and log reason.

## 7) Kill Switch / 緊急停止
- 中文：可手動啟用緊急停止，立即取消所有掛單。  
  English: Manual kill switch cancels all open orders.
- 中文：交易所異常或連線失敗可自動觸發。  
  English: Auto-trigger on exchange outage or connectivity failure.
