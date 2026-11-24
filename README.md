# Gravi Island

Dynamic island untuk Windows, ditulis ulang dengan **Rust + Slint**. Target RAM: di bawah 50 MB (renderer software).

## Fitur
- **Sesi AI**: Antigravity CLI, Copilot CLI, Claude Code (CLI), plus status Antigravity/Claude/ChatGPT Desktop. Izin CLI bisa **Setujui / Tolak** langsung dari island lewat `gravi-hook.exe`.
- **Now playing** (Windows SMTC), **Jam** (pomodoro yang bisa diatur, timer, stopwatch, alarm, adzan + suara), **Google Calendar** (atau URL iCal rahasia), **GitHub** (commit hari ini, repo terbaru, contribution graph), **Monitor sistem** (CPU, RAM, suhu, uptime), **Clipboard** dan **Shelf** (drag-drop file).
- **Preferensi**: gelap/terang/otomatis, skema warna, efek suara, posisi dock **kiri / kanan / atas**.
- **Maskot Gravi** dengan 12 ekspresi (lihat `assets/expressions.png`), mata mengikuti kursor, klik = lompat.

## Build (Windows)
```
rustup default stable
cargo build --release
```
Hasil: `target\release\gravi-island.exe` dan `gravi-hook.exe` (taruh berdampingan). Installer: `iscc installer\gravi-island.iss`. CI: `.github/workflows/build.yml` (tag `v*` membuat rilis).

## Status jujur
Logika inti (`core`, `hook`, `engine`) sudah dites di Linux. **File `.slint` dan `app/src/main.rs` belum pernah dikompilasi**, begitu juga kode khusus Windows (`win.rs`). Kemungkinan ada beberapa error build; kirim pesan error-nya dan akan diperbaiki.

Jika bermasalah:
- Drag-drop gagal compile: `cargo build --release --no-default-features`.
- Latar island tampak solid/hitam kotak (transparansi renderer software): di `settings.json` ubah `"renderer":"gpu"` (atau pilih "Render GPU" di Pengaturan).

## Tampilan
UI mengikuti 9 desain referensi: kartu kaca bergradien, ornamen, nav dock 9 ikon, kartu persetujuan. Warna dasar mengikuti mode gelap/terang; gradien dan ornamen mengikuti skema warna. Tidak ada tombol "Hentikan Semua" (mematikan proses AI dari luar dianggap berbahaya) dan tidak ada panah ganti bulan di kalender (belum ada logikanya).

## Placeholder
- **Adzan**: nada sintetis bawaan. Pilih file MP3/WAV sendiri di Pengaturan (`adzan_file`).
- **Google Calendar**: buat OAuth Client ID tipe *Desktop app* di Google Cloud Console, aktifkan Calendar API, lalu isi `gcal_client_id` (dan secret bila ada) di Pengaturan.
- **GitHub**: isi username; token (opsional) membuka grafik lengkap dan repo privat.

## Hook izin CLI
Aktifkan di Pengaturan. Claude Code: ditambahkan hook `PermissionRequest` ke `~/.claude/settings.json` (dibackup `.gravi-bak`). Copilot CLI: `~/.copilot/hooks/gravi-island.json`. Jika island mati, hook diam dan CLI memakai alur izin normalnya. Komunikasi hanya lewat loopback `127.0.0.1:47831` dengan token di `hook.json`.
Token dan pengaturan disimpan **plaintext** di `%LOCALAPPDATA%\GraviIsland`.

## Lisensi
Slint dipakai di bawah GPLv3 atau lisensi Slint Royalty-Free Desktop (perlu atribusi). Pilih salah satu sebelum rilis publik.
