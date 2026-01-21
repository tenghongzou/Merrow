# Merrow Quant Trading System (Rust)
# Merrow 量化交易系統（Rust）

中文：Merrow 是以 Rust 實作的量化交易系統，支援回測（MVP）、模擬、實盤與 UI 儀表板。  
English: Merrow is a Rust-based quant trading system with backtest (MVP), paper, live trading, and a UI dashboard.

## Quick Start / 快速開始

Build / 編譯
```bash
cargo build
```

Backtest / 回測
```bash
cargo run --bin merrow -- backtest --config config.toml
```

Paper / 模擬
```bash
cargo run --bin merrow -- paper --config config.toml
```

Live / 實盤
```bash
cargo run --bin merrow -- live --config config.toml
```

UI API / UI 後端
```bash
cargo run --bin merrow_ui -- --config config.toml --addr 127.0.0.1:8088
```

UI Frontend / UI 前端
```bash
cd ui
pnpm install
pnpm dev
```

## Configuration / 設定

中文：參考 `config.example.toml` 與 `.env.example`。  
English: See `config.example.toml` and `.env.example`.

## Docs / 文件

Main documents are under `docs/`.

- `docs/USER_MANUAL.md` (使用手冊 / User Manual)
- `docs/DEVELOPMENT_MANUAL.md` (開發手冊 / Development Manual)
- `docs/ARCHITECTURE.md`
- `docs/SPEC.md`
- `docs/OPERATIONS.md`
- `docs/DECISIONS.md`
- `docs/UI_SWOT.md`

## Structure / 專案結構

```text
merrow/
  src/           Rust core
  tests/         Tests
  ui/            Svelte UI
  docs/          Documentation
```

## Notes / 備註
- 中文：量化交易有風險，請先回測與模擬，再評估實盤風險。  
  English: Trading is risky; validate in backtest/paper before going live.
