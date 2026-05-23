-- 005_daily_levels.sql

CREATE TABLE daily_levels (
    symbol      TEXT NOT NULL,
    series      TEXT NOT NULL,
    trade_date  DATE NOT NULL,

    range       DOUBLE PRECISION,

    jgd         DOUBLE PRECISION,
    jwd         DOUBLE PRECISION,

    bdp         DOUBLE PRECISION,
    wdp         DOUBLE PRECISION,

    pattern     TEXT,

    PRIMARY KEY (symbol, series, trade_date)
);

SELECT create_hypertable(
    'daily_levels',
    'trade_date'
);

CREATE INDEX idx_levels_symbol_date
ON daily_levels(symbol, trade_date DESC);