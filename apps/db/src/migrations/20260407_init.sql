

-- 1. User Table (Core Identity)--
CREATE TABLE IF NOT EXISTS users (
    id BIGINT PRIMARY KEY, -- u64 (engine_id) maps here 
    username TEXT UNIQUE NOT NULL,
    email TEXT UNIQUE NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- 2. Balances Table (The Ledger)
CREATE TABLE IF NOT EXISTS balances (
    user_id BIGINT REFERENCES users(id),
    asset TEXT NOT NULL, -- 'BTC', 'ETH', 'USDT'
    available NUMERIC(38, 18) DEFAULT 0,
    locked NUMERIC(38, 18) DEFAULT 0,
    PRIMARY KEY (user_id, asset)
);

-- 3. Trade History (TimescaleDB Hypertable)
CREATE TABLE IF NOT EXISTS trade_history (
    time TIMESTAMPTZ NOT NULL,
    symbol TEXT NOT NULL, -- 'BTC_USDT' or 'ETH_USDT'
    price NUMERIC(38, 18) NOT NULL,
    volume NUMERIC(38, 18) NOT NULL,
    taker_side TEXT NOT NULL
);

SELECT create_hypertable('trade_history', 'time', if_not_exists => TRUE);

-- 4. Open Orders (For Crash Recovery)
CREATE TABLE IF NOT EXISTS open_orders (
    order_id BIGINT PRIMARY KEY,
    user_id BIGINT REFERENCES users(id),
    symbol TEXT NOT NULL,
    side TEXT NOT NULL,
    price NUMERIC(38, 18) NOT NULL,
    quantity NUMERIC(38, 18) NOT NULL,
    filled NUMERIC(38, 18) DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- 5. Klines (Continuous Aggregates)
-- This is better than Materialized Views because it updates AUTOMATICALLY
CREATE MATERIALIZED VIEW IF NOT EXISTS klines_1m 
WITH (timescaledb.continuous) AS
SELECT
    time_bucket('1 minute', time) AS bucket,
    symbol,
    first(price, time) AS open,
    max(price) AS high,
    min(price) AS low,
    last(price, time) AS close,
    sum(volume) AS volume
FROM trade_history
GROUP BY bucket, symbol;

CREATE MATERIALIZED VIEW IF NOT EXISTS klines_5m 
WITH (timescaledb.continuous) AS
SELECT
    time_bucket('5 minute', time) AS bucket,
    symbol,
    first(price, time) AS open,
    max(price) AS high,
    min(price) AS low,
    last(price, time) AS close,
    sum(volume) AS volume
FROM trade_history
GROUP BY bucket, symbol;

CREATE MATERIALIZED VIEW IF NOT EXISTS klines_15m 
WITH (timescaledb.continuous) AS
SELECT
    time_bucket('15 minute', time) AS bucket,
    symbol,
    first(price, time) AS open,
    max(price) AS high,
    min(price) AS low,
    last(price, time) AS close,
    sum(volume) AS volume
FROM trade_history
GROUP BY bucket, symbol;
