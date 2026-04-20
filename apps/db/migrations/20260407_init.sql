
-- sqlx-no-tx

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

ALTER TABLE balances 
ADD CONSTRAINT positive_available CHECK (available >= 0);

ALTER TABLE balances 
ADD CONSTRAINT positive_locked CHECK (locked >= 0);

-- 3. Trade History (TimescaleDB Hypertable)
CREATE TABLE IF NOT EXISTS trade_history (
    time TIMESTAMPTZ NOT NULL,
    symbol TEXT NOT NULL, -- 'BTC_USDT' or 'ETH_USDT'
    price NUMERIC(38, 18) NOT NULL,
    volume NUMERIC(38, 18) NOT NULL,

    -- Accountabality fields
    taker_user_id BIGINT NOT NULL,
    maker_user_id BIGINT NOT NULL,
    taker_order_id BIGINT NOT NULL,
    maker_order_id BIGINT NOT NULL,

    taker_side TEXT NOT NULL
);

SELECT create_hypertable('trade_history', 'time', if_not_exists => TRUE);

-- ADD INDEXES INSTEAD OF FOREIGN KEYS
-- This allows "My Trades" queries to be instant (log time)
CREATE INDEX IF NOT EXISTS idx_trade_taker_user ON trade_history (taker_user_id, time DESC);
CREATE INDEX IF NOT EXISTS idx_trade_maker_user ON trade_history (maker_user_id, time DESC);


-- 4. Open Orders (For Crash Recovery)
CREATE TABLE IF NOT EXISTS open_orders (
    order_id BIGINT PRIMARY KEY,
    user_id BIGINT NOT NULL,
    symbol TEXT NOT NULL,
    side TEXT NOT NULL,
    price NUMERIC(38, 18) NOT NULL,
    quantity NUMERIC(38, 18) NOT NULL,
    filled NUMERIC(38, 18) DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_open_orders_user ON open_orders (user_id);
-- ALTER TABLE open_orders ADD CONSTRAINT filled_not_exceed_qty CHECK (filled <= quantity);

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
GROUP BY bucket, symbol
WITH NO DATA;

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
GROUP BY bucket, symbol
WITH NO DATA;

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
GROUP BY bucket, symbol
WITH NO DATA;


--- 6. Automation Policies (The "Set and Forget") ---

-- Refresh 1m candles every minute
SELECT add_continuous_aggregate_policy('klines_1m',
    start_offset => INTERVAL '2 minutes',
    end_offset => INTERVAL '0 seconds',
    schedule_interval => INTERVAL '1 minute');

-- Refresh 5m candles every 5 minutes
SELECT add_continuous_aggregate_policy('klines_5m',
    start_offset => INTERVAL '10 minutes',
    end_offset => INTERVAL '0 seconds',
    schedule_interval => INTERVAL '5 minutes');

-- Refresh 15m candles every 15 minutes
SELECT add_continuous_aggregate_policy('klines_15m',
    start_offset => INTERVAL '30 minutes',
    end_offset => INTERVAL '0 seconds',
    schedule_interval => INTERVAL '15 minutes');