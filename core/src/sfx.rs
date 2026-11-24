//! Efek suara yang disintesis sendiri (tanpa berkas aset): WAV 16-bit mono 22.05 kHz dalam memori.
//! Dipakai juga sebagai adzan *placeholder* bila pengguna belum menaruh berkas adzan sendiri.

pub const RATE: u32 = 22_050;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sfx { Click, Toggle, Open, Close, Success, Error, Notify, Alarm, PomoDone, Adzan }

impl Sfx {
    /// (frekuensi Hz, durasi ms) berurutan; frekuensi 0 = hening.
    fn notes(self) -> &'static [(f32, u32)] {
        match self {
            Sfx::Click => &[(1800.0, 18)],
            Sfx::Toggle => &[(900.0, 30), (1300.0, 40)],
            Sfx::Open => &[(520.0, 50), (780.0, 70)],
            Sfx::Close => &[(780.0, 50), (520.0, 70)],
            Sfx::Success => &[(660.0, 90), (880.0, 90), (1320.0, 160)],
            Sfx::Error => &[(300.0, 120), (0.0, 40), (240.0, 200)],
            Sfx::Notify => &[(988.0, 90), (1319.0, 160)],
            Sfx::Alarm => &[(880.0, 140), (0.0, 60), (880.0, 140), (0.0, 60), (880.0, 140), (0.0, 400), (880.0, 140), (0.0, 60), (880.0, 140), (0.0, 60), (880.0, 140)],
            Sfx::PomoDone => &[(784.0, 150), (988.0, 150), (1175.0, 150), (1568.0, 350)],
            // Placeholder adzan: melodi lambat bernada sederhana. Ganti dengan rekaman adzan asli (lihat README).
            Sfx::Adzan => &[(392.0, 500), (440.0, 500), (494.0, 700), (440.0, 400), (392.0, 900), (0.0, 300), (392.0, 500), (440.0, 500), (494.0, 700), (587.0, 900), (494.0, 1100)],
        }
    }
}

/// Sintesis: gelombang sinus + sedikit harmonik, dengan envelope agar tidak berdetak.
pub fn samples(s: Sfx, volume: f32) -> Vec<i16> {
    let vol = volume.clamp(0.0, 1.0) * 0.55;
    let mut out = Vec::new();
    for &(f, ms) in s.notes() {
        let n = (RATE as u64 * ms as u64 / 1000) as usize;
        let fade = (n / 6).clamp(1, 1200);
        for i in 0..n {
            let t = i as f32 / RATE as f32;
            let env = if i < fade { i as f32 / fade as f32 } else if i + fade > n { (n - i) as f32 / fade as f32 } else { 1.0 };
            let w = if f <= 0.0 { 0.0 } else { (2.0 * std::f32::consts::PI * f * t).sin() * 0.8 + (2.0 * std::f32::consts::PI * f * 2.0 * t).sin() * 0.2 };
            out.push((w * env * vol * i16::MAX as f32) as i16);
        }
    }
    out
}

/// Bungkus sampel menjadi berkas WAV (header RIFF 44 byte).
pub fn wav(samples: &[i16]) -> Vec<u8> {
    let data_len = (samples.len() * 2) as u32;
    let mut b = Vec::with_capacity(44 + samples.len() * 2);
    b.extend_from_slice(b"RIFF"); b.extend_from_slice(&(36 + data_len).to_le_bytes()); b.extend_from_slice(b"WAVEfmt ");
    b.extend_from_slice(&16u32.to_le_bytes()); b.extend_from_slice(&1u16.to_le_bytes()); b.extend_from_slice(&1u16.to_le_bytes());
    b.extend_from_slice(&RATE.to_le_bytes()); b.extend_from_slice(&(RATE * 2).to_le_bytes()); b.extend_from_slice(&2u16.to_le_bytes()); b.extend_from_slice(&16u16.to_le_bytes());
    b.extend_from_slice(b"data"); b.extend_from_slice(&data_len.to_le_bytes());
    for s in samples { b.extend_from_slice(&s.to_le_bytes()); }
    b
}

pub fn render(s: Sfx, volume: f32) -> Vec<u8> { wav(&samples(s, volume)) }

#[cfg(test)]
mod tests {
    use super::*;
    const ALL: [Sfx; 10] = [Sfx::Click, Sfx::Toggle, Sfx::Open, Sfx::Close, Sfx::Success, Sfx::Error, Sfx::Notify, Sfx::Alarm, Sfx::PomoDone, Sfx::Adzan];

    #[test]
    fn wav_header_is_valid() {
        for s in ALL {
            let w = render(s, 0.8);
            assert_eq!(&w[0..4], b"RIFF"); assert_eq!(&w[8..16], b"WAVEfmt ");
            let riff = u32::from_le_bytes([w[4], w[5], w[6], w[7]]) as usize;
            assert_eq!(riff + 8, w.len(), "{s:?}");
            assert!(w.len() > 44 + 200, "{s:?} terlalu pendek");
        }
    }
    #[test]
    fn volume_scales_amplitude_and_zero_is_silent() {
        let peak = |v: f32| samples(Sfx::Notify, v).iter().map(|x| x.unsigned_abs()).max().unwrap();
        assert!(peak(1.0) > peak(0.3));
        assert_eq!(peak(0.0), 0);
    }
    #[test]
    fn no_clipping() { for s in ALL { assert!(samples(s, 1.0).iter().all(|x| x.unsigned_abs() < 32000)); } }
}
