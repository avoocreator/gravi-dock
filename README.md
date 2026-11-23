# Antigravity Island (Rust) v0.5

Versi Rust murni: tanpa Electron/Chromium, digambar GPU lewat Slint.

## Build (harus di Windows)
1. Pasang Rust (https://rustup.rs) + Visual Studio Build Tools (workload C++).
2. `cargo build --release`
3. `install.cmd` (tambahkan `-Autostart` agar jalan saat Windows start: `install.cmd -Autostart`)
4. Win+R, ketik `island`. Ctrl+Alt+A membuka/menutup. Lepas: `install.cmd -Uninstall`.

`cargo test -p island-core` menjalankan 22 tes inti (lolos).

## Fitur
Delapan tab: Home (jam, sholat, agenda, cuaca, CPU/RAM), Sesi AI (Claude Code, Claude Desktop, Copilot CLI, Antigravity; ChatGPT/Claude Desktop-chat lewat CPU, "perkiraan"), Musik (SMTC: judul, artis, putar/jeda/next/prev), Fokus (pomodoro), Klip (riwayat 12 item), Cari (aplikasi Start Menu, kalkulator, URL), Shelf (rak berkas), Atur (terang/gelap, aksen, hover-buka, adzan, sumber sesi, kota sholat).
- Ikon tray: klik kiri buka/tutup, klik kanan menu (Buka/tutup, Keluar); dipasang ulang otomatis bila Explorer restart.
- Maskot Gravi punya 20 suasana hati (senang, bekerja, fokus, ngantuk, tidur, waspada, panas, musik, istirahat, sholat, pagi, malam, semangat, penasaran, sayang, berpikir, selesai, sedih, cemas, diam) yang dipilih dari konteks.
- Agenda iCal mendukung ulangan harian, mingguan, bulanan, tahunan.
- Adzan: bunyi saat waktu fardu (suara sistem, atau .wav lewat `adzanWav`).
- Cuaca (open-meteo) dan kalender diambil lewat `curl.exe` bawaan Windows.

Catatan desain:
- Musik memakai satu proses PowerShell yang hanya hidup saat tab Musik dibuka (berhenti 30 dtk setelah ditutup) supaya memori tetap kecil.
- Shelf: seret-lepas berkas tidak didukung Slint; salin berkas di Explorer (Ctrl+C) lalu tekan "+ Tambah dari clipboard".
- Ikon tray memakai ikon aplikasi bawaan Windows (belum ikon sendiri).

## Status jujur
- `core/` diuji (22 tes). `app/` (Slint + FFI Win32 + tray + media) belum pernah dikompilasi; kirim galat build bila ada.
- Belum ada: gelombang audio asli (visualizer), sampul album, pengaturan hotkey lewat UI.

## settings.json (`%LOCALAPPDATA%\AntigravityIsland\`)
`lat, lon, city, hover, topPct, light, accent, ical, adzan, adzanWav, focusMin, breakMin, srcClaude, srcCopilot, srcAgy, srcDesk`

Slint: GPLv3 atau lisensi desktop gratis dengan atribusi.

## Catatan v0.5
Tampilan didesain ulang: kartu gelap membulat dengan border tipis, jam besar, cincin CPU/RAM, toggle di Pengaturan, tab bergaris bawah, tombol bulat untuk musik dan fokus.
