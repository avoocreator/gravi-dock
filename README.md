# Gravi

Dock interaktif bergaya Dynamic Island untuk Windows 10 dan Windows 11, dibangun dengan Tauri 2 (Rust, React, TypeScript, dan Tailwind CSS) beserta maskot robot-kucing 60 fps di Canvas 2D.

## Download dan Instalasi (Windows)

1. Buka halaman [Releases](https://github.com/avoocreator/gravi-dock/releases/latest) di repository ini.
2. Unduh file installer `Gravi_<versi>_x64-setup.exe` (contoh: `Gravi_0.1.0_x64-setup.exe`).
3. Jalankan installer tersebut. Gravi terpasang di folder pengguna (`currentUser`), jadi tidak meminta hak Administrator.
4. Setelah selesai, buka Gravi dari Start Menu atau shortcut Desktop.

> **Catatan Windows SmartScreen:** Karena rilis publik belum memakai code-signing certificate berbayar, Windows Defender SmartScreen dapat menampilkan peringatan *Windows protected your PC* saat pertama kali membuka installer. Klik **More info**, lalu **Run anyway** untuk melanjutkan.

### Persyaratan Sistem Windows
- Windows 10 atau Windows 11 (64-bit).
- Microsoft Edge WebView2 Runtime (sudah terpasang bawaan di Windows 11 dan sebagian besar Windows 10; jika belum ada, installer akan mengunduhnya otomatis).

---

## Fitur

- Maskot robot-kucing 60 fps di Canvas 2D yang mengikuti arah kursor, berkedip otomatis, bereaksi saat diklik, menari mengikuti musik, dan berubah menjadi kotak penerima saat file diseret ke island.
- Monitor AI Desktop (Claude Desktop, Gemini Desktop, ChatGPT Desktop, Antigravity Desktop) dengan tombol fokus jendela, serta kartu persetujuan `Allow / Always / Deny` untuk AI CLI (`antigravity-cli`, `copilot-cli`) lewat `gravi-hook.exe` yang sudah ikut terpasang.
- LocalDrop (HP ke Laptop dan sebaliknya) untuk kirim file lewat jaringan Wi-Fi lokal dengan scan QR code, tanpa internet. Saat pertama kali menyalakan server LocalDrop, izinkan akses jaringan lokal pada popup Windows Firewall agar perangkat di Wi-Fi yang sama bisa terhubung.
- Kontrol pemutar musik Windows (GSMTC) beserta pengatur volume per aplikasi langsung dari dock.
- Ringkasan notifikasi Windows dan System HUD (Caps Lock, baterai, Wi-Fi, statistik CPU dan RAM). Untuk membaca notifikasi Windows secara live, aktifkan izin **Notification access** di Windows Settings lewat tombol pintas di tab Settings Gravi.
- Panel GitHub (grafik kontribusi, repository, commit terbaru) dan Google Calendar bulanan via Secret iCal URL atau OAuth Token.
- Mini Chat AI (Ollama lokal, Anthropic, Gemini, atau OpenAI) dengan dukungan drag-and-drop file sebagai konteks percakapan.
- Timer Pomodoro untuk fokus dan istirahat dengan cincin progres pada tampilan compact.

### Pintasan Keyboard Bawaan
- `Alt + Space`: Buka atau tutup Gravi.
- `Alt + C`: Buka langsung panel Mini Chat AI.

---

## Pengaturan Awal di Aplikasi

Buka tab **Settings** (ikon gerigi) di Gravi untuk menyesuaikan:
- Matikan **Mode Demo** jika ingin memakai data live untuk GitHub, Calendar, dan Notifikasi.
- Simpan token (GitHub PAT, Google Calendar, API key AI) di kolom yang tersedia. Semua kredensial disimpan di Windows Credential Manager, bukan dalam file teks biasa.
- Lihat bagian **CLI Hook Status** untuk menyalin path `gravi-hook.exe` yang sudah dibundel ke konfigurasi hook CLI.

---

## Build dari Source (Untuk Developer)

### Prasyarat
- Node.js 20+
- Rust dan Cargo (`rustup`)
- Microsoft C++ Build Tools (Desktop development with C++) dan WebView2

### Perintah

```powershell
# 1. Masuk ke folder proyek dan install dependensi frontend
cd gravi-dock
npm install

# 2. Jalankan mode development
npm run tauri dev

# 3. Build installer NSIS Windows (.exe)
npm run pack
```

Perintah `npm run pack` otomatis membangun `hook-cli` (`gravi-hook.exe`), frontend (`dist/`), binary Tauri (`gravi.exe`), lalu mengemasnya ke `src-tauri/target/release/bundle/nsis/Gravi_<versi>_x64-setup.exe`.

### Rilis Otomatis via GitHub Actions

Workflow di `.github/workflows/release.yml` otomatis membangun dan mengunggah installer NSIS ke GitHub Releases setiap kali tag versi (`v*`) di-push:

```powershell
git tag v0.1.0
git push origin v0.1.0
```

Pastikan nomor versi di `package.json`, `src-tauri/Cargo.toml`, dan `src-tauri/tauri.conf.json` sudah sesuai sebelum membuat tag.

## Lisensi

Proyek ini menggunakan lisensi MIT. Lihat file [LICENSE](LICENSE).
