CREATE EXTENSION IF NOT EXISTS timescaledb;

CREATE TABLE daily_prices (

    symbol TEXT NOT NULL,

    series TEXT NOT NULL,

    trade_date DATE NOT NULL,

    open_price DOUBLE PRECISION,

    high_price DOUBLE PRECISION,

    low_price DOUBLE PRECISION,

    close_price DOUBLE PRECISION,

    volume BIGINT,

    PRIMARY KEY(
        symbol,
        series,
        trade_date
    )
);

SELECT create_hypertable(
    'daily_prices',
    'trade_date'
);