# Nobi Dynamic Island (Tauri 2 — Windows & Linux)

Aplikasi *Dynamic Island* lintas platform (Windows 10/11 & Linux) dibangun dengan **Tauri 2 (Rust + React + TypeScript + Tailwind CSS)** dan karakter pendamping robot-kucing prosedural 60 fps di Canvas 2D.

## Fitur Utama

1. **Karakter Robot-Kucing Interaktif (Canvas 2D, 60 fps):** Helm putih-lavender, visor gelap mengilap, telinga kucing bergradien biru–hijau & oranye–kuning, *eye-tracking* kursor, kedip otomatis, *squish* saat diklik, *dizzy* saat diklik 3x cepat, menari saat musik main, dan berubah menjadi kotak penerima saat file diseret ke *island*.
2. **Monitor AI Desktop & AI CLI + Kartu Approve/Deny:**
   - **AI Desktop:** Deteksi proses otomatis untuk **Claude Desktop**, **Gemini Desktop**, **ChatGPT Desktop**, dan **Antigravity Desktop** lengkap dengan tombol *Focus Window*.
   - **AI CLI:** Deteksi proses & integrasi *hook* lokal (`nobi-hook`) untuk **Antigravity CLI** dan **GitHub Copilot CLI** lengkap dengan kartu persetujuan **Allow / Always / Deny**.
3. **GitHub Pill:** Grafik kontribusi (*Contribution Graph*) dengan filter **3 Bulan (default)**, **6 Bulan**, dan **1 Tahun**, daftar **Repo Terbaru**, serta daftar **Commit Terbaru** (`short SHA` + pesan commit).
4. **Google Calendar Pill:** Mini kalender bulanan interaktif dengan penanda jadwal; klik tanggal mana saja untuk melihat daftar agenda di tanggal tersebut. Mendukung **Secret iCal URL** (paling cepat) maupun **Google Calendar API v3 OAuth Token**.
5. **Mini Chat AI:** Mendukung Ollama (lokal), Anthropic, Gemini, dan OpenAI dengan konteks otomatis dari file yang di-*drag-and-drop* ke *island*.
6. **Kontrol Musik OS:** Terhubung ke Windows GSMTC dan Linux MPRIS (`playerctl`).
7. **Pomodoro / Timer Fokus:** Preset 25 menit fokus & 5 menit istirahat dengan cincin progres SVG dan indikator di mode *compact*.
8. **Drag & Drop File + OS Keyring:** Seluruh token/API key disimpan aman di *Windows Credential Manager* atau *Linux Secret Service* (GNOME Keyring / KWallet).

---

## Prasyarat Build

### Windows 10 / 11
- [Node.js 20+](https://nodejs.org/)
- [Rust & Cargo](https://rustup.rs/)
- Microsoft C++ Build Tools (Desktop development with C++) & WebView2 (sudah bawaan Windows 11)

### Linux (Ubuntu / Debian)
```bash
sudo apt update
sudo apt install -y build-essential pkg-config \
  libwebkit2gtk-4.1-dev libayatana-appindicator3-dev \
  librsvg2-dev libssl-dev libdbus-1-dev patchelf \
  playerctl wmctrl
```
*(Untuk Fedora/Arch, pasang paket setara `webkit2gtk4.1-devel`, `libappindicator-gtk3`, `playerctl`, `wmctrl`.)*

---

## Cara Menjalankan & Build

```bash
# 1. Masuk ke folder proyek & install dependensi frontend
cd nobi-island
npm install

# 2. Jalankan mode development (otomatis membuka overlay Dynamic Island)
npm run tauri dev

# 3. Build installer produksi (.msi/.exe di Windows, .AppImage/.deb/.rpm di Linux)
npm run pack
```

Hasil *installer* akan tersimpan di `src-tauri/target/release/bundle/`.

---

## Build & Pasang `nobi-hook` (Untuk AI CLI)

```bash
cd hook-cli
cargo build --release
```

Binary akan tersedia di `hook-cli/target/release/nobi-hook` (atau `nobi-hook.exe` di Windows). Binary ini bersifat **fail-open** (langsung mengizinkan dalam 250 ms jika Nobi Island sedang tidak dijalankan agar CLI tidak pernah macet).

### Contoh Hook Antigravity CLI (`~/.gemini/config/hooks.json`)
```json
{
  "PreToolUse": [
    {
      "matcher": "shell",
      "hooks": [
        {
          "type": "command",
          "command": "/path/to/nobi-hook --agent antigravity-cli --event permission"
        }
      ]
    }
  ]
}
```

### Contoh Pengujian Manual Hook dari Terminal
```bash
echo '{"tool_name":"shell","tool_input":{"command":"cargo test"}}' | ./hook-cli/target/release/nobi-hook --agent copilot-cli --event permission
```

---

## Setup Token (Tab `Set` di dalam Island)

1. Matikan **Mode Demo** di tab **Set** untuk beralih ke data *live*.
2. **GitHub:** Buat *Personal Access Token* (classic) dengan scope `read:user` dan `repo`, lalu tempel di kolom *GitHub PAT*.
3. **Google Calendar:** Buka Google Calendar di web → *Settings* kalender Anda → salin **Secret address in iCal format**, lalu tempel di kolom *Google Calendar Secret iCal URL* (atau gunakan *OAuth Access Token*).
4. **Mini Chat AI:** Jalankan Ollama lokal (`ollama serve`) atau isi API key Anthropic/Gemini/OpenAI.
