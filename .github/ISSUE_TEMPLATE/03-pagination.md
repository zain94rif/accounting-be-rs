---
name: "3. Tambahkan Fitur Pagination pada Endpoint List"
about: "Membatasi data yang diambil dengan parameter limit & offset agar server efisien"
title: "Enhancement: Tambahkan Fitur Pagination pada Endpoint List"
labels: ["enhancement", "performance"]
assignees: []
---

## Konteks
Endpoint `GET /v1/accounts` and `GET /v1/journals` saat ini mengambil seluruh data dari database sekaligus (`SELECT * ...`). Jika data transaksi sudah mencapai ribuan atau jutaan, server akan lambat dan memory membengkak (sama seperti melakukan `find()` di Mongoose tanpa limit). Kita perlu membatasi jumlah data yang dikembalikan menggunakan parameter `limit` dan `offset` (pagination).

## Langkah-langkah Implementasi
1. Buka file `src/models/mod.rs` atau buat file model baru khusus request jika perlu. Untuk kemudahan, kita bisa buat struct di `src/models/account.rs` atau langsung di route.
2. Definisikan struct parameter query baru:
   ```rust
   use serde::Deserialize;

   #[derive(Deserialize)]
   pub struct PaginationParams {
       pub limit: Option<i64>,
       pub offset: Option<i64>,
   }
   ```
3. Buka file `src/routes/accounts.rs`.
4. Import `Query` dari axum: `use axum::extract::Query;`.
5. Ubah parameter fungsi `list_accounts` untuk menerima parameter query tersebut:
   ```rust
   async fn list_accounts(
       State(st): State<AppState>,
       Query(params): Query<PaginationParams>,
   ) -> Result<Json<Vec<Account>>, AppError> {
   ```
6. Set nilai default untuk `limit` (misal 50) dan `offset` (misal 0):
   ```rust
   let limit = params.limit.unwrap_or(50);
   let offset = params.offset.unwrap_or(0);
   ```
7. Ubah query database SQL menggunakan `sqlx::query_as!` untuk menyertakan `LIMIT` dan `OFFSET`:
   ```rust
   let rows = sqlx::query_as!(
       Account,
       r#"
       SELECT id, company_id, code, name, account_type, normal_balance, is_active, parent_id
       FROM accounts
       ORDER BY code
       LIMIT $1 OFFSET $2
       "#,
       limit,
       offset
   )
   .fetch_all(&st.db)
   .await?;
   ```
8. Lakukan hal yang sama untuk fungsi `list_journals` di `src/routes/journals.rs`.
