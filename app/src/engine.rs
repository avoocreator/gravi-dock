//! Engine: seluruh state & logika aplikasi tanpa menyentuh Slint. `main.rs` hanya menyalin `View` ke UI dan meneruskan `Cmd`.
//! Waktu disuntikkan lewat `Wall`, sehingga seluruh perilaku bisa diuji dengan jam palsu.
use crate::{audio, hooks, media, online, win};
use island_core::clock::{self, Alarms, Countdown, Stopwatch, TimerState};
use island_core::focus::Focus;
use island_core::hookproto::Decision;
use island_core::json::{self, Value};
use island_core::settings::Settings;
use island_core::sfx::Sfx;
use island_core::sysstat::{self, CpuSampler, History};
use island_core::timeutil::{self as tu, DAYS_ID, MONTHS_ID};
use island_core::{ics, launcher, mood, prayer, sessions as ses, theme};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/* ===================================================== tipe tampilan (polos) ===================================================== */

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Rgb(pub u8, pub u8, pub u8);

#[derive(Clone, Debug, PartialEq, Default)] pub struct SessionRow { pub id: String, pub title: String, pub sub: String, pub meta: String, pub mark: String, pub tint: Rgb, pub running: bool, pub action: bool }
#[derive(Clone, Debug, PartialEq, Default)] pub struct ApprovalRow { pub id: String, pub tool: String, pub summary: String, pub cwd: String, pub mark: String, pub tint: Rgb, pub src: String }
#[derive(Clone, Debug, PartialEq, Default)] pub struct EventRow { pub title: String, pub time: String, pub sub: String, pub tint: Rgb, pub all_day: bool }
#[derive(Clone, Debug, PartialEq, Default)] pub struct AlarmRow { pub id: i32, pub time: String, pub label: String, pub repeat: String, pub on: bool }
#[derive(Clone, Debug, PartialEq, Default)] pub struct LapRow { pub n: String, pub lap: String, pub total: String }
#[derive(Clone, Debug, PartialEq, Default)] pub struct CommitRow { pub repo: String, pub message: String, pub meta: String }
#[derive(Clone, Debug, PartialEq, Default)] pub struct RepoRow { pub name: String, pub meta: String, pub url: String }
#[derive(Clone, Debug, PartialEq, Default)] pub struct ClipRow { pub text: String, pub pinned: bool }
#[derive(Clone, Debug, PartialEq, Default)] pub struct ShelfRow { pub name: String, pub path: String }
#[derive(Clone, Debug, PartialEq, Default)] pub struct DayCell { pub label: String, pub num: String, pub today: bool, pub dot: bool }
#[derive(Clone, Debug, PartialEq, Default)] pub struct PrayerRow { pub name: String, pub time: String, pub next: bool, pub past: bool }
#[derive(Clone, Debug, PartialEq, Default)] pub struct HitRow { pub title: String, pub sub: String, pub kind: String }

#[derive(Clone, Debug, PartialEq)]
pub struct FaceV { pub v: mood::FaceView }
impl Default for FaceV { fn default() -> Self { FaceV { v: mood::view(mood::Face::Idle) } } }

#[derive(Clone, Debug, PartialEq, Default)]
pub struct View {
    // umum
    pub clock: String, pub clock_sec: String, pub date_long: String, pub greeting: String, pub face: FaceV,
    pub badge: i32, pub ai_running: i32, pub toast: String, pub toast_kind: i32, pub mascot_on: bool,
    pub scale: f32, pub dark: bool, pub accent: Rgb, pub accent2: Rgb,
    // beranda
    pub weather: String, pub weather_temp: String, pub weather_desc: String, pub weather_city: String, pub next_prayer: String, pub next_prayer_time: String, pub next_prayer_left: String,
    pub prayers: Vec<PrayerRow>, pub agenda: Vec<EventRow>, pub hits: Vec<HitRow>, pub pill_line: String,
    // ai
    pub sessions: Vec<SessionRow>, pub approvals: Vec<ApprovalRow>,
    // musik
    pub m_active: bool, pub m_playing: bool, pub m_title: String, pub m_artist: String, pub m_source: String, pub m_pos: f32, pub m_pos_text: String, pub m_pos_l: String, pub m_pos_r: String,
    // jam
    pub pomo_text: String, pub pomo_phase: String, pub pomo_progress: f32, pub pomo_running: bool, pub pomo_rounds: String,
    pub timer_text: String, pub timer_progress: f32, pub timer_state: i32,
    pub sw_text: String, pub sw_running: bool, pub laps: Vec<LapRow>, pub alarms: Vec<AlarmRow>, pub alarm_next: String, pub adzan_note: String,
    // kalender
    pub cal_month: String, pub cal_week: Vec<DayCell>, pub cal_events: Vec<EventRow>, pub cal_status: String, pub cal_connected: bool,
    // github
    pub gh_user: String, pub gh_status: String, pub gh_total: String, pub gh_streak: String, pub gh_today: String, pub gh_cells: Vec<i32>,
    pub commits: Vec<CommitRow>, pub repos: Vec<RepoRow>,
    // sistem
    pub cpu: f32, pub mem: f32, pub disk: f32, pub battery: f32, pub has_battery: bool, pub charging: bool,
    pub cpu_text: String, pub mem_text: String, pub disk_text: String, pub temp_text: String, pub uptime_text: String, pub cpu_line: String, pub cpu_area: String,
    // klip & shelf
    pub clips: Vec<ClipRow>, pub shelf: Vec<ShelfRow>,
    // pengaturan (hanya-baca)
    pub hook_claude: bool, pub hook_copilot: bool, pub settings_note: String,
}

/* ===================================================== masukan ===================================================== */

