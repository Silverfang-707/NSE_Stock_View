-- =====================================
-- MULTI-TIMEFRAME MARKET LEVELS
-- =====================================

CREATE TABLE market_levels (

    symbol TEXT NOT NULL,

    series TEXT NOT NULL,

    timeframe TEXT NOT NULL,

    trade_date DATE NOT NULL,

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

CREATE INDEX idx_market_levels_symbol_tf_date

ON market_levels(

    symbol,

    timeframe,

    trade_date DESC
);

-- =====================================
-- HYPERTABLE
-- =====================================

SELECT create_hypertable(

    'market_levels',

    'trade_date',

    if_not_exists => TRUE
);