# Gravi Dock 🏝️🤖

Dynamic Island desktop multifungsi bergaya macOS untuk Windows 10/11. Menggabungkan pelacak sesi AI interaktif, Now Playing media, jam & waktu sholat/adzan, kalender, GitHub heatmap, dan monitor sistem dalam satu kapsul fleksibel (Top notch, Right, atau Left edge).

Ditenagai oleh **Rust murni & Slint**: ultra-ringan (~20–30 MB RAM, 0% CPU idle), digambar langsung ke GPU pada 60/120 FPS tanpa Electron/Chromium.

---

## Fitur Utama

- **🤖 Maskot Gravi Bot**: Vektor mecha 3D interaktif dengan telinga pelangi (kiri biru-hijau, kanan merah-kuning), visor kaca reflektif, dan 12 ekspresi emosi dinamis (Idle, Happy, Surprised, Sleepy, Confused, Sad, Thinking, Searching, Working, Success, Error, Peek).
- **⚡ AI Session Tracker**: Memantau sesi Antigravity CLI, Copilot CLI, Antigravity Desktop, Claude Desktop, dan ChatGPT Desktop dengan tombol tindakan/izin 1-klik langsung dari Island.
- **🎵 Now Playing (GSMTC)**: Mendeteksi pemutaran lagu/video di Windows (Spotify, YouTube, VLC) lengkap dengan animated waveform visualizer dan kontrol playback.
- **⏰ Jam, Timer, Pomodoro & Sholat**: Jam flip/digital, Pomodoro kustom, stopwatch, alarm, serta kalkulasi waktu sholat Kemenag RI dengan lantunan suara adzan otomatis.
- **📅 Kalender**: Integrasi feed iCal Google Calendar untuk memantau agenda harian dan tombol cepat Google Meet.
- **🐙 GitHub Activity**: Jumlah commit hari ini, repositori terakhir, dan 52-week GitHub Contribution Graph heatmap.
- **📊 Monitor Sistem**: Pemantauan penggunaan CPU (sparkline graph), RAM, suhu perangkat, dan running time / uptime.
- **📋 Clipboard & File Shelf**: Riwayat clipboard teks dan rak berkas sementara.
- **🎛️ Fleksibilitas Docking**: Bisa ditempatkan di **Top** (notch atas), **Right** (dock kanan), atau **Left** (dock kiri).

---

## Build & Instalasi

### Persyaratan
- Windows 10 atau Windows 11
- Rust toolchain (`rustup`) + Visual Studio Build Tools (C++ workload)

### Kompilasi
```bash
# Uji tes unit
cargo test -p island-core

# Kompilasi rilis
cargo build --release
```

Hasil biner mandiri `.exe` berada di `target/release/island.exe`.

---

## Lisensi
MIT License © 2026 Kira (Avoo Creator)
