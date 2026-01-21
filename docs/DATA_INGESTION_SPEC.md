# Data Ingestion and Validation Spec
# 資料匯入與驗證規格

中文：本文件規範 CSV 與交易所歷史資料的匯入格式、驗證與清洗流程。  
English: This document specifies CSV and exchange data ingestion formats, validation, and cleaning steps.

## 1) CSV Schema / CSV 格式
必填欄位 / Required fields
- time (RFC3339 or epoch seconds, UTC)
- open, high, low, close (float)
- volume (float)

可選欄位 / Optional fields
- symbol (string)  
- interval (string, e.g., 1m, 5m)

## 2) Time Rules / 時間規則
- 中文：所有時間統一視為 UTC。  
  English: All timestamps are treated as UTC.
- 中文：資料需單調遞增，若非遞增則排序後匯入。  
  English: Data must be monotonic; if not, sort before import.
- 中文：時間重複時保留最後一筆。  
  English: On duplicate timestamps, keep the last row.

## 3) Value Validation / 值檢查
- high >= max(open, close)  
- low <= min(open, close)  
- volume >= 0  
- price > 0

中文：不符合者丟棄並記錄錯誤。  
English: Invalid rows are dropped and logged.

## 4) Missing Data / 缺資料
- 中文：預設不補值，直接跳過該區間並記錄。  
  English: Default is no fill; skip gaps and log.
- 中文：若啟用 `fill_gaps`，以前一根 close 補 open/high/low/close，volume=0。  
  English: If `fill_gaps` enabled, forward-fill OHLC with volume=0.

## 5) Exchange Data Fetch / 交易所資料下載
- 中文：支援依 `symbol + interval + start/end` 分批下載。  
  English: Support paginated fetch by symbol/interval/start/end.
- 中文：需遵守交易所 rate limit，並記錄抓取範圍。  
  English: Respect rate limits and record fetch ranges.
- 中文：下載後需進行同樣的驗證與清洗流程。  
  English: Apply the same validation/cleaning after download.
- 中文：Base URL 與單次上限可由設定提供（如 `data.exchange_base_url`/`data.exchange_limit`）。  
  English: Base URL and page limit can be configured (e.g., `data.exchange_base_url`/`data.exchange_limit`).
- 中文：可支援多交易所（如 Binance/Bybit/OKX），由 `exchange` 與設定控制。  
  English: Multiple exchanges (e.g., Binance/Bybit/OKX) can be selected via `exchange` and config.
- 中文：OKX `bar` 週期需依規則映射（例如 `1h` -> `1H`，`1d` -> `1D`）。  
  English: OKX `bar` intervals require mapping (e.g., `1h` -> `1H`, `1d` -> `1D`).

## 6) Storage / 儲存規則
- 中文：寫入 PGSQL 時使用 upsert（避免重複）。  
  English: Use upsert when writing to PostgreSQL.
- 中文：若同時間點已存在資料，以新資料覆蓋。  
  English: Overwrite existing rows with new data.

## 7) Data Provenance / 資料來源紀錄
- 中文：保存資料來源、下載時間與參數，以利追蹤。  
  English: Record source, download time, and params for traceability.
