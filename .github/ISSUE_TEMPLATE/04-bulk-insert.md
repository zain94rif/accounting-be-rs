---
name: "4. Gunakan Bulk Insert untuk Baris Jurnal (Journal Lines)"
about: "Mengoptimalkan pengisian baris jurnal dalam 1 query transaksi agar database cepat"
title: "Performance: Gunakan Bulk Insert untuk Baris Jurnal (Journal Lines)"
labels: ["enhancement", "performance"]
assignees: []
---

## Konteks
Saat ini di fungsi `create_journal` (`src/routes/journals.rs`), kita menggunakan perulangan `for l in req.lines` untuk menginsert setiap baris jurnal satu per satu ke database. Jika satu jurnal memiliki 10 baris, server akan melakukan 10 kali bolak-balik query ke database. Ini sangat lambat. Kita harus mengirimkannya sekaligus dalam 1 query (Bulk Insert).

## Langkah-langkah Implementasi
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
