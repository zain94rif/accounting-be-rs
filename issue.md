# Perbaikan & Peningkatan Backend (Accounting)

Dokumen ini berisi daftar perbaikan yang perlu dilakukan pada backend akuntansi ini. Setiap tugas ditulis dengan langkah-langkah yang sangat detail dan jelas agar bisa dikerjakan oleh junior programmer atau model AI murah.

---

## 1. Ganti Tipe Data `f64` dengan `Decimal` untuk Uang (Currency/Money)
**Konteks:** 
Saat ini sistem menggunakan tipe data `f64` (floating-point) untuk nilai `debit` dan `credit` di `src/models/journal.rs` dan `src/routes/journals.rs`. 
Di JavaScript, ini sama seperti menggunakan tipe data `Number` biasa untuk uang, yang mana rawan terkena masalah presisi (contoh klasik: `0.1 + 0.2` menghasilkan `0.30000000000000004`). Dalam sistem akuntansi, selisih 1 perak pun sangat fatal. Kita harus menggunakan tipe data desimal presisi tinggi.

**Langkah-langkah Implementasi:**
1. Buka file `Cargo.toml`.
2. Tambahkan dependency `rust_decimal` dengan fitur `serde` dan `db` (untuk SQLx):
   ```toml
   rust_decimal = { version = "1.35", features = ["serde-float", "maths"] }
   ```
   *(Atau jalankan perintah `cargo add rust_decimal --features serde-float`)*
3. Buka file `src/models/journal.rs`.
4. Import Decimal di bagian atas file:
   ```rust
   use rust_decimal::Decimal;
   ```
5. Ganti tipe data `debit: f64` dan `credit: f64` pada struct `CreateJournalLineReq` menjadi `debit: Decimal` dan `credit: Decimal`.
6. Buka file `src/routes/journals.rs`.
7. Import Decimal di bagian atas file jika diperlukan.
8. Ubah inisialisasi variabel `debit` dan `credit` di fungsi `create_journal`. 
   Karena menggunakan `Decimal`, ganti tipenya:
   ```rust
   let debit: Decimal = req.lines.iter().map(|l| l.debit).sum();
   let credit: Decimal = req.lines.iter().map(|l| l.credit).sum();
   ```
9. Ubah validasi balance di fungsi `create_journal`. Karena desimal sangat presisi, kita tidak butuh toleransi `.abs() > 0.0001` lagi. Cukup bandingkan secara langsung:
   ```rust
   if debit != credit {
       return Err(AppError::BadRequest(format!(
           "Journal tidak balance: debit={} credit={}",
           debit, credit
       )));
   }
   ```
10. Jalankan `cargo check` atau `cargo build` untuk memastikan tidak ada error kompilasi.

---

## 2. Tangani Error Database Spesifik (Unique Constraint pada Code Akun)
**Konteks:**
Jika user mencoba membuat akun baru dengan `code` (kode akun) yang sudah terdaftar di database, PostgreSQL akan melempar error *unique constraint violation*. Saat ini, error tersebut ditangkap sebagai error database umum (`sqlx::Error`) dan langsung dikembalikan sebagai `500 Internal Server Error`. 
Seharusnya, ini dikembalikan sebagai `400 Bad Request` karena kesalahan ada di sisi input user, bukan kerusakan di server kita.

**Langkah-langkah Implementasi:**
1. Buka file `src/error.rs`.
2. Di dalam implementasi `IntoResponse` untuk `AppError` (sekitar baris 21-31), kita perlu mendeteksi jika error `sqlx::Error` adalah pelanggaran constraint unik.
3. Ubah bagian `AppError::Db(_)` di `match &self`:
   ```rust
   AppError::Db(err) => {
       // Cek apakah error dari postgres dan merupakan unique violation (kode error PG: 23505)
       if let Some(db_err) = err.as_database_error() {
           if db_err.code().as_deref() == Some("23505") {
               return (StatusCode::BAD_REQUEST, Json(ErrBody { 
                   error: "Kode akun (code) sudah digunakan. Gunakan kode lain.".to_string() 
               })).into_response();
           }
       }
       (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrBody { error: "Database error internal".to_string() })).into_response()
   }
   ```
