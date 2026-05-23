CREATE TABLE symbol_indices (

    id SERIAL PRIMARY KEY,

    instrument_id INTEGER
        REFERENCES instruments(id),

    index_name TEXT NOT NULL

);