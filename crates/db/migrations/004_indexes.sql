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