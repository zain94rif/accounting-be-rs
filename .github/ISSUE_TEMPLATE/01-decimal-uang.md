---
name: "1. Ganti Tipe f64 ke Decimal (Uang)"
about: "Panduan implementasi desimal presisi tinggi untuk kolom finansial (debit/credit)"
title: "Refactor: Ganti f64 ke Decimal untuk Akurasi Uang"
labels: ["enhancement", "good first issue", "help wanted"]
assignees: []
---

## Konteks
Saat ini sistem menggunakan tipe data `f64` (floating-point) untuk nilai `debit` dan `credit` di `src/models/journal.rs` dan `src/routes/journals.rs`. 
Di JavaScript, ini sama seperti menggunakan tipe data `Number` biasa untuk uang, yang mana rawan terkena masalah presisi (contoh klasik: `0.1 + 0.2` menghasilkan `0.30000000000000004`). Dalam sistem akuntansi, selisih 1 perak pun sangat fatal. Kita harus menggunakan tipe data desimal presisi tinggi.

## Langkah-langkah Implementasi
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