#[derive(Clone, Copy, Debug)]
pub struct Wall { pub y: i64, pub mo: i64, pub d: i64, pub h: i64, pub mi: i64, pub s: i64, pub ms: i64, pub tz: f64 }
impl Wall {
    pub fn day(&self) -> i64 { prayer::days_from_civil(self.y, self.mo, self.d) }
    pub fn local_secs(&self) -> i64 { self.day() * 86400 + self.h * 3600 + self.mi * 60 + self.s }
    pub fn utc_secs(&self) -> i64 { self.ms / 1000 }
    pub fn now() -> Wall {
        let n = win::api::now();
        Wall { y: n.y, mo: n.mo, d: n.d, h: n.h, mi: n.mi, s: n.s, ms: ses::now_ms(), tz: win::api::tz_hours() }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct UiState { pub mode: i32, pub page: i32, pub clock_tab: i32 }

#[derive(Clone, Debug)]
pub enum Cmd {
    PomoToggle, PomoSkip, PomoReset,
    TimerToggle, TimerReset, TimerAdd(i64),
    SwToggle, SwLap, SwReset,
    AlarmAdd(i64, i64, i64, String), AlarmToggle(i64), AlarmDel(i64),
    AdzanTest, SfxTest,
    CalConnect, CalDisconnect, GhRefresh,
    OpenUrl(String),
    ClipCopy(usize), ClipPin(usize), ClipDel(usize), ClipClear,
    ShelfAdd, ShelfAddPaths(Vec<String>), ShelfOpen(usize), ShelfDel(usize), ShelfCopy(usize),
    Search(String), SearchRun(usize),
    Approve(String, i32),
    Media(String),
    HooksInstall(String), HooksRemove(String),
    PreviewFace(i32), MascotClicked, Interaction,
    Settings(Box<Settings>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Effect { OpenPanel { page: i32, clock_tab: i32 }, Relayout, Fast(bool) }

/* ===================================================== util ===================================================== */

pub const PALETTE: [Rgb; 6] = [Rgb(0x4d, 0xa3, 0xff), Rgb(0x34, 0xd3, 0x99), Rgb(0xff, 0x8a, 0x4c), Rgb(0xa7, 0x8b, 0xfa), Rgb(0xfb, 0x71, 0x85), Rgb(0xff, 0xc9, 0x4d)];

fn hex_rgb(h: &str) -> Rgb { theme::parse_hex(h).map(|(r, g, b)| Rgb(r, g, b)).unwrap_or(Rgb(0x4d, 0xa3, 0xff)) }

pub fn provider_style(p: &str) -> (&'static str, Rgb) {
    match p {
        "agy" => ("A›", Rgb(0x6f, 0x9b, 0xff)),
        "claude" => ("CC", Rgb(0xf0, 0x8a, 0x5d)),
        "claudedesk" => ("Cd", Rgb(0xe8, 0xa8, 0x7c)),
        "copilot" => ("Cp", Rgb(0x4f, 0xb3, 0xff)),
        "chatgpt" => ("GP", Rgb(0x2f, 0xd2, 0x9a)),
        "agydesk" => ("AD", Rgb(0x9b, 0x8c, 0xff)),
        _ => ("AI", Rgb(0x9b, 0x8c, 0xff)),
    }
}

fn short_path(p: &str) -> String {
    let parts: Vec<&str> = p.split(['/', '\\']).filter(|x| !x.is_empty()).collect();
    parts[parts.len().saturating_sub(2)..].join("/")
}

fn first_line(s: &str, n: usize) -> String { s.lines().find(|l| !l.trim().is_empty()).unwrap_or("").trim().chars().take(n).collect() }

fn scan_start_menu(dir: &Path, depth: u8, out: &mut Vec<(String, String)>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() { if depth > 0 { scan_start_menu(&p, depth - 1, out); } }
        else if p.extension().map(|x| x.eq_ignore_ascii_case("lnk")).unwrap_or(false) {
            if let Some(n) = p.file_stem().and_then(|s| s.to_str()) { out.push((n.to_string(), p.to_string_lossy().into_owned())); }
        }
    }
}

fn when_label(start_utc: i64, all_day: bool, w: &Wall) -> String {
    let ld = tu::local_day(start_utc, w.tz);
    let today = w.day();
    let pre = match ld - today { i64::MIN..=0 => "Hari ini".to_string(), 1 => "Besok".to_string(), _ => tu::day_label(ld) };
    if all_day { format!("{pre} · seharian") } else { format!("{pre} {}", tu::hhmm(start_utc, w.tz)) }
}

/* ===================================================== engine ===================================================== */

#[derive(Clone)]
struct ClipEntry { text: String, pinned: bool }

pub struct Engine {
    pub cfg: Arc<Mutex<Settings>>,
    data: PathBuf,
    focus: Focus, sw: Stopwatch, cd: Countdown, alarms: Alarms,
    cpu: CpuSampler, hist: History,
    clips: Vec<ClipEntry>, shelf: Vec<String>, last_clip_seq: u32,
    pub media: media::Media,
    online: Option<online::Online>,
    hook_srv: Option<hooks::Server>,
    sessions: Arc<Mutex<Vec<ses::Session>>>,
    apps: Arc<Mutex<Vec<(String, String)>>>,
    temp: Arc<Mutex<Option<f32>>>, temp_started_ms: i64,
    search_targets: Vec<String>,
    prev_running: HashSet<String>,
    cpu_prev: (i64, Vec<u64>), busy: [(u8, u8, bool); 2],
    media_idle: u32, ticks: u64, sys_dark: bool,
    refresh_only: bool, last_cpu: f32, last_rows: Vec<SessionRow>,
    toast_until: i64, done_until: i64, click_until: i64, ring_until: i64, prayer_until: i64, search_until: i64, last_interaction: i64,
    preview: Option<(mood::Face, i64)>,
    last_adzan_key: i64,
    cached_disk: (f32, u64, u64), cached_batt: (bool, f32, bool),
    pub view: View,
}

impl Engine {
    /// Tanpa thread/layanan: dipakai tes. `start_services` menyalakan sisanya.
    pub fn new_offline(cfg: Settings, data: &Path) -> Engine {
        let mut focus = Focus::new(cfg.focus_min, cfg.break_min);
        focus.configure(cfg.focus_min, cfg.break_min, cfg.long_break_min, cfg.rounds, cfg.auto_next);
        let alarms = std::fs::read_to_string(data.join("alarms.json")).map(|s| Alarms::parse(&s)).unwrap_or_default();
        let shelf = std::fs::read_to_string(data.join("shelf.txt")).unwrap_or_default().lines().filter(|l| !l.is_empty()).map(String::from).collect();
        let clips = std::fs::read_to_string(data.join("clips.json")).ok().and_then(|s| json::parse(&s)).map(|v| {
            v.arr().iter().map(|x| ClipEntry { text: x.get("t").text().to_string(), pinned: true }).filter(|c| !c.text.is_empty()).collect::<Vec<_>>()
        }).unwrap_or_default();
        Engine {
            cfg: Arc::new(Mutex::new(cfg)), data: data.to_path_buf(), focus, sw: Stopwatch::default(), cd: Countdown::default(), alarms,
            cpu: CpuSampler::default(), hist: History::new(60), clips, shelf, last_clip_seq: win::api::clip_seq(),
            media: media::Media::new(data), online: None, hook_srv: None,
            sessions: Arc::new(Mutex::new(vec![])), apps: Arc::new(Mutex::new(vec![])), temp: Arc::new(Mutex::new(None)), temp_started_ms: 0,
            search_targets: vec![], prev_running: HashSet::new(),
            cpu_prev: (0, vec![0; 2]), busy: [(0, 0, false); 2], media_idle: 0, ticks: 0, sys_dark: true, refresh_only: false, last_cpu: 0.0, last_rows: vec![],
            toast_until: 0, done_until: 0, click_until: 0, ring_until: 0, prayer_until: 0, search_until: 0, last_interaction: 0, preview: None,
            last_adzan_key: -1, cached_disk: (0.0, 0, 0), cached_batt: (false, 1.0, false), view: View::default(),
        }
    }

    pub fn start_services(&mut self) {
        let data = self.data.clone();
        self.online = Some(online::start(self.cfg.clone(), data.clone()));
        self.hook_srv = hooks::start(&data);
        // pemindai sesi AI
        {
            let (shared, cfg) = (self.sessions.clone(), self.cfg.clone());
            std::thread::spawn(move || {
                let home = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")).map(PathBuf::from).unwrap_or_default();
                let ad = std::env::var_os("APPDATA").map(PathBuf::from);
                let la = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
                let sc = ses::Scanner::new(&home, ad.as_deref(), la.as_deref(), None, None);
                loop {
                    let mut list = sc.scan(ses::now_ms(), &|p| cfg.lock().unwrap().enabled(p));
                    list.sort_by(|a, b| b.updated_ms.cmp(&a.updated_ms));
                    list.truncate(20);
                    *shared.lock().unwrap() = list;
                    std::thread::sleep(std::time::Duration::from_secs(3));
                }
            });
        }
        // daftar aplikasi Start Menu untuk pencarian
        {
            let apps = self.apps.clone();
            std::thread::spawn(move || {
                let mut v = vec![];
                for k in ["APPDATA", "PROGRAMDATA"] {
                    if let Some(b) = std::env::var_os(k) { scan_start_menu(&PathBuf::from(b).join("Microsoft/Windows/Start Menu/Programs"), 3, &mut v); }
                }
                v.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase())); v.dedup_by(|a, b| a.0 == b.0);
                *apps.lock().unwrap() = v;
            });
        }
    }

    pub fn hook_dirty(&self) -> bool { self.hook_srv.as_ref().map_or(false, |s| s.dirty.swap(false, std::sync::atomic::Ordering::SeqCst)) }
    pub fn approvals_pending(&self) -> usize { self.hook_srv.as_ref().map_or(0, |s| s.count()) }
    #[cfg(test)]
    pub fn inject_sessions(&self, v: Vec<ses::Session>) { *self.sessions.lock().unwrap() = v; }

    fn toast(&mut self, w: &Wall, t: &str) { let l = t.to_lowercase(); self.view.toast_kind = if l.contains("gagal") || l.contains("error") { 2 } else if l.contains("menyegarkan") || l.contains("memuat") { 1 } else { 0 }; self.view.toast = t.to_string(); self.toast_until = w.ms + 3500; }
    fn sfx(&self, s: Sfx) { let c = self.cfg.lock().unwrap(); if c.sfx { audio::play(s, c.sfx_vol); } }
    fn save_settings(&self) { let _ = std::fs::write(self.data.join("settings.json"), self.cfg.lock().unwrap().to_json()); }
    fn save_alarms(&self) { let _ = std::fs::write(self.data.join("alarms.json"), self.alarms.to_json()); }
    fn save_shelf(&self) { let _ = std::fs::write(self.data.join("shelf.txt"), self.shelf.join("\n")); }
    fn save_clips(&self) {
        let v = Value::Arr(self.clips.iter().filter(|c| c.pinned).map(|c| Value::obj(vec![("t", Value::s(&c.text))])).collect());
        let _ = std::fs::write(self.data.join("clips.json"), v.pretty());
    }

    /* ---------------------------------------------------- perintah ---------------------------------------------------- */

    pub fn cmd(&mut self, c: Cmd, w: &Wall) -> Vec<Effect> {
        let mut fx = vec![];
        let now_s = w.utc_secs();
        self.last_interaction = w.ms;
        match c {
            Cmd::Interaction => {}
            Cmd::PomoToggle => { if self.focus.phase == island_core::focus::Phase::Idle { self.focus.start(now_s) } else { self.focus.toggle_pause(now_s) } self.sfx(Sfx::Toggle); }
            Cmd::PomoSkip => { self.focus.skip(now_s); self.sfx(Sfx::Click); }
            Cmd::PomoReset => { self.focus.stop(); self.sfx(Sfx::Click); }
            Cmd::TimerToggle => { self.cd.toggle(w.ms); self.sfx(Sfx::Toggle); }
            Cmd::TimerReset => { self.cd.reset(); self.sfx(Sfx::Click); }
            Cmd::TimerAdd(s) => { self.cd.add(s); self.sfx(Sfx::Click); }
            Cmd::SwToggle => { self.sw.toggle(w.ms); fx.push(Effect::Fast(self.sw.running)); self.sfx(Sfx::Toggle); }
            Cmd::SwLap => { self.sw.lap(w.ms); self.sfx(Sfx::Click); }
            Cmd::SwReset => { self.sw.reset(); fx.push(Effect::Fast(false)); self.sfx(Sfx::Click); }
            Cmd::AlarmAdd(h, m, days, label) => { self.alarms.add(h as u8, m as u8, days as u8, label.trim()); self.save_alarms(); self.sfx(Sfx::Success); }
            Cmd::AlarmToggle(id) => { self.alarms.toggle(id as u32); self.save_alarms(); self.sfx(Sfx::Toggle); }
            Cmd::AlarmDel(id) => { self.alarms.remove(id as u32); self.save_alarms(); self.sfx(Sfx::Click); }
            Cmd::AdzanTest => {
                if audio::adzan_playing() { audio::adzan_stop(); } else { let c = self.cfg.lock().unwrap().clone(); audio::adzan_start(&c.adzan_file, c.adzan_vol); }
            }
            Cmd::SfxTest => { let c = self.cfg.lock().unwrap().clone(); audio::play(Sfx::Success, c.sfx_vol.max(20)); }
            Cmd::CalConnect => { if let Some(o) = &self.online { o.kick(online::Kick::GcalConnect); } }
            Cmd::CalDisconnect => { if let Some(o) = &self.online { o.kick(online::Kick::GcalDisconnect); } }
            Cmd::GhRefresh => { if let Some(o) = &self.online { o.kick(online::Kick::Github); } self.toast(w, "Menyegarkan GitHub…"); }
            Cmd::OpenUrl(u) => { if u.starts_with("https://") { win::api::shell_open(&u); } }
            Cmd::ClipCopy(i) => { if let Some(c) = self.clips.get(i) { win::api::clip_set(&c.text); self.last_clip_seq = win::api::clip_seq(); self.toast(w, "Disalin"); } }
            Cmd::ClipPin(i) => { if let Some(c) = self.clips.get_mut(i) { c.pinned = !c.pinned; } self.clips.sort_by_key(|c| !c.pinned); self.save_clips(); }
            Cmd::ClipDel(i) => { if i < self.clips.len() { self.clips.remove(i); self.save_clips(); } }
            Cmd::ClipClear => { self.clips.retain(|c| c.pinned); self.save_clips(); }
            Cmd::ShelfAdd => { let files = win::api::clip_files(); self.shelf_add(files, w); }
            Cmd::ShelfAddPaths(p) => self.shelf_add(p, w),
            Cmd::ShelfOpen(i) => { if let Some(p) = self.shelf.get(i) { win::api::shell_open(p); } }
            Cmd::ShelfCopy(i) => { if let Some(p) = self.shelf.get(i) { win::api::clip_set(p); self.last_clip_seq = win::api::clip_seq(); self.toast(w, "Path disalin"); } }
            Cmd::ShelfDel(i) => { if i < self.shelf.len() { self.shelf.remove(i); self.save_shelf(); } }
            Cmd::Search(q) => self.search(&q, w),
            Cmd::SearchRun(i) => {
                if let Some(t) = self.search_targets.get(i).cloned() {
                    if let Some(v) = t.strip_prefix("copy:") { win::api::clip_set(v); self.last_clip_seq = win::api::clip_seq(); self.toast(w, "Hasil disalin"); } else { win::api::shell_open(&t); }
                }
            }
            Cmd::Approve(id, kind) => {
                let d = match kind { 1 => Decision::Allow, 0 => Decision::Deny("Ditolak dari Gravi Island".into()), _ => Decision::Ask };
                if let Some(s) = &self.hook_srv { if s.answer(&id, d) { self.sfx(if kind == 1 { Sfx::Success } else { Sfx::Click }); if kind == 1 { self.done_until = w.ms + 2500; } } }
            }
            Cmd::Media(m) => { self.media.send(&m); }
            Cmd::HooksInstall(k) => { let n = match hooks::install(&k) { Ok(m) | Err(m) => m }; self.view.settings_note = n; self.refresh_hook_flags(); }
            Cmd::HooksRemove(k) => { let n = match hooks::remove(&k) { Ok(m) | Err(m) => m }; self.view.settings_note = n; self.refresh_hook_flags(); }
            Cmd::PreviewFace(i) => { self.preview = Some((mood::Face::from_index(i), w.ms + 4000)); }
            Cmd::MascotClicked => { self.click_until = w.ms + 1800; self.sfx(Sfx::Click); }
            Cmd::Settings(new) => fx.extend(self.apply_settings(*new)),
        }
        fx
    }

    fn refresh_hook_flags(&mut self) { self.view.hook_claude = hooks::installed("claude"); self.view.hook_copilot = hooks::installed("copilot"); }

    fn shelf_add(&mut self, files: Vec<String>, w: &Wall) {
        let before = self.shelf.len();
        for f in files { if !self.shelf.contains(&f) { self.shelf.insert(0, f); } }
        self.shelf.truncate(40);
        self.save_shelf();
        if self.shelf.len() > before { self.toast(w, "Ditambahkan ke Shelf"); }
    }

    fn search(&mut self, q: &str, w: &Wall) {
        let q = q.trim();
        let (mut hits, mut tg) = (vec![], vec![]);
        if let Some(v) = launcher::calc(q) {
            let s = if v.fract() == 0.0 && v.abs() < 1e15 { format!("{}", v as i64) } else { format!("{}", (v * 1e8).round() / 1e8) };
            hits.push(HitRow { title: format!("= {s}"), sub: "Salin hasil".into(), kind: "calc".into() }); tg.push(format!("copy:{s}"));
        }
        if let Some(url) = launcher::detect_url(q) { hits.push(HitRow { title: url.clone(), sub: "Buka di browser".into(), kind: "web".into() }); tg.push(url); }
        if !q.is_empty() {
            let list = self.apps.lock().unwrap();
            for a in launcher::rank(&list, |a| a.0.as_str(), q, 6) { hits.push(HitRow { title: a.0.clone(), sub: "Aplikasi".into(), kind: "app".into() }); tg.push(a.1.clone()); }
            if hits.is_empty() { hits.push(HitRow { title: format!("Cari di web: {q}"), sub: "Google".into(), kind: "web".into() }); tg.push(format!("https://www.google.com/search?q={}", island_core::gcal::url_encode(q))); }
        }
        self.search_until = if q.is_empty() { 0 } else { w.ms + 8000 };
        self.search_targets = tg;
        self.view.hits = hits;
    }

    fn apply_settings(&mut self, mut new: Settings) -> Vec<Effect> {
        let mut fx = vec![];
        if let Some((_, la, lo)) = prayer::CITIES.iter().find(|c| c.0.eq_ignore_ascii_case(new.city.trim())) { new.lat = *la; new.lon = *lo; }
        let old = self.cfg.lock().unwrap().clone();
        if old == new { return fx; }
        if (old.dock, old.mascot) != (new.dock, new.mascot) || (old.edge_pct - new.edge_pct).abs() > 1e-9 || (old.scale - new.scale).abs() > 1e-4 { fx.push(Effect::Relayout); }
        self.focus.configure(new.focus_min, new.break_min, new.long_break_min, new.rounds, new.auto_next);
        if self.focus.phase == island_core::focus::Phase::Idle { self.focus.focus_s = new.focus_min * 60; }
        if let Some(o) = &self.online {
            if (old.lat, old.lon) != (new.lat, new.lon) || old.ical != new.ical { o.kick(online::Kick::Weather); }
            if old.gh_user != new.gh_user || old.gh_token != new.gh_token { o.kick(online::Kick::Github); }
        }
        *self.cfg.lock().unwrap() = new;
        self.save_settings();
        fx
    }

    /* ---------------------------------------------------- tick 1 detik ---------------------------------------------------- */

    /// Segarkan `view` setelah sebuah perintah, tanpa mencuplik CPU/sesi lagi dan tanpa memajukan penghitung waktu-nyata.
    pub fn refresh(&mut self, ui: UiState, w: &Wall) -> Vec<Effect> {
        self.refresh_only = true;
        let fx = self.tick(ui, w);
        self.refresh_only = false;
        fx
    }

    /// Perbarui hanya daftar persetujuan (dipanggil segera saat hook baru masuk). True bila ada permintaan baru yang belum pernah terlihat.
    pub fn refresh_approvals(&mut self) -> bool {
        let reqs = self.hook_srv.as_ref().map(|s| s.snapshot()).unwrap_or_default();
        let before = self.view.approvals.len();
        self.view.approvals = reqs.iter().map(|r| {
            let (mark, tint) = provider_style(&r.src);
            ApprovalRow { id: r.id.clone(), tool: r.tool.clone(), summary: r.summary.clone(), cwd: short_path(&r.cwd), mark: mark.into(), tint, src: r.src.clone() }
        }).collect();
        self.view.badge = self.view.approvals.len() as i32;
        self.view.approvals.len() > before
    }
    pub fn notify_sound(&self) { self.sfx(Sfx::Notify); }

    pub fn tick(&mut self, ui: UiState, w: &Wall) -> Vec<Effect> {
        let mut fx = vec![];
        if !self.refresh_only { self.ticks += 1; }
        let cfg = self.cfg.lock().unwrap().clone();
        let (now_s, day, local) = (w.utc_secs(), w.day(), w.local_secs());
        if self.last_interaction == 0 { self.last_interaction = w.ms; }
        let panel = ui.mode == 2;

        // ---- waktu ----
        self.view.clock = format!("{:02}:{:02}", w.h, w.mi);
        self.view.clock_sec = format!("{:02}", w.s);
        self.view.date_long = format!("{}, {} {} {}", DAYS_ID[tu::weekday(day)], w.d, MONTHS_ID[(w.mo - 1).clamp(0, 11) as usize], w.y);
        self.view.greeting = match w.h { 4..=10 => "Selamat pagi", 11..=14 => "Selamat siang", 15..=17 => "Selamat sore", _ => "Selamat malam" }.into();
        self.view.mascot_on = cfg.mascot;
        self.view.scale = cfg.scale as f32;

        // ---- tema ----
        if self.ticks % 30 == 1 { self.sys_dark = win::api::system_dark(); }
        self.view.dark = cfg.theme.is_dark(self.sys_dark);
        let sc = theme::scheme(&cfg.scheme);
        self.view.accent = if cfg.accent.is_empty() { hex_rgb(sc.accent) } else { hex_rgb(&cfg.accent) };
        self.view.accent2 = hex_rgb(sc.accent2);

        // ---- sholat + adzan ----
        let times = prayer::times(w.y, w.mo, w.d, cfg.lat, cfg.lon, w.tz);
        let nx = prayer::next_prayer(local, cfg.lat, cfg.lon, w.tz);
        let cur_min = (w.h * 60 + w.mi) as i64;
        self.view.next_prayer = nx.prayer.name().into();
        self.view.next_prayer_time = format!("{:02}:{:02}", nx.at_local_secs.rem_euclid(86400) / 3600, nx.at_local_secs.rem_euclid(86400) % 3600 / 60);
        let l = nx.left_secs;
        self.view.next_prayer_left = if l >= 3600 { format!("{} j {} mnt lagi", l / 3600, l % 3600 / 60) } else { format!("{} mnt lagi", (l + 59) / 60) };
        if panel && (ui.page == 3 || ui.page == 0) {
            let today_next = nx.at_local_secs.div_euclid(86400) == day;
            self.view.prayers = [(1usize, "Subuh"), (2, "Terbit"), (3, "Dzuhur"), (4, "Ashar"), (5, "Maghrib"), (6, "Isya")].iter().map(|&(i, n)| PrayerRow {
                name: n.into(), time: format!("{:02}:{:02}", times[i] / 60, times[i] % 60),
                next: today_next && nx.prayer.name() == n, past: (times[i] as i64) < cur_min && !(today_next && nx.prayer.name() == n),
            }).collect();
            let f = cfg.adzan_file.trim();
            self.view.adzan_note = if !f.is_empty() && Path::new(f).exists() { format!("Berkas adzan: {}", f.rsplit(['\\', '/']).next().unwrap_or(f)) }
                else { "Memakai nada placeholder. Isi path berkas adzan (.wav/.mp3) di Pengaturan > Suara.".into() };
        }
        if cfg.adzan && w.s < 30 {
            for &(i, name) in &[(1usize, "Subuh"), (3, "Dzuhur"), (4, "Ashar"), (5, "Maghrib"), (6, "Isya")] {
                let key = day * 1440 + times[i] as i64;
                if times[i] as i64 == cur_min && self.last_adzan_key != key {
                    self.last_adzan_key = key;
                    audio::adzan_start(&cfg.adzan_file, cfg.adzan_vol);
                    self.prayer_until = w.ms + 60_000;
                    self.toast(w, &format!("Waktu {name} telah tiba"));
                }
            }
        }

        // ---- cuaca / online ----
        let on = self.online.as_ref().map(|o| o.shared.lock().unwrap().clone()).unwrap_or_default();
        self.view.weather = on.weather.clone(); { let (t, d) = on.weather.split_once(' ').unwrap_or((on.weather.as_str(), "")); self.view.weather_temp = t.to_string(); self.view.weather_desc = d.to_string(); }
        self.view.weather_city = cfg.city.clone();

        // ---- kalender ----
        let events: Vec<ics::Event> = if on.gcal_connected { on.gcal_events.clone() } else { on.ics_events.clone() };
        self.view.cal_connected = on.gcal_connected;
        self.view.cal_status = if !on.gcal_status.is_empty() { on.gcal_status.clone() } else if cfg.ical.starts_with("http") { "Memakai iCal".into() } else { "Belum terhubung".into() };
        let upcoming: Vec<&ics::Event> = events.iter().filter(|e| e.end > now_s).collect();
        let row = |i: usize, e: &ics::Event, long: bool| EventRow {
            title: e.title.clone(),
            time: if long { when_label(e.start, e.all_day, w) } else if e.all_day { "Seharian".into() } else { tu::hhmm(e.start, w.tz) },
            sub: if long { String::new() } else { when_label(e.start, true, w).replace(" · seharian", "") },
            tint: PALETTE[i % PALETTE.len()], all_day: e.all_day,
        };
        self.view.agenda = upcoming.iter().take(3).enumerate().map(|(i, e)| row(i, e, true)).collect();
        if panel && ui.page == 4 {
            let (_, mo, _) = prayer::civil_from_days(day);
            self.view.cal_month = format!("{} {}", MONTHS_ID[(mo - 1) as usize], w.y);
            let monday = day - tu::weekday(day) as i64;
            self.view.cal_week = (0..7).map(|k| {
                let d = monday + k;
                let (_, m2, dd) = prayer::civil_from_days(d);
                let _ = m2;
                DayCell { label: tu::DAYS_SHORT[k as usize].into(), num: dd.to_string(), today: d == day, dot: events.iter().any(|e| tu::local_day(e.start, w.tz) == d && e.end > now_s - 86400) }
            }).collect();
            self.view.cal_events = upcoming.iter().take(25).enumerate().map(|(i, e)| row(i, e, false)).collect();
        }

        // ---- github ----
        if panel && ui.page == 5 {
            self.view.gh_user = cfg.gh_user.trim().to_string();
            self.view.gh_status = on.gh_status.clone();
            if let Some(g) = &on.gh {
                let total = if g.heat.total > 0 { g.heat.total } else { g.heat.days.iter().map(|d| d.count).sum() };
                self.view.gh_total = format!("{total} kontribusi / tahun");
                self.view.gh_streak = if g.streak > 0 { format!("🔥 {} hari", g.streak) } else { String::new() };
                self.view.gh_today = format!("{} commit", g.today_commits);
                self.view.gh_cells = g.heat.cells(53);
                self.view.commits = g.commits.iter().take(20).map(|c| CommitRow { repo: c.repo.clone(), message: c.message.clone(), meta: tu::hhmm(c.at, w.tz) }).collect();
                self.view.repos = g.repos.iter().take(6).map(|r| RepoRow { name: r.name.clone(), meta: tu::ago(w.ms, r.at * 1000), url: r.url.clone() }).collect();
            } else { self.view.gh_cells.clear(); self.view.commits.clear(); self.view.repos.clear(); self.view.gh_total.clear(); self.view.gh_streak.clear(); self.view.gh_today = "0 commit".into(); }
        }

        // ---- sistem ----
        let (idle, tot) = win::api::cpu_times();
        #[cfg(test)]
        let cpu = 0.1;
        #[cfg(not(test))]
        let cpu = if self.refresh_only { self.last_cpu } else { let c = self.cpu.update(idle, tot); self.hist.push(c); self.last_cpu = c; c };
        #[cfg(test)]
        let (mem, used, total) = (0.2, 4.0, 20.0);
        #[cfg(not(test))]
        let (mem, used, total) = win::api::mem();
        self.view.cpu = cpu; self.view.mem = mem;
        if self.ticks % 10 == 1 { self.cached_disk = win::api::disk(); self.cached_batt = win::api::battery(); }
        self.view.disk = self.cached_disk.0;
        (self.view.has_battery, self.view.battery, self.view.charging) = self.cached_batt;
        if panel && (ui.page == 6 || ui.page == 0) {
            self.view.cpu_text = format!("{:.0}%", cpu * 100.0);
            self.view.mem_text = format!("{:.1} / {:.0} GB", used, total);
            self.view.disk_text = format!("{} bebas", sysstat::fmt_gb(self.cached_disk.1));
            self.view.uptime_text = sysstat::fmt_uptime(win::api::uptime_secs());
            self.view.cpu_line = self.hist.svg_line(120.0, 40.0);
            self.view.cpu_area = self.hist.svg_area(120.0, 40.0);
            if ui.page == 6 { self.poll_temperature(w); }
            self.view.temp_text = self.temp.lock().unwrap().map(|t| format!("{t:.0}°C")).unwrap_or_else(|| "—".into());
        }

        // ---- pomodoro ----
        if self.focus.tick(now_s) {
            self.sfx(Sfx::PomoDone);
            self.done_until = w.ms + 5000;
            let msg = format!("{} dimulai", self.focus.phase_name());
            self.toast(w, &msg);
        }
        self.view.pomo_text = self.focus.label(now_s);
        self.view.pomo_phase = self.focus.phase_name().into();
        self.view.pomo_progress = self.focus.progress(now_s);
        self.view.pomo_running = self.focus.running();
        self.view.pomo_rounds = format!("{} sesi fokus selesai", self.focus.done);

        // ---- timer ----
        if self.cd.tick(w.ms) {
            self.sfx(Sfx::Alarm);
            self.ring_until = w.ms + 8000;
            self.toast(w, "Timer selesai!");
            fx.push(Effect::OpenPanel { page: 3, clock_tab: 1 });
        }
        self.view.timer_text = clock::fmt_hms((self.cd.left_ms(w.ms) + 999) / 1000);
        self.view.timer_progress = if self.cd.state == TimerState::Finished { 0.0 } else { self.cd.progress(w.ms) };
        self.view.timer_state = match self.cd.state { TimerState::Idle => 0, TimerState::Running => 1, TimerState::Paused => 2, TimerState::Finished => 3 };

        // ---- stopwatch ----
        self.refresh_stopwatch(w.ms);

        // ---- alarm ----
        for a in self.alarms.due(local) {
            self.sfx(Sfx::Alarm);
            self.ring_until = w.ms + 10_000;
            self.toast(w, &format!("⏰ {}", if a.label.is_empty() { a.time_label() } else { a.label.clone() }));
            self.save_alarms();
            fx.push(Effect::OpenPanel { page: 3, clock_tab: 3 });
        }
        if panel && ui.page == 3 {
            self.view.alarms = self.alarms.list.iter().map(|a| AlarmRow { id: a.id as i32, time: a.time_label(), label: a.label.clone(), repeat: a.repeat_label(), on: a.enabled }).collect();
            self.view.alarm_next = match self.alarms.next(local) { Some((a, left)) => format!("Berikutnya: {} {} ({} lagi)", if a.label.is_empty() { "Alarm" } else { a.label.as_str() }, a.time_label(), tu::fmt_duration_short(left)), None => String::new() };
        }

        // ---- klip ----
        let seq = win::api::clip_seq();
        if seq != self.last_clip_seq {
            self.last_clip_seq = seq;
            if let Some(t) = win::api::clip_get() { self.push_clip(t); }
        }
        if panel && ui.page == 7 {
            self.view.clips = self.clips.iter().map(|c| ClipRow { text: first_line(&c.text, 140), pinned: c.pinned }).collect();
            self.view.shelf = self.shelf.iter().map(|p| ShelfRow { name: p.rsplit(['\\', '/']).next().unwrap_or(p).to_string(), path: p.clone() }).collect();
        }

        // ---- sesi AI ----
        let rows = if self.refresh_only { self.last_rows.clone() } else { let r = self.build_sessions(&cfg, w); self.last_rows = r.clone(); r };
        if !self.refresh_only {
            let running_now: HashSet<String> = rows.iter().filter(|r| r.running).map(|r| r.id.clone()).collect();
            let finished: Vec<String> = self.prev_running.difference(&running_now).cloned().collect();
            if !finished.is_empty() && !self.prev_running.is_empty() {
                self.done_until = w.ms + 5000;
                self.sfx(Sfx::Notify);
                if let Some(r) = rows.iter().find(|r| finished.contains(&r.id)) { let t = format!("Selesai: {}", r.title); self.toast(w, &t); }
            }
            self.prev_running = running_now;
        }
        self.view.ai_running = rows.iter().filter(|r| r.running).count() as i32;
        let log_action = rows.iter().any(|r| r.action);
        if panel && (ui.page == 1 || ui.page == 0) { self.view.sessions = rows; }

        // ---- persetujuan dari hook ----
        let reqs = self.hook_srv.as_ref().map(|s| s.snapshot()).unwrap_or_default();
        let _ = reqs;
        self.refresh_approvals();

        // ---- musik (pembantu PowerShell hanya hidup saat perlu) ----
        let want_media = panel && (ui.page == 2 || ui.page == 0);
        if want_media { self.media_idle = 0; if !self.media.running() { self.media.start(); } }
        else if self.media.running() && !self.refresh_only { self.media_idle += 1; if self.media_idle >= 30 { self.media.stop(); } }
        let m = self.media.snapshot();
        self.view.m_active = m.active; self.view.m_playing = m.playing;
        self.view.m_title = if m.active { m.title.clone() } else if want_media && self.media.running() { "Memuat…".into() } else { String::new() };
        self.view.m_artist = m.artist.clone(); self.view.m_source = m.source.clone();
        self.view.m_pos = if m.dur_s > 0.0 { (m.pos_s / m.dur_s).clamp(0.0, 1.0) as f32 } else { 0.0 };
        self.view.m_pos_text = if m.active { media::fmt_pos(m.pos_s, m.dur_s) } else { String::new() };
        { let (l, r) = self.view.m_pos_text.split_once(" / ").unwrap_or((self.view.m_pos_text.as_str(), "")); let (l, r) = (l.to_string(), r.to_string()); self.view.m_pos_l = if l.is_empty() { "0:00".into() } else { l }; self.view.m_pos_r = if r.is_empty() { "0:00".into() } else { r }; }
        let music = m.active && m.playing;

        // ---- hook flags (hanya saat pengaturan terbuka) ----
        if panel && ui.page == 8 && self.ticks % 10 == 0 { self.refresh_hook_flags(); }

        // ---- teks status pil ----
        self.view.pill_line = if self.focus.running() { format!("{} {}", if self.focus.phase == island_core::focus::Phase::Focus { "●" } else { "☕" }, self.focus.label(now_s)) }
            else if self.cd.state == TimerState::Running { format!("⏳ {}", clock::fmt_hms((self.cd.left_ms(w.ms) + 999) / 1000)) }
            else if self.view.ai_running > 0 { format!("AI ×{}", self.view.ai_running) }
            else if music { first_line(&m.title, 14) }
            else if self.sw.running { "⏱".into() }
            else { String::new() };

        // ---- ekspresi maskot ----
        let idle_long = w.ms - self.last_interaction > 90_000;
        let ctx = mood::Ctx {
            hour: w.h, ai_running: self.view.ai_running, ai_action: log_action || self.view.badge > 0, cpu, mem,
            battery_low: self.cached_batt.0 && self.cached_batt.1 < 0.15 && !self.cached_batt.2,
            focus: self.focus.phase == island_core::focus::Phase::Focus && self.focus.running(),
            on_break: matches!(self.focus.phase, island_core::focus::Phase::Break | island_core::focus::Phase::LongBreak) && self.focus.running(),
            music, searching: w.ms < self.search_until, offline: on.offline && self.ticks > 20,
            prayer_now: w.ms < self.prayer_until, prayer_soon: l > 0 && l < 600,
            just_done: w.ms < self.done_until, clicked: w.ms < self.click_until, alarm_ringing: w.ms < self.ring_until, idle_long,
        };
        self.view.face = FaceV { v: mood::pick(&ctx) };
        if let Some((f, until)) = self.preview { if w.ms < until { self.view.face = FaceV { v: mood::view(f) }; } else { self.preview = None; } }

        if w.ms >= self.toast_until && !self.view.toast.is_empty() { self.view.toast.clear(); }
        fx
    }

    pub fn refresh_stopwatch(&mut self, ms: i64) {
        self.view.sw_text = clock::fmt_stopwatch(self.sw.elapsed(ms));
        self.view.sw_running = self.sw.running;
        if self.view.laps.len() != self.sw.laps.len() {
            self.view.laps = self.sw.laps.iter().enumerate().rev().map(|(i, &t)| {
                let prev = if i == 0 { 0 } else { self.sw.laps[i - 1] };
                LapRow { n: format!("#{}", i + 1), lap: clock::fmt_stopwatch(t - prev), total: clock::fmt_stopwatch(t) }
            }).collect();
        }
    }
    pub fn stopwatch_text(&self, ms: i64) -> String { clock::fmt_stopwatch(self.sw.elapsed(ms)) }

    fn push_clip(&mut self, t: String) {
        let t = t.trim().to_string();
        if t.is_empty() { return; }
        let pinned = self.clips.iter().find(|c| c.text == t).map_or(false, |c| c.pinned);
        self.clips.retain(|c| c.text != t);
        self.clips.insert(0, ClipEntry { text: t, pinned });
        self.clips.sort_by_key(|c| !c.pinned);
        while self.clips.iter().filter(|c| !c.pinned).count() > 30 { if let Some(i) = self.clips.iter().rposition(|c| !c.pinned) { self.clips.remove(i); } }
        if pinned { self.save_clips(); }
    }

    fn poll_temperature(&mut self, w: &Wall) {
        if !cfg!(windows) || w.ms - self.temp_started_ms < 20_000 { return; }
        self.temp_started_ms = w.ms;
        let slot = self.temp.clone();
        std::thread::spawn(move || {
            let mut c = std::process::Command::new("powershell.exe");
            c.args(["-NoProfile", "-NonInteractive", "-Command", "(Get-CimInstance -Namespace root/wmi -ClassName MSAcpi_ThermalZoneTemperature -ErrorAction SilentlyContinue).CurrentTemperature"]);
            #[cfg(windows)]
            { use std::os::windows::process::CommandExt; c.creation_flags(0x0800_0000); }
            if let Ok(o) = c.output() { *slot.lock().unwrap() = sysstat::parse_thermal(&String::from_utf8_lossy(&o.stdout)); }
        });
    }

    fn build_sessions(&mut self, cfg: &Settings, w: &Wall) -> Vec<SessionRow> {
        let mut rows: Vec<SessionRow> = vec![];
        let sessions = self.sessions.lock().unwrap();
        let has_claude_desk_log = sessions.iter().any(|s| s.provider == "claudedesk" && s.state == ses::State::Running);
        // aplikasi desktop tanpa berkas sesi: perkiraan dari CPU proses ([chatgpt, claude]).
        // antigravity.exe dikecualikan karena proses IDE Electron-nya sering spike saat idle/edit; antigravity dilacak dari transcript.jsonl.
        if cfg.src_desk {
            let names = ["chatgpt.exe", "claude.exe"];
            let cur = win::api::proc_cpu(&names);
            let el = ((w.ms - self.cpu_prev.0).max(1) as f64) / 1000.0;
            if self.cpu_prev.0 != 0 && self.cpu_prev.1.len() == 2 {
                for i in 0..2 {
                    let frac = cur[i].saturating_sub(self.cpu_prev.1[i]) as f64 / 1e7 / el;
                    let b = &mut self.busy[i];
                    if frac > 0.30 { b.0 = b.0.saturating_add(1); b.1 = 0; } else { b.1 = b.1.saturating_add(1); b.0 = 0; }
                    if b.0 >= 4 { b.2 = true } else if b.1 >= 2 { b.2 = false }
                }
            }
            self.cpu_prev = (w.ms, cur);
            for (i, (prov, name)) in [("chatgpt", "ChatGPT Desktop"), ("claudedesk", "Claude (perkiraan)")].iter().enumerate() {
                if self.busy[i].2 && !(i == 1 && has_claude_desk_log) {
                    let (mark, tint) = provider_style(prov);
                    rows.push(SessionRow { id: format!("guess:{prov}"), title: if i == 1 { name.to_string() } else { format!("{name} (perkiraan)") }, sub: "aktif memproses".into(), meta: "berjalan".into(), mark: mark.into(), tint, running: true, action: false });
                }
            }
        }
        for s in sessions.iter() {
            let (mark, tint) = provider_style(s.provider);
            let running = s.state == ses::State::Running;
            let sub = if !s.current.is_empty() && running { s.current.clone() } else { s.cwd.rsplit(['\\', '/']).next().unwrap_or("").to_string() };
            rows.push(SessionRow { id: s.id.clone(), title: s.title.clone(), sub, meta: if running { "berjalan".into() } else { tu::ago(w.ms, s.updated_ms) }, mark: mark.into(), tint, running, action: s.state == ses::State::NeedsAction });
        }
        rows
    }
}

/* ===================================================== tes ===================================================== */

#[cfg(test)]
mod tests {
    use super::*;
    use island_core::hookproto::Request;

