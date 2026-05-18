---
name: "5. Implementasikan Graceful Shutdown pada Server"
about: "Mengizinkan server memproses request yang berjalan dulu sebelum dimatikan"
title: "Enhancement: Implementasikan Graceful Shutdown pada Server"
labels: ["enhancement", "good first issue"]
assignees: []
---

## Konteks
Saat ini jika server kita hentikan (misal saat dideploy ulang atau ditekan Ctrl+C), server langsung mati seketika. Jika ada request dari user yang sedang berjalan atau database sedang melakukan write di tengah jalan, transaksi tersebut bisa rusak. Kita perlu memberi tahu Axum agar menunggu request yang sedang berjalan selesai sebelum benar-benar mati.

## Langkah-langkah Implementasi
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