4. Pastikan library `sqlx` di-import dengan benar untuk membaca error database.

---

## 3. Tambahkan Fitur Pagination pada Endpoint List (Daftar Data)
**Konteks:**
Endpoint `GET /v1/accounts` dan `GET /v1/journals` saat ini mengambil seluruh data dari database sekaligus (`SELECT * ...`). Jika data transaksi sudah mencapai ribuan atau jutaan, server akan lambat dan memory membengkak (sama seperti melakukan `find()` di Mongoose tanpa limit). Kita perlu membatasi jumlah data yang dikembalikan menggunakan parameter `limit` dan `offset` (pagination).

**Langkah-langkah Implementasi:**
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

---

## 4. Gunakan Bulk Insert untuk Baris Jurnal (Journal Lines)
**Konteks:**
Saat ini di fungsi `create_journal` (`src/routes/journals.rs`), kita menggunakan perulangan `for l in req.lines` untuk menginsert setiap baris jurnal satu per satu ke database. Jika satu jurnal memiliki 10 baris, server akan melakukan 10 kali bolak-balik query ke database. Ini sangat lambat. Kita harus mengirimkannya sekaligus dalam 1 query (Bulk Insert).

**Langkah-langkah Implementasi:**
1. Buka file `src/routes/journals.rs`.
2. Di bagian atas file, import `sqlx::QueryBuilder`.
3. Di dalam fungsi `create_journal`, setelah menginsert `journal_entries` dan mendapatkan `entry.id`, hapus perulangan `for` yang lama.
4. Buat dynamic query menggunakan `QueryBuilder`:
   ```rust
   let mut query_builder = sqlx::QueryBuilder::new(
       "INSERT INTO journal_lines (journal_entry_id, account_id, description, debit, credit) "
   );

   query_builder.push_values(req.lines, |mut b, line| {
       b.push_bind(entry.id)
        .push_bind(line.account_id)
        .push_bind(line.description)
        .push_bind(line.debit)
        .push_bind(line.credit);
   });

   let query = query_builder.build();
   query.execute(&mut *tx).await?;
   ```
5. Dengan begini, semua baris jurnal dimasukkan hanya dengan 1 kali eksekusi query SQL!

---

## 5. Implementasikan Graceful Shutdown pada Server
**Konteks:**
Saat ini jika server kita hentikan (misal saat dideploy ulang atau ditekan Ctrl+C), server langsung mati seketika. Jika ada request dari user yang sedang berjalan atau database sedang melakukan write di tengah jalan, transaksi tersebut bisa rusak. Kita perlu memberi tahu Axum agar menunggu request yang sedang berjalan selesai sebelum benar-benar mati.

**Langkah-langkah Implementasi:**
1. Buka file `src/main.rs`.
2. Buat fungsi helper asynchronous untuk mendeteksi sinyal shutdown di bagian bawah file:
   ```rust
   async fn shutdown_signal() {
       let ctrl_c = async {
           tokio::signal::ctrl_c()
               .await
               .expect("gagal mendengarkan event Ctrl+C");
       };

       #[cfg(unix)]
       let terminate = async {
           tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
               .expect("gagal mendaftarkan sinyal handler")
               .recv()
               .await;
       };

       #[cfg(not(unix))]
       let terminate = std::future::pending::<()>();

       tokio::select! {
           _ = ctrl_c => {},
           _ = terminate => {},
       }
       
       tracing::info!("Sinyal shutdown diterima, menghentikan server secara anggun...");
   }
   ```
3. Cari bagian eksekusi server `axum::serve(listener, app)` di dalam fungsi `main`.
4. Tambahkan `.with_graceful_shutdown(shutdown_signal())` pada pemanggilan server tersebut:
   ```rust
   axum::serve(listener, app)
       .with_graceful_shutdown(shutdown_signal())
       .await?;
   ```