    fn tmpdir(n: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("gravi-engine-{}-{}", std::process::id(), n));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }
    fn settings() -> Settings { let mut s = Settings::default(); s.sfx = false; s.adzan = false; s.src_desk = false; s }
    /// Senin, 23 Nov 2026 pukul h:mi:s WIB.
    fn wall(h: i64, mi: i64, s: i64) -> Wall {
        let day = prayer::days_from_civil(2026, 11, 23);
        Wall { y: 2026, mo: 11, d: 23, h, mi, s, ms: ((day * 86400 + h * 3600 + mi * 60 + s) - 7 * 3600) * 1000, tz: 7.0 }
    }
    const PANEL_HOME: UiState = UiState { mode: 2, page: 0, clock_tab: 0 };

    #[test]
    fn clock_date_and_prayer_rows() {
        let mut e = Engine::new_offline(settings(), &tmpdir("a"));
        e.tick(UiState { mode: 2, page: 3, clock_tab: 4 }, &wall(19, 2, 5));
        assert_eq!((e.view.clock.as_str(), e.view.clock_sec.as_str()), ("19:02", "05"));
        assert_eq!(e.view.date_long, "Senin, 23 November 2026");
        assert_eq!(e.view.greeting, "Selamat malam");
        assert_eq!(e.view.prayers.len(), 6);
        assert_eq!(e.view.prayers[0].name, "Subuh");
        assert!(e.view.prayers[0].past && e.view.prayers[4].past && e.view.prayers[5].next, "Isya berikutnya: {:?}", e.view.prayers);
        assert_eq!(e.view.next_prayer, "Isya");
        assert!(e.view.adzan_note.contains("placeholder"));
    }

