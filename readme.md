# Airaa Stock Handler

A full-stack stock market analytics platform built with Rust, PostgreSQL, TimescaleDB, and a modern web frontend.

---

# Features

* NSE Bhav Copy ingestion
* Multi-timeframe calculations

  * Daily
  * Weekly
  * Monthly
  * Quarterly
  * Half-Yearly
  * Yearly
* Pattern detection

  * 2+2
  * 2+1
  * 3+1
* JGD / JWD calculations
* BDP / WDP calculations
* Historical backfill support
* Admin dashboard
* JWT Authentication
* User management
* TimescaleDB optimized storage

---

# Project Structure

```text
airaastockhandler/

├── apps
│   ├── api
│   ├── calculator
│   ├── frontend
│   └── ingester
│
├── crates
│   ├── db
│   │   └── migrations
│   │       ├── 000_full_schema.sql
│   │       ├── ...
│   │
│   ├── indicators
│   ├── market_core
│   └── patterns
│
├── .env
├── Cargo.toml
└── docker-compose.yml
```

---

# Requirements

Install the following:

## Rust

https://rustup.rs

Verify:

```bash
rustc --version
cargo --version
```

---

## Node.js

Install latest LTS version:

https://nodejs.org

Verify:

```bash
node -v
npm -v
```

---

## PostgreSQL

Install PostgreSQL 16+.

Verify:

```bash
psql --version
```

---

## TimescaleDB

Install TimescaleDB extension for PostgreSQL.

Verify inside PostgreSQL:

```sql
CREATE EXTENSION IF NOT EXISTS timescaledb;
```

---

# Database Setup

Create database:

```sql
CREATE DATABASE nse_market;
```

Connect:

```bash
psql -U postgres -d nse_market
```

Run the schema:

```bash
psql -U postgres -d nse_market -f crates/db/migrations/000_full_schema.sql
```

This will create:

* users
* instruments
* daily_prices
* market_levels
* symbol_indices

along with all indexes, triggers, and TimescaleDB hypertables.

---

# Environment Variables

Create a `.env` file in the project root.

Example:

```env
DATABASE_URL=postgres://postgres:password@localhost:5432/nse_market

JWT_SECRET=replace_this_with_a_long_random_secret
```

---

# Build Workspace

From project root:

```bash
cargo build --workspace
```

For release builds:

```bash
cargo build --workspace --release
```

---

# Frontend Setup

Navigate to frontend:

```bash
cd apps/frontend
```

Install dependencies:

```bash
npm install
```

Run development server:

```bash
npm run dev
```

Default URL:

```text
http://localhost:5173
```

---

# Running the API

From project root:

```bash
cargo run -p api
```

Expected output:

```text
✅ Database Connected
🚀 API running on http://localhost:3000
```

API Base URL:

```text
http://localhost:3000
```

---

# Initial Data Import

The database is empty after installation.

Import market data:

```bash
cargo run -p ingester
```

This downloads and stores NSE daily price data.

---

# Generate Market Levels

After data is imported:

```bash
cargo run -p calculator
```

This generates:

* JGD
* JWD
* BDP
* WDP
* Patterns

for all supported timeframes.

---

# Admin Login

Create an admin user using the API:

```http
POST /admin/create-user
```

Or insert one directly into the database if required.

Once created:

```http
POST /auth/login
```

returns a JWT token.

Use:

```http
Authorization: Bearer <token>
```

for protected admin routes.

---

# Updating Market Data

The preferred method is through the Admin Dashboard.

The update process:

```text
Update Market
        ↓
Run Ingest
        ↓
Run Calculator
        ↓
Database Updated
```

This keeps all levels synchronized.

---

# Backfill Historical Data

Backfill a specific date range:

```http
GET /admin/backfill?from=2025-01-01&to=2025-12-31
```

Backfill an entire year:

```http
GET /admin/backfill?year=2025
```

---

# Common Commands

## Run API

```bash
cargo run -p api
```

## Run Calculator

```bash
cargo run -p calculator
```

## Run Ingester

```bash
cargo run -p ingester
```

## Build Everything

```bash
cargo build --workspace
```

## Release Build

```bash
cargo build --workspace --release
```

---

# Troubleshooting

## Database Connection Error

Verify:

```env
DATABASE_URL
```

matches your PostgreSQL configuration.

---

## TimescaleDB Error

Ensure TimescaleDB is installed and enabled:

```sql
CREATE EXTENSION IF NOT EXISTS timescaledb;
```

---

## Login Returns 401

Verify:

* User exists
* Password is correct
* JWT secret matches configuration

---

## Frontend Cannot Reach API

Verify API is running:

```text
http://localhost:3000
```

and frontend environment variables point to the correct backend URL.

---

# Production Deployment

Recommended:

* Rust Release Build
* PostgreSQL + TimescaleDB
* Reverse Proxy (Nginx)
* HTTPS
* Automated Database Backups

Release build:

```bash
cargo build --workspace --release
```

Run binaries from:

```text
target/release/
```

---

# License

Private Project

Airaa Stock Handler
Market Analytics Platform
