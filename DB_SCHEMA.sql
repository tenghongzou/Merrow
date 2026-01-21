-- Merrow PostgreSQL schema
-- Merrow PostgreSQL 資料庫結構

-- Prices (candles)
CREATE TABLE IF NOT EXISTS prices (
    symbol TEXT NOT NULL,
    interval TEXT NOT NULL,
    time TIMESTAMPTZ NOT NULL,
    open DOUBLE PRECISION NOT NULL,
    high DOUBLE PRECISION NOT NULL,
    low DOUBLE PRECISION NOT NULL,
    close DOUBLE PRECISION NOT NULL,
    volume DOUBLE PRECISION NOT NULL,
    PRIMARY KEY (symbol, interval, time)
);

-- Orders
CREATE TABLE IF NOT EXISTS orders (
    id TEXT PRIMARY KEY,
    time TIMESTAMPTZ NOT NULL,
    mode TEXT NOT NULL CHECK (mode IN ('backtest', 'paper', 'live')),
    symbol TEXT NOT NULL,
    side TEXT NOT NULL CHECK (side IN ('buy', 'sell')),
    order_type TEXT NOT NULL CHECK (order_type IN ('market', 'limit')),
    price DOUBLE PRECISION,
    qty DOUBLE PRECISION NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('new', 'partially_filled', 'filled', 'canceled', 'rejected')),
    exchange_order_id TEXT,
    client_order_id TEXT
);

-- Trades
CREATE TABLE IF NOT EXISTS trades (
    id TEXT PRIMARY KEY,
    order_id TEXT NOT NULL REFERENCES orders(id),
    time TIMESTAMPTZ NOT NULL,
    symbol TEXT NOT NULL,
    side TEXT NOT NULL CHECK (side IN ('buy', 'sell')),
    price DOUBLE PRECISION NOT NULL,
    qty DOUBLE PRECISION NOT NULL,
    fee DOUBLE PRECISION NOT NULL,
    fee_asset TEXT
);

-- Positions (snapshots)
CREATE TABLE IF NOT EXISTS positions (
    time TIMESTAMPTZ NOT NULL,
    mode TEXT NOT NULL CHECK (mode IN ('backtest', 'paper', 'live')),
    symbol TEXT NOT NULL,
    qty DOUBLE PRECISION NOT NULL,
    avg_price DOUBLE PRECISION NOT NULL,
    PRIMARY KEY (time, mode, symbol)
);

-- Balances (snapshots)
CREATE TABLE IF NOT EXISTS balances (
    time TIMESTAMPTZ NOT NULL,
    mode TEXT NOT NULL CHECK (mode IN ('backtest', 'paper', 'live')),
    asset TEXT NOT NULL,
    free DOUBLE PRECISION NOT NULL,
    locked DOUBLE PRECISION NOT NULL,
    PRIMARY KEY (time, mode, asset)
);

-- Backtest runs
CREATE TABLE IF NOT EXISTS backtest_runs (
    id TEXT PRIMARY KEY,
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ NOT NULL,
    params JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Backtest metrics
CREATE TABLE IF NOT EXISTS backtest_metrics (
    run_id TEXT PRIMARY KEY REFERENCES backtest_runs(id),
    return DOUBLE PRECISION NOT NULL,
    max_drawdown DOUBLE PRECISION NOT NULL,
    win_rate DOUBLE PRECISION NOT NULL,
    trade_count INTEGER NOT NULL,
    sharpe DOUBLE PRECISION
);

-- Signals (optional, for auditing)
CREATE TABLE IF NOT EXISTS signals (
    time TIMESTAMPTZ NOT NULL,
    mode TEXT NOT NULL CHECK (mode IN ('backtest', 'paper', 'live')),
    symbol TEXT NOT NULL,
    signal TEXT NOT NULL CHECK (signal IN ('buy', 'sell', 'hold')),
    reason TEXT,
    PRIMARY KEY (time, mode, symbol)
);

-- Indexes
CREATE INDEX IF NOT EXISTS prices_time_idx ON prices (time);
CREATE INDEX IF NOT EXISTS orders_symbol_time_idx ON orders (symbol, time);
CREATE INDEX IF NOT EXISTS trades_symbol_time_idx ON trades (symbol, time);