    #[test]
    fn pomodoro_and_timer_via_commands() {
        let mut e = Engine::new_offline(settings(), &tmpdir("b"));
        let w = wall(10, 0, 0);
        e.cmd(Cmd::PomoToggle, &w);
        e.tick(PANEL_HOME, &w);
        assert!(e.view.pomo_running && e.view.pomo_text == "25:00" && e.view.pill_line.contains("25:00"));
        let w2 = wall(10, 25, 1);
        e.tick(PANEL_HOME, &w2);
        assert_eq!(e.view.pomo_phase, "Istirahat");
        assert!(e.view.face.v.arc > 0.5, "wajah sukses setelah fokus selesai");
        assert!(e.view.toast.contains("Istirahat"));
        // timer
        e.cmd(Cmd::TimerAdd(-240), &w); // 5 mnt - 4 = 1 mnt
        e.cmd(Cmd::TimerToggle, &w);
        assert!(e.tick(PANEL_HOME, &wall(10, 0, 30)).is_empty());
        assert_eq!(e.view.timer_state, 1);
        let fx = e.tick(PANEL_HOME, &wall(10, 1, 1));
        assert!(fx.contains(&Effect::OpenPanel { page: 3, clock_tab: 1 }));
        assert_eq!(e.view.timer_state, 3);
    }

