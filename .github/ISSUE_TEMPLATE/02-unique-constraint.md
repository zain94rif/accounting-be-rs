---
name: "2. Tangani Unique Constraint Error pada Code Akun"
about: "Mengubah response 500 menjadi 400 Bad Request jika kode akun duplikat dimasukkan"
title: "Bug: Tangani Unique Constraint Error pada Code Akun"
labels: ["bug", "good first issue"]
assignees: []
---

## Konteks
Jika user mencoba membuat akun baru dengan `code` (kode akun) yang sudah terdaftar di database, PostgreSQL akan melempar error *unique constraint violation*. Saat ini, error tersebut ditangkap sebagai error database umum (`sqlx::Error`) dan langsung dikembalikan sebagai `500 Internal Server Error`. 
Seharusnya, ini dikembalikan sebagai `400 Bad Request` karena kesalahan ada di sisi input user, bukan kerusakan di server kita.

## Langkah-langkah Implementasi
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
