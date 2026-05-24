-- =====================================
-- DROP OLD TABLE
-- =====================================

DROP TABLE IF EXISTS market_levels CASCADE;

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

-- =====================================
-- INDEXES
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
-- HYPERTABLE
-- =====================================

SELECT create_hypertable(

    'market_levels',

    'trade_date',

    if_not_exists => TRUE
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