    #[test]
    fn alarm_fires_once_and_persists() {
        let d = tmpdir("c");
        let mut e = Engine::new_offline(settings(), &d);
        e.cmd(Cmd::AlarmAdd(19, 5, 0x7f, "Minum obat".into()), &wall(19, 0, 0));
        assert!(e.tick(PANEL_HOME, &wall(19, 4, 59)).is_empty());
        let fx = e.tick(UiState { mode: 2, page: 3, clock_tab: 3 }, &wall(19, 5, 1));
        assert!(fx.contains(&Effect::OpenPanel { page: 3, clock_tab: 3 }));
        assert!(e.view.toast.contains("Minum obat"));
        assert!(e.tick(PANEL_HOME, &wall(19, 5, 2)).is_empty(), "tidak berbunyi dua kali");
        let e2 = Engine::new_offline(settings(), &d);
        assert_eq!(e2.alarms.list.len(), 1);
        assert_eq!(e2.alarms.list[0].label, "Minum obat");
    }

    #[test]
    fn stopwatch_laps() {
        let mut e = Engine::new_offline(settings(), &tmpdir("d"));
        let w = wall(8, 0, 0);
        assert!(e.cmd(Cmd::SwToggle, &w).contains(&Effect::Fast(true)));
        let mut w2 = w; w2.ms += 3_500;
        e.cmd(Cmd::SwLap, &w2);
        w2.ms += 2_000;
        e.cmd(Cmd::SwLap, &w2);
        e.refresh_stopwatch(w2.ms);
        assert_eq!(e.view.sw_text, "00:05.50");
        assert_eq!(e.view.laps.len(), 2);
        assert_eq!((e.view.laps[0].n.as_str(), e.view.laps[0].lap.as_str()), ("#2", "00:02.00"));
        assert!(e.cmd(Cmd::SwReset, &w2).contains(&Effect::Fast(false)));
    }

