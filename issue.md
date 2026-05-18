# Backend Improvements

This document lists actionable improvements for the accounting backend. Each task is written so that a junior programmer or an AI assistant can pick it up and implement it easily.

## 1. Replace `f64` with `Decimal` for Currency/Money Types
**Context:** Currently, the system uses `f64` (floating-point numbers) for `debit` and `credit` in `src/models/journal.rs` and `src/routes/journals.rs`. In JavaScript, this is like using standard `Number` for money, which can lead to precision issues (e.g., `0.1 + 0.2 = 0.30000000000000004`). In accounting software, precision is critical.
**Steps to implement:**
1. Add `rust_decimal` dependency to `Cargo.toml`: `cargo add rust_decimal --features serde`
2. Update `src/models/journal.rs`: Replace `f64` with `rust_decimal::Decimal` in `CreateJournalLineReq`.
3. Update `src/routes/journals.rs`: 
   - Remove the `f64` types and use `Decimal`.
   - Update the balance check logic: Instead of checking `(debit - credit).abs() > 0.0001`, you can exactly check if `debit == credit` because `Decimal` provides exact precision.
4. (If applicable) Make sure the PostgreSQL database schema uses `NUMERIC` or `DECIMAL` instead of `FLOAT` or `REAL`, and update any SQL queries in the Rust code to match.

## 2. Handle Specific Database Errors (e.g., Unique Constraint)
**Context:** When a user creates an account with a `code` that already exists, PostgreSQL throws a unique constraint violation. Currently, the `AppError::Db(sqlx::Error)` catches this and returns a `500 Internal Server Error`. We should return a `400 Bad Request` instead to tell the user they made a mistake, not the server.
**Steps to implement:**
1. In `src/error.rs`, modify the `IntoResponse` implementation for `AppError::Db(err)`.
2. Check if the error is a database constraint error. You can do this by matching on `sqlx::Error::Database(db_err)`.
3. If `db_err.is_unique_violation()`, return a `400 Bad Request` with a friendly message (e.g., "This record already exists").
4. Otherwise, fallback to the `500 Internal Server Error`.

## 3. Add Pagination to List Endpoints
**Context:** Endpoints like `GET /v1/accounts` and `GET /v1/journals` currently return all rows from the database. As data grows, this will cause memory and performance issues (similar to fetching an array of 1 million objects in Node.js).
**Steps to implement:**
1. Create a pagination query struct in `src/models/mod.rs` (e.g., `PaginationQuery { limit: Option<i64>, offset: Option<i64> }`).
2. Add `#[derive(Deserialize)]` to this struct.
3. In `src/routes/accounts.rs` and `src/routes/journals.rs`, inject this struct into the list handlers using Axum's `Query<PaginationQuery>` extractor.
4. Modify the SQL queries to include `LIMIT $1 OFFSET $2`, passing in the parsed limit (default 50) and offset (default 0).

## 4. Bulk Insert for Journal Lines
**Context:** In `src/routes/journals.rs` (`create_journal`), the code loops over `req.lines` and runs an `INSERT` query for each line individually. This creates multiple network trips to the database.
**Steps to implement:**
1. Refactor the line insertion in `create_journal`.
2. Use `sqlx::QueryBuilder` to construct a single bulk `INSERT` query for all lines at once.
3. Alternatively, you can use PostgreSQL's `UNNEST` function to insert arrays of data in a single query.
4. Execute the single bulk query using the transaction (`&mut *tx`).

## 5. Implement Graceful Shutdown
**Context:** When the server receives a kill signal (like Ctrl+C or from Docker/Kubernetes), it stops instantly. This means ongoing requests or database transactions might be cut off abruptly.
**Steps to implement:**
1. In `src/main.rs`, define a new async function `async fn shutdown_signal()` that listens for `tokio::signal::ctrl_c()`.
2. Add the `.with_graceful_shutdown(shutdown_signal())` method to the `axum::serve(listener, app)` call.
3. This will allow the server to finish processing active requests before completely stopping.
