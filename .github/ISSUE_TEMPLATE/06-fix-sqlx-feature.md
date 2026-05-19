---
name: "6. Perbaiki Error Fitur sqlx di Cargo.toml"
about: "Memperbaiki error kompilasi: sqlx does not have feature `decimal`"
title: "Bug: Perbaiki nama fitur sqlx untuk integrasi rust_decimal"
labels: ["bug", "good first issue"]
assignees: []
---

## Konteks
Saat menjalankan `cargo run`, terjadi error kompilasi karena kita mencoba mengaktifkan fitur `decimal` pada library `sqlx`. 
Error dari terminal:
```text
package `accounting-be` depends on `sqlx` with feature `decimal` but `sqlx` does not have that feature.
```
Ternyata, nama fitur yang benar di SQLx untuk mendukung library `rust_decimal` adalah **`rust_decimal`**, bukan `decimal`. Kita perlu memperbaiki penulisan fitur ini di dalam file konfigurasi agar program bisa dikompilasi ulang dengan sukses.

## Langkah-langkah Implementasi
1. Buka file `Cargo.toml` yang berada di direktori utama proyek.
2. Cari baris yang mengatur dependency `sqlx`, yang kurang lebih terlihat seperti ini:
   ```toml
   sqlx = { version = "0.7", features = ["runtime-tokio", "postgres", "uuid", "chrono", "macros", "migrate", "decimal"] }
   ```
3. Ubah kata `"decimal"` di dalam array `features` menjadi `"rust_decimal"`.
4. Baris tersebut setelah diperbaiki akan menjadi:
   ```toml
   sqlx = { version = "0.7", features = ["runtime-tokio", "postgres", "uuid", "chrono", "macros", "migrate", "rust_decimal"] }
   ```
5. Simpan file `Cargo.toml`.
6. Jalankan kembali perintah `cargo check` atau `cargo run` di terminal.
7. Pastikan tidak ada lagi error `failed to select a version for sqlx`.