    #[test]
    fn search_calc_and_fallback() {
        let mut e = Engine::new_offline(settings(), &tmpdir("e"));
        let w = wall(9, 0, 0);
        e.cmd(Cmd::Search("2+3*4".into()), &w);
        assert_eq!(e.view.hits[0].title, "= 14");
        e.cmd(Cmd::Search("xyzzy-tidak-ada".into()), &w);
        assert!(e.view.hits[0].title.starts_with("Cari di web"));
        e.tick(PANEL_HOME, &w);
        assert!(e.view.face.v.look_x < -0.5, "maskot 'Searching' saat mencari");
        e.cmd(Cmd::Search("".into()), &w);
        assert!(e.view.hits.is_empty());
    }

    #[test]
    fn settings_apply_city_dock_and_focus() {
        let d = tmpdir("f");
        let mut e = Engine::new_offline(settings(), &d);
        let mut s = settings();
        s.city = "jakarta".into(); s.dock = theme::Dock::Top; s.focus_min = 50; s.scheme = "everforest".into();
        let fx = e.cmd(Cmd::Settings(Box::new(s)), &wall(9, 0, 0));
        assert!(fx.contains(&Effect::Relayout));
        let c = e.cfg.lock().unwrap().clone();
        assert!((c.lat - -6.2088).abs() < 1e-3 && c.dock == theme::Dock::Top);
        drop(c);
        e.tick(PANEL_HOME, &wall(9, 0, 1));
        assert_eq!(e.view.accent, hex_rgb("#a7c080"));
        assert_eq!(e.view.pomo_text, "50:00");
        assert!(Settings::parse(&std::fs::read_to_string(d.join("settings.json")).unwrap()).dock == theme::Dock::Top);
        let same = e.cfg.lock().unwrap().clone();
        assert!(e.cmd(Cmd::Settings(Box::new(same)), &wall(9, 0, 2)).is_empty(), "tanpa perubahan -> tanpa efek");
    }

