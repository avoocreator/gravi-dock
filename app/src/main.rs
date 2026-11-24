#![cfg_attr(all(not(debug_assertions), windows), windows_subsystem = "windows")]
mod media;
mod win;

use island_core::{focus::{Focus, Phase}, github, ics, launcher, mood, prayer, sessions as ses, settings::Settings, weather};
use slint::{ComponentHandle, Model, ModelRc, PhysicalPosition, PhysicalSize, SharedString, Timer, TimerMode, VecModel};
use std::{cell::{Cell, RefCell}, path::{Path, PathBuf}, rc::Rc, sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex}, time::Duration};

slint::include_modules!();

const TITLE: &str = "Antigravity Island";
const BOX_W: f32 = 440.0;

fn data_dir() -> PathBuf {
    let base = std::env::var_os("LOCALAPPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    let d = base.join("AntigravityIsland");
    let _ = std::fs::create_dir_all(&d);
    d
}

fn save(s: &Settings) { let _ = std::fs::write(data_dir().join("settings.json"), s.to_json()); }

fn hex(c: slint::Color) -> String { format!("#{:02x}{:02x}{:02x}", c.red(), c.green(), c.blue()) }
fn unhex(s: &str) -> slint::Color {
    let t = s.trim_start_matches('#');
    let p = |a: usize| u8::from_str_radix(t.get(a..a + 2).unwrap_or("00"), 16).unwrap_or(0);
    if t.len() == 6 { slint::Color::from_rgb_u8(p(0), p(2), p(4)) } else { slint::Color::from_rgb_u8(0x5a, 0xe0, 0xff) }
}

fn push_settings(ui: &MainWindow, s: &Settings) {
    ui.global::<Theme>().set_light(s.light);
    ui.global::<Theme>().set_cyan(unhex(&s.accent));
    ui.set_hover_open(s.hover);
    ui.set_adzan(s.adzan);
    ui.set_src_claude(s.src_claude); ui.set_src_copilot(s.src_copilot); ui.set_src_agy(s.src_agy); ui.set_src_desk(s.src_desk);
    ui.set_city(SharedString::from(s.city.as_str()));
    ui.set_dock_pos(SharedString::from(s.position.as_str()));
    ui.set_sfx(s.sfx);
}

fn pull_settings(ui: &MainWindow, s: &mut Settings) {
    s.light = ui.global::<Theme>().get_light();
    s.accent = hex(ui.global::<Theme>().get_cyan());
    s.hover = ui.get_hover_open();
    s.adzan = ui.get_adzan();
    s.src_claude = ui.get_src_claude(); s.src_copilot = ui.get_src_copilot(); s.src_agy = ui.get_src_agy(); s.src_desk = ui.get_src_desk();
    s.position = ui.get_dock_pos().to_string();
    s.sfx = ui.get_sfx();
}

fn style(p: &str) -> (&'static str, slint::Color) {
    let c = |r: u8, g: u8, b: u8| slint::Color::from_rgb_u8(r, g, b);
    match p {
        "agy" => ("agy", c(0x5a, 0xe0, 0xff)),         // Antigravity Desktop (cyan)
        "agycli" => ("agycli", c(0x38, 0xbd, 0xf8)),   // Antigravity CLI (sky blue)
        "chatgpt" => ("chatgpt", c(0x10, 0xa3, 0x7f)), // ChatGPT Desktop (emerald)
        "claude" => ("claude", c(0xf0, 0x8a, 0x5d)),   // Claude Code CLI
        "claudedesk" => ("claudedesk", c(0xe8, 0xa8, 0x7c)), // Claude Desktop (coral)
        "copilot" => ("copilot", c(0x81, 0x8c, 0xf8)), // GitHub Copilot CLI (indigo)
        "gemini" => ("gemini", c(0x42, 0x85, 0xf4)),   // Gemini Desktop (Google blue)
        "hermes" => ("hermes", c(0x06, 0xb6, 0xd4)),   // Hermes (cyan/teal)
        _ => ("agy", c(0x5a, 0xe0, 0xff)),
    }
}

fn ago(ms: i64) -> String {
    let s = (ses::now_ms() - ms).max(0) / 1000;
    if s < 60 { "baru saja".into() } else if s < 3600 { format!("{} mnt", s / 60) } else if s < 86400 { format!("{} j", s / 3600) } else { format!("{} h", s / 86400) }
}

fn to_item(s: &ses::Session) -> SessionItem {
    let (mark, tint) = style(s.provider);
    let running = s.state == ses::State::Running;
    let action = s.state == ses::State::NeedsAction;
    let sub = if !s.current.is_empty() && running { s.current.clone() } else { s.cwd.rsplit(['\\', '/']).next().unwrap_or("").to_string() };
    SessionItem { title: s.title.as_str().into(), sub: sub.into(), meta: if running { "berjalan".into() } else { ago(s.updated_ms).into() }, mark: mark.into(), tint, running, action }
}

/// Jalankan curl.exe (ada bawaan Windows 10+) tanpa jendela konsol.
fn curl(url: &str) -> Option<String> {
    let mut c = std::process::Command::new("curl.exe");
    c.args(["-s", "-L", "-m", "15", url]);
    #[cfg(windows)]
    { use std::os::windows::process::CommandExt; c.creation_flags(0x08000000); }
    let o = c.output().ok()?;
    if o.status.success() { Some(String::from_utf8_lossy(&o.stdout).into_owned()) } else { None }
}

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

#[derive(Default)]
struct Extra { weather_temp: String, weather_desc: String, weather: String, events: Vec<ics::Event>, github: github::GitHubSummary }

fn clock_str(local_secs: i64) -> String { let t = local_secs.rem_euclid(86400); format!("{:02}:{:02}", t / 3600, t % 3600 / 60) }

fn mood_view(c: &mood::Ctx, accent: slint::Color) -> MoodView {
    let v = mood::view(mood::pick(c));
    MoodView {
        eye_h: v.eye_h,
        eye_w: v.eye_w,
        eye_dy: v.eye_dy,
        eye_dx: v.eye_dx,
        eye_shape: v.eye_shape as i32,
        mouth: v.mouth as i32,
        glyph: v.glyph.into(),
        glyph_type: v.glyph_type as i32,
        peek: v.peek,
        eye_color: v.rgb.map(|(r, g, b)| slint::Color::from_rgb_u8(r, g, b)).unwrap_or(accent),
    }
}

fn shelf_file() -> PathBuf { data_dir().join("shelf.txt") }
fn push_shelf(ui: &MainWindow, v: &[String]) {
    let names: Vec<SharedString> = v.iter().map(|p| SharedString::from(p.rsplit(['\\', '/']).next().unwrap_or(p.as_str()))).collect();
    ui.set_shelf(ModelRc::new(VecModel::from(names)));
    let _ = std::fs::write(shelf_file(), v.join("\n"));
}

fn main() {
    if !win::api::single_instance() {
        let _ = std::fs::write(data_dir().join("show.signal"), b"1");
        return;
    }
    let cfg: Arc<Mutex<Settings>> = Arc::new(Mutex::new(Settings::parse(&std::fs::read_to_string(data_dir().join("settings.json")).unwrap_or_default())));
    let ui = MainWindow::new().unwrap();
    push_settings(&ui, &cfg.lock().unwrap());
    let scale = Rc::new(Cell::new(1.0f32));

    // ---- posisi & ukuran ----
    let place = {
        let ui = ui.as_weak(); let cfg = cfg.clone(); let scale = scale.clone();
        Rc::new(move || {
            let Some(ui) = ui.upgrade() else { return };
            let w = ui.window();
            let sf = w.scale_factor();
            scale.set(sf);
            let (wx, wy, ww, wh) = win::api::work_area();
            let h_logic = (wh as f32 / sf - 20.0).clamp(300.0, 720.0);
            let top = (wh as f32 / sf - h_logic) * cfg.lock().unwrap().top_pct as f32;
            let pos = cfg.lock().unwrap().position.clone();

            let (px, py) = match pos.as_str() {
                "top" => {
                    let cx = wx + (ww - (BOX_W * sf) as i32) / 2;
                    (cx, wy)
                }
                "left" => {
                    (wx, wy + (top * sf) as i32)
                }
                _ => { // "right"
                    (wx + ww - (BOX_W * sf) as i32, wy + (top * sf) as i32)
                }
            };
            w.set_size(PhysicalSize::new((BOX_W * sf) as u32, (h_logic * sf) as u32));
            w.set_position(PhysicalPosition::new(px, py));
        })
    };
    ui.show().unwrap();
    place();
    win::api::tray();
    let hwnd = Rc::new(Cell::new(win::api::hwnd(TITLE)));
    win::api::set_overlay_window(hwnd.get());
    win::api::set_passthrough(hwnd.get(), true);
    win::api::topmost(hwnd.get());

    // ---- hotkey & sinyal instance kedua ----
    let want_toggle = Arc::new(AtomicBool::new(false));
    { let f = want_toggle.clone(); win::api::hotkey(move || f.store(true, Ordering::SeqCst)); }
    let signal = data_dir().join("show.signal");

    // ---- thread: sesi AI ----
    let shared: Arc<Mutex<Vec<ses::Session>>> = Arc::new(Mutex::new(vec![]));
    {
        let shared = shared.clone(); let cfg = cfg.clone();
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
                std::thread::sleep(Duration::from_secs(3));
            }
        });
    }

    // ---- thread: daftar aplikasi (Start Menu) ----
    let apps: Arc<Mutex<Vec<(String, String)>>> = Arc::new(Mutex::new(vec![]));
    {
        let apps = apps.clone();
        std::thread::spawn(move || {
            let mut v = vec![];
            for k in ["APPDATA", "PROGRAMDATA"] {
                if let Some(b) = std::env::var_os(k) { scan_start_menu(&PathBuf::from(b).join("Microsoft/Windows/Start Menu/Programs"), 3, &mut v); }
            }
            v.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase())); v.dedup_by(|a, b| a.0 == b.0);
            *apps.lock().unwrap() = v;
        });
    }

    // ---- thread: cuaca + kalender + github tiap 15 menit ----
    let extra: Arc<Mutex<Extra>> = Arc::new(Mutex::new(Extra::default()));
    {
        let extra = extra.clone(); let cfg = cfg.clone();
        std::thread::spawn(move || loop {
            let (lat, lon, ical) = { let c = cfg.lock().unwrap(); (c.lat, c.lon, c.ical.clone()) };
            let w_parsed = curl(&weather::url(lat, lon)).and_then(|b| weather::parse(&b));
            let (w_temp, w_desc, w_full) = match w_parsed {
                Some((t, d)) => (format!("{:.0}°C", t), d.to_string(), format!("{:.0}°C {}", t, d)),
                None => ("29°C".into(), "Berawan".into(), "29°C Berawan".into()),
            };
            let ev = if ical.starts_with("http") { curl(&ical).map(|b| ics::upcoming(&b, ses::now_ms() / 1000, 7 * 86400, win::api::tz_hours())) } else { None };
            let gh_json = curl("https://api.github.com/users/avoocreator/events");
            let today_ymd = {
                let n = win::api::now();
                format!("{:04}-{:02}-{:02}", n.y, n.mo, n.d)
            };
            let gh_sum = gh_json.map(|j| github::parse_events(&j, &today_ymd)).unwrap_or_else(|| {
                let mut d = github::GitHubSummary::default();
                d.recent_repo = "gravi-dock".into();
                d.recent_commit = "feat: add multi-docking support".into();
                let mut map = vec![0u8; 42];
                for &idx in &[5, 9, 14, 18, 22, 25, 28, 32, 35, 40] { map[idx] = (idx % 3 + 1) as u8; }
                d.heatmap = map;
                d
            });
            {
                let mut e = extra.lock().unwrap();
                e.weather_temp = w_temp;
                e.weather_desc = w_desc;
                e.weather = w_full;
                if let Some(ev) = ev { e.events = ev; }
                e.github = gh_sum;
            }
            std::thread::sleep(Duration::from_secs(900));
        });
    }

    // ---- status UI ----
    let focus = Rc::new(RefCell::new({ let c = cfg.lock().unwrap(); Focus::new(c.focus_min, c.break_min) }));
    let clips: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(vec![]));
    let targets: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(vec![]));
    let shelf: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(std::fs::read_to_string(shelf_file()).unwrap_or_default().lines().filter(|l| !l.is_empty()).map(String::from).collect()));
    push_shelf(&ui, &shelf.borrow());
    let media = Rc::new(RefCell::new(media::Media::new(&data_dir())));
    let media_idle = Rc::new(Cell::new(0u32));
    let ctx_cell = Rc::new(Cell::new(mood::Ctx::default()));
    let hover_now = Rc::new(Cell::new(false));
    let done_at = Rc::new(Cell::new(0i64));
    let click_at = Rc::new(Cell::new(0i64));
    let back = |ui: &MainWindow| { let n = ui.get_live().row_count(); ui.set_mode(if n > 0 { 1 } else { 0 }); };

    // ---- callback ----
    ui.on_clicked_open({ let ui = ui.as_weak(); let hw = hwnd.clone(); let ca = click_at.clone(); move || { if let Some(u) = ui.upgrade() { ca.set(ses::now_ms() / 1000); u.set_mode(2); win::api::focus(hw.get()); } } });
    ui.on_clicked_close({ let ui = ui.as_weak(); move || { if let Some(u) = ui.upgrade() { back(&u); } } });
    ui.on_quit(|| { let _ = slint::quit_event_loop(); });
    ui.on_settings_changed({
        let ui = ui.as_weak(); let cfg = cfg.clone(); let place = place.clone();
        move || {
            if let Some(u) = ui.upgrade() {
                let mut c = cfg.lock().unwrap();
                pull_settings(&u, &mut c);
                save(&c);
                drop(c);
                place();
            }
        }
    });
    ui.on_cycle_city({
        let ui = ui.as_weak(); let cfg = cfg.clone();
        move || {
            let Some(u) = ui.upgrade() else { return };
            let mut c = cfg.lock().unwrap();
            let i = prayer::CITIES.iter().position(|x| x.0 == c.city).map(|i| (i + 1) % prayer::CITIES.len()).unwrap_or(0);
            let (n, la, lo) = prayer::CITIES[i];
            c.city = n.to_string(); c.lat = la; c.lon = lo;
            pull_settings(&u, &mut c); save(&c);
            u.set_city(n.into());
        }
    });
    ui.on_focus_start({
        let f = focus.clone();
        move || { let now = ses::now_ms() / 1000; let mut f = f.borrow_mut(); if f.phase == Phase::Idle { f.start(now) } else { f.toggle_pause(now) } }
    });
    ui.on_focus_pause({ let f = focus.clone(); move || f.borrow_mut().toggle_pause(ses::now_ms() / 1000) });
    ui.on_focus_stop({ let f = focus.clone(); move || f.borrow_mut().stop() });
    let sw_running = Rc::new(Cell::new(false));
    let sw_secs = Rc::new(Cell::new(0u64));
    ui.on_sw_toggle({
        let sw_running = sw_running.clone();
        let ui = ui.as_weak();
        move || {
            let r = !sw_running.get();
            sw_running.set(r);
            if let Some(u) = ui.upgrade() { u.set_sw_running(r); }
        }
    });
    ui.on_sw_reset({
        let sw_running = sw_running.clone();
        let sw_secs = sw_secs.clone();
        let ui = ui.as_weak();
        move || {
            sw_running.set(false);
            sw_secs.set(0);
            if let Some(u) = ui.upgrade() {
                u.set_sw_running(false);
                u.set_sw_label("00:00".into());
            }
        }
    });
    ui.on_copy_clip({ let c = clips.clone(); move |i| { if let Some(t) = c.borrow().get(i as usize) { win::api::clip_set(t); } } });
    ui.on_query_changed({
        let ui = ui.as_weak(); let apps = apps.clone(); let targets = targets.clone();
        move |q| {
            let Some(u) = ui.upgrade() else { return };
            let q = q.trim().to_string();
            let mut items: Vec<LaunchItem> = vec![]; let mut tg: Vec<String> = vec![];
            if let Some(v) = launcher::calc(&q) {
                let s = if v.fract() == 0.0 && v.abs() < 1e15 { format!("{}", v as i64) } else { format!("{}", (v * 1e8).round() / 1e8) };
                items.push(LaunchItem { name: format!("= {}", s).into(), hint: "salin".into() }); tg.push(format!("copy:{}", s));
            }
            if let Some(url) = launcher::detect_url(&q) { items.push(LaunchItem { name: url.as_str().into(), hint: "buka web".into() }); tg.push(url); }
            if !q.is_empty() {
                let list = apps.lock().unwrap();
                for a in launcher::rank(&list, |a| a.0.as_str(), &q, 8) { items.push(LaunchItem { name: a.0.as_str().into(), hint: "aplikasi".into() }); tg.push(a.1.clone()); }
            }
            *targets.borrow_mut() = tg;
            u.set_results(ModelRc::new(VecModel::from(items)));
        }
    });
    ui.on_run_result({
        let ui = ui.as_weak(); let targets = targets.clone();
        move |i| {
            let t = targets.borrow().get(i as usize).cloned();
            let Some(t) = t else { return };
            if let Some(v) = t.strip_prefix("copy:") { win::api::clip_set(v); } else { win::api::shell_open(&t); if let Some(u) = ui.upgrade() { back(&u); } }
        }
    });

    ui.on_media_cmd({ let m = media.clone(); move |c| m.borrow_mut().send(&c) });
    ui.on_shelf_add({
        let ui = ui.as_weak(); let sh = shelf.clone();
        move || {
            let Some(u) = ui.upgrade() else { return };
            let mut v = sh.borrow_mut();
            for f in win::api::clip_files() { if !v.contains(&f) { v.insert(0, f); } }
            v.truncate(40);
            push_shelf(&u, &v);
        }
    });
    ui.on_shelf_open({ let sh = shelf.clone(); move |i| { if let Some(p) = sh.borrow().get(i as usize) { win::api::shell_open(p); } } });
    ui.on_shelf_remove({
        let ui = ui.as_weak(); let sh = shelf.clone();
        move |i| { let Some(u) = ui.upgrade() else { return }; let mut v = sh.borrow_mut(); if (i as usize) < v.len() { v.remove(i as usize); } push_shelf(&u, &v); }
    });
    ui.on_gh_open(|| { win::api::shell_open("https://github.com/avoocreator"); });
    ui.on_cal_open(|| { win::api::shell_open("https://calendar.google.com"); });
    ui.on_session_action({
        let ui = ui.as_weak();
        move |_idx| {
            win::api::play_action_sfx();
            win::api::send_action_confirm();
            if let Some(u) = ui.upgrade() {
                u.set_greeting("Izin disetujui ✓".into());
            }
        }
    });
    ui.on_play_sfx({
        let cfg = cfg.clone();
        move || {
            if cfg.lock().unwrap().sfx {
                win::api::play_click();
            }
        }
    });

    // ---- timer 1 dtk ----
    let last_cpu = Rc::new(Cell::new(win::api::cpu_times()));
    let last_clip = Rc::new(Cell::new(win::api::clip_seq()));
    let last_adzan = Rc::new(Cell::new(-1i64));
    let cpu_hist = Rc::new(RefCell::new(Vec::<f32>::new()));
    let t1 = Timer::default();
    {
        let ui = ui.as_weak(); let cfg = cfg.clone(); let shared = shared.clone(); let extra = extra.clone();
        let focus = focus.clone(); let clips = clips.clone();
        let (media, media_idle, ctx_cell, hover_now, done_at, click_at, last_adzan) = (media.clone(), media_idle.clone(), ctx_cell.clone(), hover_now.clone(), done_at.clone(), click_at.clone(), last_adzan.clone());
        let (sw_running, sw_secs, cpu_hist) = (sw_running.clone(), sw_secs.clone(), cpu_hist.clone());
        t1.start(TimerMode::Repeated, Duration::from_secs(1), move || {
            let Some(ui) = ui.upgrade() else { return };
            let n = win::api::now();
            let tz = win::api::tz_hours();
            let (lat, lon, adzan_on, wav) = { let c = cfg.lock().unwrap(); (c.lat, c.lon, c.adzan, c.adzan_wav.clone()) };
            ui.set_user_name("Avian".into());
            ui.set_clock(format!("{:02}:{:02}", n.h, n.mi).into());
            const HARI: [&str; 7] = ["Minggu", "Senin", "Selasa", "Rabu", "Kamis", "Jumat", "Sabtu"];
            const BLN: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "Mei", "Jun", "Jul", "Agu", "Sep", "Okt", "Nov", "Des"];
            ui.set_date(format!("{}, {} {} {}", HARI[(n.dow % 7) as usize].to_uppercase(), n.d, BLN[(n.mo - 1).clamp(0, 11) as usize].to_uppercase(), n.y).into());
            ui.set_greeting(match n.h { 4..=10 => "Selamat pagi", 11..=14 => "Selamat siang", 15..=17 => "Selamat sore", _ => "Selamat malam" }.into());

            // sholat + adzan
            let day = prayer::days_from_civil(n.y, n.mo, n.d);
            let local = day * 86400 + n.h * 3600 + n.mi * 60 + n.s;
            let nx = prayer::next_prayer(local, lat, lon, tz);
            ui.set_prayer_name(nx.prayer.name().into());
            ui.set_prayer_time(clock_str(nx.at_local_secs).into());
            let l = nx.left_secs;
            ui.set_prayer_left(if l >= 3600 { format!("{} j {} mnt lagi", l / 3600, l % 3600 / 60) } else { format!("{} mnt lagi", (l + 59) / 60) }.into());
            if adzan_on && n.s < 3 {
                let t = prayer::times(n.y, n.mo, n.d, lat, lon, tz);
                let min = n.h * 60 + n.mi;
                let key = day * 1440 + min;
                if [1usize, 3, 4, 5, 6].iter().any(|&i| t[i] as i64 == min) && last_adzan.get() != key { last_adzan.set(key); win::api::play_wav(&wav); }
            }

            // sistem
            let (idle, tot) = win::api::cpu_times();
            let (pi, pt) = last_cpu.replace((idle, tot));
            let dt = tot.saturating_sub(pt) as f32;
            let cpu = if dt > 0.0 { (1.0 - idle.saturating_sub(pi) as f32 / dt).clamp(0.0, 1.0) } else { 0.0 };
            ui.set_cpu(cpu); ui.set_cpu_text(format!("{:.0}%", cpu * 100.0).into());
            let (m, used, total) = win::api::mem();
            ui.set_mem(m); ui.set_mem_text(format!("{:.1}/{:.0} GB", used, total).into());
            ui.set_uptime(win::api::uptime_str().into());

            // cpu sparkline
            {
                let mut hist = cpu_hist.borrow_mut();
                hist.push(cpu);
                if hist.len() > 10 { hist.remove(0); }
                let n = hist.len();
                if n > 1 {
                    let mut cmd = String::new();
                    for (i, &v) in hist.iter().enumerate() {
                        let x = (i as f32 * 58.0 / (n - 1) as f32) as i32;
                        let y = (27.0 - (v.clamp(0.0, 1.0) * 23.0)) as i32;
                        if i == 0 { cmd.push_str(&format!("M {} {} ", x, y)); }
                        else { cmd.push_str(&format!("L {} {} ", x, y)); }
                    }
                    ui.set_cpu_sparkline(cmd.into());
                }
            }

            // stopwatch
            if sw_running.get() {
                let s = sw_secs.get() + 1;
                sw_secs.set(s);
                ui.set_sw_label(format!("{:02}:{:02}", s / 60, s % 60).into());
            }

            // cuaca + agenda + github
            {
                let e = extra.lock().unwrap();
                if !e.weather_temp.is_empty() {
                    ui.set_weather_temp(e.weather_temp.as_str().into());
                    ui.set_weather_desc(e.weather_desc.as_str().into());
                }
                if !e.weather.is_empty() { ui.set_weather_text(e.weather.as_str().into()); }
                let now_utc = ses::now_ms() / 1000;
                let txt = match e.events.iter().find(|x| x.end > now_utc) {
                    None => "Tidak ada agenda".to_string(),
                    Some(ev) => {
                        let ls = ev.start + (tz * 3600.0) as i64;
                        let dd = ls.div_euclid(86400) - day;
                        let when = if ev.all_day { "Seharian".to_string() } else { clock_str(ls) };
                        let pre = match dd { i64::MIN..=0 => "Hari ini".to_string(), 1 => "Besok".to_string(), _ => { let (_, mo, d) = prayer::civil_from_days(ls.div_euclid(86400)); format!("{}/{}", d, mo) } };
                        format!("{} {}  {}", pre, when, ev.title)
                    }
                };
                ui.set_event_text(txt.into());
                ui.set_gh_commits_today(e.github.commits_today as i32);
                if !e.github.recent_repo.is_empty() { ui.set_gh_recent_repo(e.github.recent_repo.as_str().into()); }
                if !e.github.recent_commit.is_empty() { ui.set_gh_recent_commit(e.github.recent_commit.as_str().into()); }
                if e.github.heatmap.len() >= 42 {
                    let hm: Vec<i32> = e.github.heatmap.iter().map(|&x| x as i32).collect();
                    ui.set_gh_heatmap(ModelRc::new(VecModel::from(hm)));
                }
            }

            // fokus
            {
                let now = ses::now_ms() / 1000;
                let mut f = focus.borrow_mut();
                if f.tick(now) { win::api::play_wav(""); done_at.set(now); }
                ui.set_focus_label(f.label(now).into());
                ui.set_focus_phase(match f.phase { Phase::Idle => "Siap", Phase::Focus => "Fokus", Phase::Break => "Istirahat" }.into());
                ui.set_focus_running(f.phase != Phase::Idle && f.paused_left.is_none());
                ui.set_focus_done(f.done as i32);
            }

            // clipboard
            let seq = win::api::clip_seq();
            if seq != last_clip.get() {
                last_clip.set(seq);
                if let Some(t) = win::api::clip_get() {
                    let t = t.trim().to_string();
                    if !t.is_empty() {
                        let mut c = clips.borrow_mut();
                        c.retain(|x| *x != t); c.insert(0, t); c.truncate(12);
                        let v: Vec<SharedString> = c.iter().map(|x| SharedString::from(x.lines().next().unwrap_or("").chars().take(80).collect::<String>())).collect();
                        ui.set_clips(ModelRc::new(VecModel::from(v)));
                    }
                }
            }

            // sesi
            let list = shared.lock().unwrap();
            let items: Vec<SessionItem> = list.iter().map(to_item).collect();
            let live: Vec<SessionItem> = items.iter().filter(|s| s.running || s.action).take(3).cloned().collect();
            ui.set_running_count(list.iter().filter(|s| s.state == ses::State::Running).count() as i32);
            let had_live = ui.get_live().row_count() > 0;
            ui.set_sessions(ModelRc::new(VecModel::from(items)));
            ui.set_live(ModelRc::new(VecModel::from(live.clone())));
            if ui.get_mode() < 2 && (!live.is_empty()) != had_live { ui.set_mode(if live.is_empty() { 0 } else { 1 }); }

            // musik: pembantu PowerShell hanya hidup saat tab Musik dibuka (+30 dtk), agar memori tetap kecil
            let music = {
                let want = ui.get_mode() == 2 && ui.get_tab() == 2;
                let mut md = media.borrow_mut();
                if want { media_idle.set(0); if !md.running() { md.start(); } }
                else if md.running() { media_idle.set(media_idle.get() + 1); if media_idle.get() >= 30 { md.stop(); } }
                if md.running() {
                    let st = md.state.lock().unwrap().clone();
                    ui.set_media_active(st.active); ui.set_media_playing(st.playing);
                    ui.set_media_title(if st.active { st.title.as_str() } else { "Memuat…" }.into()); ui.set_media_artist(st.artist.as_str().into());
                    st.playing
                } else { false }
            };

            // suasana hati maskot
            let nowc = ses::now_ms() / 1000;
            let fph = focus.borrow().phase;
            let cx = mood::Ctx {
                hour: n.h, running: ui.get_running_count(), action: list.iter().any(|s| s.state == ses::State::NeedsAction), cpu, mem: m,
                focus: fph == Phase::Focus, brk: fph == Phase::Break, music, hover: hover_now.get(),
                prayer_left_s: l, prayer_now: last_adzan.get() >= day * 1440 + n.h * 60 + n.mi - 1, just_done: nowc - done_at.get() < 5, clicked_recently: nowc - click_at.get() < 3,
                is_peek: ui.get_mode() == 0, is_searching: ui.get_tab() == 5, is_error: false,
            };
            ctx_cell.set(cx);
            ui.set_mood(mood_view(&cx, ui.global::<Theme>().get_cyan()));
        });
    }

    // ---- timer 40 ms: hit-test, click-through, hover, fokus ----
    let t2 = Timer::default();
    {
        let ui = ui.as_weak(); let hwnd = hwnd.clone(); let scale = scale.clone(); let place = place.clone();
        let (ctx_cell, hover_now) = (ctx_cell.clone(), hover_now.clone());
        let (inside_ms, outside_ms, lost_ms, tick, pass) = (Cell::new(0u32), Cell::new(0u32), Cell::new(0u32), Cell::new(0u32), Cell::new(true));
        t2.start(TimerMode::Repeated, Duration::from_millis(40), move || {
            let Some(ui) = ui.upgrade() else { return };
            tick.set(tick.get() + 1);
            ui.set_wave_tick(tick.get() as i32);
            let mut toggle = want_toggle.swap(false, Ordering::SeqCst);
            if tick.get() % 10 == 0 && signal.exists() { let _ = std::fs::remove_file(&signal); toggle = true; }
            if tick.get() % 250 == 0 { place(); win::api::topmost(hwnd.get()); }
            if hwnd.get() == 0 {
                let h = win::api::hwnd(TITLE);
                if h != 0 {
                    hwnd.set(h);
                    win::api::set_overlay_window(h);
                }
            }
            match win::api::TRAY_EVENT.swap(0, Ordering::SeqCst) { 1 => toggle = true, 2 => { let _ = slint::quit_event_loop(); } _ => {} }
            if toggle { if ui.get_mode() == 2 { back(&ui); } else { ui.set_mode(2); win::api::focus(hwnd.get()); } }

            let pos = ui.window().position();
            let sf = scale.get().max(0.5);
            let (cx, cy) = win::api::cursor();
            let (lx, ly) = ((cx - pos.x) as f32 / sf, (cy - pos.y) as f32 / sf);
            let (hx, hy, hw, hh) = (ui.get_hit_x(), ui.get_hit_y(), ui.get_hit_w(), ui.get_hit_h());
            let inside = lx >= hx && lx <= hx + hw && ly >= hy && ly <= hy + hh;
            if !inside != pass.get() { pass.set(!inside); win::api::set_passthrough(hwnd.get(), !inside); }

            if inside != hover_now.get() { hover_now.set(inside); let mut c = ctx_cell.get(); c.hover = inside; ctx_cell.set(c); ui.set_mood(mood_view(&c, ui.global::<Theme>().get_cyan())); ui.set_is_hovered(inside); }
            let mode = ui.get_mode();
            if inside { inside_ms.set(inside_ms.get() + 40); outside_ms.set(0); } else { outside_ms.set(outside_ms.get() + 40); inside_ms.set(0); }
            if ui.get_hover_open() {
                if mode < 2 && inside_ms.get() >= 280 { ui.set_mode(2); }
                if mode == 2 && outside_ms.get() >= 650 && ui.get_tab() != 4 { back(&ui); }
            } else if mode == 2 {
                if win::api::foreground() != hwnd.get() { lost_ms.set(lost_ms.get() + 40); } else { lost_ms.set(0); }
                if lost_ms.get() >= 400 && !inside { back(&ui); lost_ms.set(0); }
            } else { lost_ms.set(0); }
        });
    }

    // ---- kedip & napas ----
    let t3 = Timer::default();
    {
        let ui = ui.as_weak(); let n = Cell::new(0u32);
        t3.start(TimerMode::Repeated, Duration::from_millis(450), move || {
            let Some(ui) = ui.upgrade() else { return };
            n.set(n.get() + 1);
            ui.set_breathe(n.get() % 2 == 0);
            ui.set_blink(n.get() % 11 == 0);
        });
    }
    ui.run().unwrap();
}
