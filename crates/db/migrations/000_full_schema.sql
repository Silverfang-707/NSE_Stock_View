-- =====================================
-- NSE MARKET ANALYTICS PLATFORM
-- COMPLETE DATABASE SCHEMA
-- =====================================

-- =====================================
-- EXTENSIONS
-- =====================================

CREATE EXTENSION IF NOT EXISTS timescaledb;

-- =====================================
-- DROP OLD LEGACY TABLES
-- =====================================

DROP TABLE IF EXISTS daily_levels CASCADE;

DROP TABLE IF EXISTS market_levels CASCADE;

DROP TABLE IF EXISTS symbol_indices CASCADE;

DROP TABLE IF EXISTS instruments CASCADE;

DROP TABLE IF EXISTS daily_prices CASCADE;

DROP TABLE IF EXISTS users CASCADE;

-- =====================================
-- INSTRUMENTS
-- =====================================

CREATE TABLE instruments (

    id SERIAL PRIMARY KEY,

    symbol TEXT NOT NULL,

    series TEXT NOT NULL,

    company_name TEXT,

    sector TEXT,

    industry TEXT,

    isin TEXT,

    UNIQUE(symbol, series)
);

-- =====================================
-- DAILY MARKET PRICES
-- =====================================

CREATE TABLE daily_prices (

    symbol TEXT NOT NULL,

    series TEXT NOT NULL,

    trade_date DATE NOT NULL,

    open_price DOUBLE PRECISION,

    high_price DOUBLE PRECISION,

    low_price DOUBLE PRECISION,

    close_price DOUBLE PRECISION,

    volume BIGINT,

    PRIMARY KEY (

        symbol,

        series,

        trade_date
    )
);

SELECT create_hypertable(

    'daily_prices',

    'trade_date',

    if_not_exists => TRUE
);

-- =====================================
-- DAILY PRICE INDEXES
-- =====================================

CREATE INDEX idx_symbol_date

ON daily_prices(

    symbol,

    trade_date DESC
);

CREATE INDEX idx_series_symbol

ON daily_prices(

    series,

    symbol
);

CREATE INDEX idx_trade_date

ON daily_prices(

    trade_date DESC
);

-- =====================================
-- SYMBOL INDICES
-- =====================================

CREATE TABLE symbol_indices (

    id SERIAL PRIMARY KEY,

    instrument_id INTEGER
    REFERENCES instruments(id),

    index_name TEXT NOT NULL
);

-- =====================================
-- USERS + RBAC
-- =====================================

CREATE TABLE users (

    id UUID PRIMARY KEY
    DEFAULT gen_random_uuid(),

    username TEXT NOT NULL UNIQUE,

    password_hash TEXT NOT NULL,

    role TEXT NOT NULL
    DEFAULT 'viewer',

    is_admin BOOLEAN NOT NULL
    DEFAULT FALSE,

    is_root BOOLEAN NOT NULL
    DEFAULT FALSE,

    created_at TIMESTAMPTZ NOT NULL
    DEFAULT NOW()
);

CREATE INDEX idx_users_username
ON users(username);

CREATE INDEX idx_users_role
ON users(role);

CREATE INDEX idx_users_root
ON users(is_root);

-- =====================================
-- MULTI-TIMEFRAME MARKET LEVELS
-- =====================================

CREATE TABLE market_levels (

    symbol TEXT NOT NULL,

    series TEXT NOT NULL,

    timeframe TEXT NOT NULL,

    trade_date DATE NOT NULL,

    period_start DATE NOT NULL,

    period_end DATE NOT NULL,

    open_price DOUBLE PRECISION,

    high_price DOUBLE PRECISION,

    low_price DOUBLE PRECISION,

    close_price DOUBLE PRECISION,

    range_value DOUBLE PRECISION,

    buffer_value DOUBLE PRECISION,

    jgd DOUBLE PRECISION,

    jwd DOUBLE PRECISION,

    bdp DOUBLE PRECISION,

    wdp DOUBLE PRECISION,

    pattern TEXT,

    calculation_version INTEGER
    DEFAULT 1,

    created_at TIMESTAMPTZ
    DEFAULT NOW(),

    updated_at TIMESTAMPTZ
    DEFAULT NOW(),

    PRIMARY KEY (

        symbol,

        series,

        timeframe,

        trade_date
    )
);

SELECT create_hypertable(

    'market_levels',

    'trade_date',

    if_not_exists => TRUE
);

-- =====================================
-- MARKET LEVEL INDEXES
-- =====================================

CREATE INDEX idx_market_levels_symbol
ON market_levels(symbol);

CREATE INDEX idx_market_levels_series
ON market_levels(series);

CREATE INDEX idx_market_levels_timeframe
ON market_levels(timeframe);

CREATE INDEX idx_market_levels_trade_date
ON market_levels(trade_date DESC);

CREATE INDEX idx_market_levels_period_start
ON market_levels(period_start DESC);

CREATE INDEX idx_market_levels_period_end
ON market_levels(period_end DESC);

CREATE INDEX idx_market_levels_symbol_tf_date

ON market_levels(

    symbol,

    timeframe,

    trade_date DESC
);

CREATE INDEX idx_market_levels_symbol_series_tf

ON market_levels(

    symbol,

    series,

    timeframe
);

-- =====================================
-- UPDATED_AT TRIGGER
-- =====================================

CREATE OR REPLACE FUNCTION
update_updated_at_column()

RETURNS TRIGGER AS $$

BEGIN

    NEW.updated_at = NOW();

    RETURN NEW;
END;

$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS
set_market_levels_updated_at
ON market_levels;

CREATE TRIGGER
set_market_levels_updated_at

BEFORE UPDATE
ON market_levels

FOR EACH ROW

EXECUTE FUNCTION
update_updated_at_column();