    #[test]
    fn approvals_and_sessions_in_view() {
        let d = tmpdir("g");
        let mut e = Engine::new_offline(settings(), &d);
        e.hook_srv = hooks::start(&d);
        let ep = island_core::hookproto::Endpoint::parse(&std::fs::read_to_string(d.join("hook.json")).unwrap()).unwrap();
        let mut c = std::net::TcpStream::connect(("127.0.0.1", ep.port)).unwrap();
        use std::io::Write;
        writeln!(c, "{}", Request { token: ep.token.clone(), src: "copilot".into(), id: "r1".into(), tool: "powershell".into(), summary: "git push".into(), cwd: "C:\\proj\\app".into(), ..Default::default() }.to_line()).unwrap();
        for _ in 0..100 { if e.approvals_pending() == 1 { break; } std::thread::sleep(std::time::Duration::from_millis(20)); }
        assert!(e.hook_dirty());
        e.tick(PANEL_HOME, &wall(12, 0, 0));
        assert_eq!((e.view.badge, e.view.approvals[0].mark.as_str(), e.view.approvals[0].cwd.as_str()), (1, "Cp", "proj/app"));
        assert!(e.view.face.v.ring > 0.5, "maskot terkejut saat butuh persetujuan");
        e.cmd(Cmd::Approve("r1".into(), 1), &wall(12, 0, 1));
        let mut line = String::new();
        std::io::BufRead::read_line(&mut std::io::BufReader::new(c), &mut line).unwrap();
        assert_eq!(Decision::parse_line(&line), Decision::Allow);
        e.tick(PANEL_HOME, &wall(12, 0, 2));
        assert_eq!(e.view.badge, 0);
    }

