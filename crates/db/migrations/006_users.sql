-- 006_users.sql

CREATE TABLE IF NOT EXISTS users (

    id SERIAL PRIMARY KEY,

    username TEXT NOT NULL UNIQUE,

    password_hash TEXT NOT NULL,

    role TEXT NOT NULL
    DEFAULT 'admin',

    is_admin BOOLEAN NOT NULL
    DEFAULT TRUE,

    created_at TIMESTAMPTZ NOT NULL
    DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_users_username
ON users(username);