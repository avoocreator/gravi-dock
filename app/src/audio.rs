//! Suara: efek sintesis (dari memori) dan adzan (berkas pengguna lewat MCI, atau nada placeholder).
use crate::win;
use island_core::sfx::{self, Sfx};
use std::collections::HashMap;
use std::sync::Mutex;

static CACHE: Mutex<Option<HashMap<(u8, u8), &'static [u8]>>> = Mutex::new(None);

fn id(s: Sfx) -> u8 { s as u8 }

/// Putar efek. WAV dirender sekali per (efek, volume 0..100 kelipatan 5) lalu dibiarkan hidup (beberapa KB).
pub fn play(s: Sfx, vol_pct: i64) {
    if vol_pct <= 0 { return; }
    let q = ((vol_pct.clamp(0, 100) + 2) / 5 * 5) as u8;
    let data: &'static [u8] = {
        let mut g = CACHE.lock().unwrap();
        let map = g.get_or_insert_with(HashMap::new);
        *map.entry((id(s), q)).or_insert_with(|| Box::leak(sfx::render(s, q as f32 / 100.0).into_boxed_slice()))
    };
    win::api::play_wav_mem(data);
}

/// Mulai adzan. `file` kosong / tidak ada / gagal diputar -> nada placeholder bawaan.
pub fn adzan_start(file: &str, vol_pct: i64) {
    if !file.trim().is_empty() && std::path::Path::new(file.trim()).exists() && win::api::mci_play(file.trim(), vol_pct) { return; }
    play(Sfx::Adzan, vol_pct);
}
pub fn adzan_stop() { win::api::mci_stop(); }
pub fn adzan_playing() -> bool { win::api::mci_playing() }