    #[test]
    fn running_session_finishing_triggers_success() {
        let mut e = Engine::new_offline(settings(), &tmpdir("h"));
        let mk = |state| ses::Session { id: "claude:abc".into(), provider: "claude", title: "Refactor".into(), state, updated_ms: 0, cwd: "C:\\p\\x".into(), current: "Menulis…".into() };
        e.inject_sessions(vec![mk(ses::State::Running)]);
        e.tick(UiState { mode: 2, page: 1, clock_tab: 0 }, &wall(15, 0, 0));
        assert_eq!(e.view.ai_running, 1);
        assert_eq!(e.view.sessions[0].sub, "Menulis…");
        assert!(e.view.pill_line.contains("AI"));
        e.inject_sessions(vec![mk(ses::State::Idle)]);
        e.tick(UiState { mode: 2, page: 1, clock_tab: 0 }, &wall(15, 0, 1));
        assert!(e.view.toast.contains("Refactor"));
        assert!(e.view.face.v.arc > 0.5);
    }

    #[test]
    fn clipboard_history_pins_and_shelf() {
        let d = tmpdir("i");
        let mut e = Engine::new_offline(settings(), &d);
        for t in ["satu", "dua", "tiga", "dua"] { e.push_clip(t.into()); }
        assert_eq!(e.clips.iter().map(|c| c.text.as_str()).collect::<Vec<_>>(), ["dua", "tiga", "satu"]);
        e.cmd(Cmd::ClipPin(2), &wall(9, 0, 0)); // "satu" naik ke atas
        assert!(e.clips[0].pinned && e.clips[0].text == "satu");
        e.cmd(Cmd::ClipClear, &wall(9, 0, 0));
        assert_eq!(e.clips.len(), 1);
        assert_eq!(Engine::new_offline(settings(), &d).clips.len(), 1, "klip tersemat bertahan");
        e.cmd(Cmd::ShelfAddPaths(vec!["C:\\a\\b.pdf".into(), "C:\\a\\b.pdf".into()]), &wall(9, 0, 0));
        assert_eq!(e.shelf.len(), 1);
        e.tick(UiState { mode: 2, page: 7, clock_tab: 0 }, &wall(9, 0, 1));
        assert_eq!(e.view.shelf[0].name, "b.pdf");
        assert_eq!(Engine::new_offline(settings(), &d).shelf.len(), 1);
    }

    #[test]
    fn refresh_does_not_advance_samplers() {
        let mut e = Engine::new_offline(settings(), &tmpdir("k"));
        e.tick(PANEL_HOME, &wall(10, 0, 0));
        let t = e.ticks;
        e.cmd(Cmd::PomoToggle, &wall(10, 0, 0));
        e.refresh(PANEL_HOME, &wall(10, 0, 0));
        assert_eq!(e.ticks, t, "refresh tidak menambah ticks");
        assert!(e.view.pomo_running, "tetapi view langsung segar");
    }

    #[test]
    fn preview_face_and_idle_peek() {
        let mut e = Engine::new_offline(settings(), &tmpdir("j"));
        let w = wall(14, 0, 0);
        e.tick(PANEL_HOME, &w);
        e.cmd(Cmd::PreviewFace(11), &w);
        e.tick(PANEL_HOME, &wall(14, 0, 1));
        assert_eq!(e.view.face.v.peek, 1.0);
        e.tick(PANEL_HOME, &wall(14, 0, 6));
        assert_eq!(e.view.face.v.peek, 0.0, "pratinjau berakhir");
        e.tick(PANEL_HOME, &wall(14, 3, 0));
        assert_eq!(e.view.face.v.peek, 1.0, "lama tak disentuh -> mengintip");
    }
}
