# Accounting Backend (Rust Axum + PostgreSQL)

Backend REST API minimal untuk program akuntansi perusahaan:
- Chart of Accounts (COA)
- Journal Entries + Journal Lines
- Posting (draft -> posted) dengan validasi debit = kredit

## Requirements
- Rust stable
- PostgreSQL
- sqlx-cli

## Setup
1) Copy `.env.example` menjadi `.env` lalu sesuaikan DATABASE_URL
2) Buat DB:
   ```bash
   cargo install sqlx-cli --no-default-features --features postgres
   sqlx database create
   ```
3) Jalankan migrations otomatis saat start:
   ```bash
   cargo run
   ```

## Endpoints
- GET  /health
- POST /v1/accounts
- GET  /v1/accounts
- GET  /v1/accounts/:id

- POST /v1/journals
- GET  /v1/journals
- GET  /v1/journals/:id
- POST /v1/journals/:id/post

## Notes
- Untuk produksi: tambahkan auth + filter company_id dari token (bukan dari body).
- Untuk uang: idealnya pakai rust_decimal end-to-end.
