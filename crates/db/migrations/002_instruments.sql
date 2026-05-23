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