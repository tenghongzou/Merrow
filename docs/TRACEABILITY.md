# Traceability Matrix
# 需求追蹤矩陣

中文：本文件將需求對應到設計文件與測試計畫，確保 TDD 與 DDD 開發可驗證。  
English: This document maps requirements to design and test artifacts for TDD/DDD verification.

## Mapping / 對應表

REQ-MODE-01 Backtest mode  
- Spec: SPEC.md (4.1)  
- Design: ARCHITECTURE.md (Backtest Flow), BACKTEST_EXECUTION_SPEC.md  
- Tests: TEST_PLAN.md (Backtest Validation)

REQ-MODE-02 Paper mode  
- Spec: SPEC.md (4.1)  
- Design: ARCHITECTURE.md (Execution Layer)  
- Tests: TEST_PLAN.md (Integration Tests)

REQ-MODE-03 Live mode  
- Spec: SPEC.md (4.1)  
- Design: EXCHANGE_CONTRACT.md, ARCHITECTURE.md (Execution Layer)  
- Tests: TEST_PLAN.md (Integration Tests)

REQ-TRG-01 Time trigger (5x, <=100)  
- Spec: SPEC.md (4.3), SPEC.md (6)  
- Design: ARCHITECTURE.md (Trigger Engine)  
- Tests: TEST_PLAN.md (Unit Tests: time trigger)

REQ-TRG-02 Price trigger (MA thresholds)  
- Spec: SPEC.md (4.3)  
- Design: ARCHITECTURE.md (Trigger Engine)  
- Tests: TEST_PLAN.md (Unit Tests: price trigger)

REQ-ORD-01 Market orders  
- Spec: SPEC.md (4.4)  
- Design: BACKTEST_EXECUTION_SPEC.md (3), EXCHANGE_CONTRACT.md  
- Tests: TEST_PLAN.md (Backtest Validation)

REQ-ORD-02 Limit orders  
- Spec: SPEC.md (4.4)  
- Design: BACKTEST_EXECUTION_SPEC.md (4), EXCHANGE_CONTRACT.md  
- Tests: TEST_PLAN.md (Unit/Backtest Validation)

REQ-DATA-01 CSV ingestion  
- Spec: SPEC.md (4.5)  
- Design: DATA_INGESTION_SPEC.md  
- Tests: TEST_PLAN.md (Integration Tests: CSV parsing)

REQ-DATA-02 Exchange historical data  
- Spec: SPEC.md (4.5)  
- Design: DATA_INGESTION_SPEC.md, EXCHANGE_CONTRACT.md  
- Tests: TEST_PLAN.md (Integration Tests)

REQ-DB-01 PostgreSQL storage  
- Spec: SPEC.md (4.6)  
- Design: DB_SCHEMA.sql  
- Tests: TEST_PLAN.md (Integration Tests: PGSQL)

REQ-RISK-01 Risk limits  
- Spec: SPEC.md (5), RISK_POLICY.md  
- Design: ARCHITECTURE.md (Risk Manager)  
- Tests: TEST_PLAN.md (Unit Tests: risk rules)

REQ-OBS-01 Observability (logs/metrics)  
- Spec: SPEC.md (5)  
- Design: CICD_OBSERVABILITY.md, OPERATIONS.md  
- Tests: TEST_PLAN.md (Manual Checks)

REQ-SEC-01 Secret management  
- Spec: SPEC.md (5)  
- Design: OPERATIONS.md, CICD_OBSERVABILITY.md  
- Tests: Manual review checklist
