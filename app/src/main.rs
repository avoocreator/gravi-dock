#![cfg_attr(all(not(debug_assertions), windows), windows_subsystem = "windows")]
//! Gravi Island: perekat tipis antara `Engine` (logika) dan UI Slint, plus pengaturan jendela (posisi, animasi pil <-> panel).
mod audio;
mod engine;
mod hooks;
mod media;
mod net;
mod online;
mod win;

use engine::{Cmd, Effect, Engine, Rgb, UiState, View, Wall};
use island_core::settings::Settings;
use island_core::theme::{self, Dock, Rect};
use slint::{Color, ComponentHandle, ModelRc, SharedString, Timer, TimerMode, VecModel};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

slint::include_modules!();

const TITLE: &str = "Gravi Island";
// ukuran logis (dp)
const PILL_V: (i32, i32) = (76, 122);
const PILL_H: (i32, i32) = (340, 62);
const PANEL: (i32, i32) = (440, 660);
const ALERT: (i32, i32) = (420, 290);
const MARGIN: i32 = 12;
const ANIM_MS: u64 = 300;

/* ---------------------------------------------------------------- konversi tipe ---------------------------------------------------------------- */

fn col(c: Rgb) -> Color { Color::from_rgb_u8(c.0, c.1, c.2) }
fn s(x: &String) -> SharedString { x.as_str().into() }
fn cp<T: Copy>(x: &T) -> T { *x }
fn model<T, U: Clone + 'static>(v: &[T], f: impl Fn(&T) -> U) -> ModelRc<U> { ModelRc::new(VecModel::from(v.iter().map(f).collect::<Vec<_>>())) }

fn face_params(f: &engine::FaceV) -> FaceParams {
    let v = &f.v;
    FaceParams {
        eye_h: v.eye_h, eye_w: v.eye_w, r_h: v.r_h, r_w: v.r_w, arc: v.arc, ring: v.ring, lid_out: v.lid_out, lid_in: v.lid_in,
        look_x: v.look_x, look_y: v.look_y, peek: v.peek, glyph: v.glyph.into(), glyph_color: Color::from_rgb_u8(v.glyph_rgb.0, v.glyph_rgb.1, v.glyph_rgb.2),
    }
}

/// Salin hanya bagian `View` yang berubah ke UI.
fn apply(ui: &MainWindow, v: &View, l: &View) {
    let st = ui.global::<AppState>();
    macro_rules! diff { ($($f:ident => $set:ident($conv:expr)),* $(,)?) => { $( if v.$f != l.$f { st.$set($conv(&v.$f)); } )* } }
    diff! {
        clock => set_clock(s), clock_sec => set_clock_sec(s), date_long => set_date_long(s), greeting => set_greeting(s),
        face => set_face(face_params), badge => set_badge(cp), ai_running => set_ai_running(cp), toast => set_toast(s), toast_kind => set_toast_kind(cp), mascot_on => set_mascot_on(cp),
        weather => set_weather(s), weather_temp => set_weather_temp(s), weather_desc => set_weather_desc(s), weather_city => set_weather_city(s), next_prayer => set_next_prayer(s), next_prayer_time => set_next_prayer_time(s),
        next_prayer_left => set_next_prayer_left(s), pill_line => set_pill_line(s),
        prayers => set_prayers(|x: &Vec<engine::PrayerRow>| model(x, |r| PrayerRow { name: s(&r.name), time: s(&r.time), next: r.next, past: r.past })),
        agenda => set_agenda(|x: &Vec<engine::EventRow>| model(x, ev_item)),
        hits => set_hits(|x: &Vec<engine::HitRow>| model(x, |r| SearchHit { title: s(&r.title), sub: s(&r.sub), kind: s(&r.kind) })),
        sessions => set_sessions(|x: &Vec<engine::SessionRow>| model(x, |r| SessionItem { id: s(&r.id), title: s(&r.title), sub: s(&r.sub), meta: s(&r.meta), mark: s(&r.mark), tint: col(r.tint), running: r.running, action: r.action })),
        approvals => set_approvals(|x: &Vec<engine::ApprovalRow>| model(x, |r| ApprovalItem { id: s(&r.id), tool: s(&r.tool), summary: s(&r.summary), cwd: s(&r.cwd), mark: s(&r.mark), tint: col(r.tint), src: s(&r.src) })),
        m_active => set_m_active(cp), m_playing => set_m_playing(cp), m_title => set_m_title(s), m_artist => set_m_artist(s), m_source => set_m_source(s),
        m_pos => set_m_pos(cp), m_pos_text => set_m_pos_text(s), m_pos_l => set_m_pos_l(s), m_pos_r => set_m_pos_r(s),
        pomo_text => set_pomo_text(s), pomo_phase => set_pomo_phase(s), pomo_progress => set_pomo_progress(cp), pomo_running => set_pomo_running(cp), pomo_rounds => set_pomo_rounds(s),
        timer_text => set_timer_text(s), timer_progress => set_timer_progress(cp), timer_state => set_timer_state(cp),
        sw_text => set_sw_text(s), sw_running => set_sw_running(cp),
        laps => set_laps(|x: &Vec<engine::LapRow>| model(x, |r| LapItem { n: s(&r.n), lap: s(&r.lap), total: s(&r.total) })),
        alarms => set_alarms(|x: &Vec<engine::AlarmRow>| model(x, |r| AlarmItem { id: r.id, time: s(&r.time), label: s(&r.label), repeat: s(&r.repeat), on: r.on })),
        alarm_next => set_alarm_next(s), adzan_note => set_adzan_note(s),
        cal_month => set_cal_month(s), cal_status => set_cal_status(s), cal_connected => set_cal_connected(cp),
        cal_week => set_cal_week(|x: &Vec<engine::DayCell>| model(x, |r| DayCell { label: s(&r.label), num: s(&r.num), today: r.today, dot: r.dot })),
        cal_events => set_cal_events(|x: &Vec<engine::EventRow>| model(x, ev_item)),
        gh_user => set_gh_user(s), gh_status => set_gh_status(s), gh_total => set_gh_total(s), gh_streak => set_gh_streak(s), gh_today => set_gh_today(s),
        gh_cells => set_gh_cells(|x: &Vec<i32>| model(x, |c| *c)),
        commits => set_commits(|x: &Vec<engine::CommitRow>| model(x, |r| CommitRow { repo: s(&r.repo), message: s(&r.message), meta: s(&r.meta) })),
        repos => set_repos(|x: &Vec<engine::RepoRow>| model(x, |r| RepoRow { name: s(&r.name), meta: s(&r.meta), url: s(&r.url) })),
        cpu => set_cpu(cp), mem => set_mem(cp), disk => set_disk(cp), battery => set_battery(cp), has_battery => set_has_battery(cp), charging => set_charging(cp),
        cpu_text => set_cpu_text(s), mem_text => set_mem_text(s), disk_text => set_disk_text(s), temp_text => set_temp_text(s), uptime_text => set_uptime_text(s),
        cpu_line => set_cpu_line(s), cpu_area => set_cpu_area(s),
        clips => set_clips(|x: &Vec<engine::ClipRow>| model(x, |r| ClipItem { text: s(&r.text), pinned: r.pinned })),
        shelf => set_shelf(|x: &Vec<engine::ShelfRow>| model(x, |r| ShelfItem { name: s(&r.name), path: s(&r.path) })),
        hook_claude => set_hook_claude(cp), hook_copilot => set_hook_copilot(cp), settings_note => set_settings_note(s),
    }
    let th = ui.global::<Theme>();
    if v.dark != l.dark { th.set_dark(v.dark); }
    if v.accent != l.accent { th.set_accent(col(v.accent)); }
    if v.accent2 != l.accent2 { th.set_accent2(col(v.accent2)); }
}

fn ev_item(r: &engine::EventRow) -> EventItem { EventItem { title: s(&r.title), time: s(&r.time), sub: s(&r.sub), tint: col(r.tint), all_day: r.all_day } }

/* ---------------------------------------------------------------- pengaturan <-> UI ---------------------------------------------------------------- */

fn push_settings(ui: &MainWindow, c: &Settings) {
    let a = ui.global::<AppState>();
    a.set_s_dock(c.dock.index()); a.set_s_theme(c.theme.index()); a.set_s_scheme(theme::scheme_index(&c.scheme));
    a.set_s_edge(c.edge_pct as f32); a.set_s_hover(c.hover); a.set_s_mascot(c.mascot); a.set_s_gpu(c.renderer == "gpu");
    a.set_s_sfx(c.sfx); a.set_s_sfx_vol(c.sfx_vol as f32 / 100.0);
    a.set_s_adzan(c.adzan); a.set_s_adzan_vol(c.adzan_vol as f32 / 100.0); a.set_s_adzan_file(c.adzan_file.as_str().into());
    a.set_s_focus_min(c.focus_min as i32); a.set_s_break_min(c.break_min as i32); a.set_s_long_min(c.long_break_min as i32); a.set_s_rounds(c.rounds as i32); a.set_s_auto_next(c.auto_next);
    a.set_s_city(c.city.as_str().into());
    a.set_s_src_claude(c.src_claude); a.set_s_src_copilot(c.src_copilot); a.set_s_src_agy(c.src_agy); a.set_s_src_desk(c.src_desk);
    a.set_s_gh_user(c.gh_user.as_str().into()); a.set_s_gh_token(c.gh_token.as_str().into());
    a.set_s_gcal_id(c.gcal_client_id.as_str().into()); a.set_s_gcal_secret(c.gcal_client_secret.as_str().into()); a.set_s_ical(c.ical.as_str().into());
    a.set_scheme_names(model(theme::SCHEMES, |x| SharedString::from(x.label)));
    a.set_scheme_colors(model(theme::SCHEMES, |x| { let (r, g, b) = theme::parse_hex(x.accent).unwrap_or((77, 163, 255)); Color::from_rgb_u8(r, g, b) }));
}

fn pull_settings(ui: &MainWindow, base: &Settings) -> Settings {
    let a = ui.global::<AppState>();
    let mut c = base.clone();
    c.dock = Dock::from_index(a.get_s_dock()); c.theme = theme::ThemeMode::from_index(a.get_s_theme());
    c.scheme = theme::SCHEMES.get(a.get_s_scheme().max(0) as usize).map(|x| x.id).unwrap_or("ocean").to_string();
    c.edge_pct = a.get_s_edge() as f64; c.hover = a.get_s_hover(); c.mascot = a.get_s_mascot();
    c.renderer = if a.get_s_gpu() { "gpu".into() } else { "software".into() };
    c.sfx = a.get_s_sfx(); c.sfx_vol = (a.get_s_sfx_vol() * 100.0).round() as i64;
    c.adzan = a.get_s_adzan(); c.adzan_vol = (a.get_s_adzan_vol() * 100.0).round() as i64; c.adzan_file = a.get_s_adzan_file().to_string();
    c.focus_min = a.get_s_focus_min() as i64; c.break_min = a.get_s_break_min() as i64; c.long_break_min = a.get_s_long_min() as i64; c.rounds = a.get_s_rounds() as i64; c.auto_next = a.get_s_auto_next();
    c.city = a.get_s_city().to_string();
    c.src_claude = a.get_s_src_claude(); c.src_copilot = a.get_s_src_copilot(); c.src_agy = a.get_s_src_agy(); c.src_desk = a.get_s_src_desk();
    c.gh_user = a.get_s_gh_user().to_string(); c.gh_token = a.get_s_gh_token().to_string();
    c.gcal_client_id = a.get_s_gcal_id().to_string(); c.gcal_client_secret = a.get_s_gcal_secret().to_string(); c.ical = a.get_s_ical().to_string();
    c
}

/* ---------------------------------------------------------------- jendela ---------------------------------------------------------------- */

struct Shell { mode: i32, abs: Rect, gen: u32, alert_seen: usize, inside: u32, outside: u32, lost: u32, tick: u32 }

fn plan(cfg: &Settings, mode: i32, sf: f32) -> theme::Placement {
    let (x, y, w, h) = win::api::work_area();
    let px = |v: i32| (v as f32 * sf).round() as i32;
    let pill = if cfg.dock.vertical() { PILL_V } else { PILL_H };
    let target = match mode { 0 => pill, 1 => ALERT, _ => PANEL };
    theme::place(cfg.dock, Rect { x, y, w, h }, (px(pill.0), px(pill.1)), (px(target.0), px(target.1)), cfg.edge_pct, px(MARGIN), mode != 0)
}

fn set_window(ui: &MainWindow, r: Rect) {
    ui.window().set_position(slint::PhysicalPosition::new(r.x, r.y));
    ui.window().set_size(slint::PhysicalSize::new(r.w.max(1) as u32, r.h.max(1) as u32));
}

fn set_isl(ui: &MainWindow, snap: bool, r: (f32, f32, f32, f32)) {
    let a = ui.global::<AppState>();
    a.set_snap(snap);
    a.set_isl_x(r.0); a.set_isl_y(r.1); a.set_isl_w(r.2); a.set_isl_h(r.3);
}

#[derive(Clone)]
struct Ctl {
    ui: slint::Weak<MainWindow>,
    eng: Rc<RefCell<Engine>>,
    sh: Rc<RefCell<Shell>>,
    hwnd: Rc<Cell<usize>>,
    last: Rc<RefCell<View>>,
    fast: Rc<Timer>,
}

impl Ctl {
    fn cfg(&self) -> Settings { self.eng.borrow().cfg.lock().unwrap().clone() }

    fn ui_state(&self, ui: &MainWindow) -> UiState {
        let a = ui.global::<AppState>();
        UiState { mode: self.sh.borrow().mode, page: a.get_page(), clock_tab: a.get_clock_tab() }
    }

    fn render(&self) {
        let Some(ui) = self.ui.upgrade() else { return };
        let v = self.eng.borrow().view.clone();
        let mut l = self.last.borrow_mut();
        apply(&ui, &v, &l);
        *l = v;
    }

    fn effects(&self, fx: Vec<Effect>) {
        for e in fx {
            match e {
                Effect::Relayout => self.go(self.sh.borrow().mode, true),
                Effect::OpenPanel { page, clock_tab } => {
                    if let Some(ui) = self.ui.upgrade() { let a = ui.global::<AppState>(); a.set_page(page); a.set_clock_tab(clock_tab); }
                    self.go(2, false);
                }
                Effect::Fast(on) => {
                    if on {
                        let me = self.clone();
                        self.fast.start(TimerMode::Repeated, Duration::from_millis(60), move || {
                            if let Some(ui) = me.ui.upgrade() { ui.global::<AppState>().set_sw_text(me.eng.borrow().stopwatch_text(Wall::now().ms).into()); }
                        });
                    } else { self.fast.stop(); }
                }
            }
        }
    }

    /// Kirim perintah ke engine, segarkan tampilan, jalankan efek.
    fn send(&self, c: Cmd) {
        let Some(ui) = self.ui.upgrade() else { return };
        let w = Wall::now();
        let st = self.ui_state(&ui);
        let mut fx = self.eng.borrow_mut().cmd(c, &w);
        fx.extend(self.eng.borrow_mut().refresh(st, &w));
        self.render();
        self.effects(fx);
    }

    /// Ubah mode: 0 pil, 1 kartu persetujuan, 2 panel penuh. `force` = hitung ulang meski mode sama (mis. dock berubah).
    fn go(&self, target: i32, force: bool) {
        let Some(ui) = self.ui.upgrade() else { return };
        let cfg = self.cfg();
        let sf = ui.window().scale_factor().max(0.5);
        let (cur_mode, cur_abs, gen) = { let mut s = self.sh.borrow_mut(); if s.mode == target && !force { return; } s.gen += 1; (s.mode, s.abs, s.gen) };
        let a = ui.global::<AppState>();
        a.set_dock(cfg.dock.index()); a.set_vertical(cfg.dock.vertical());
        let hw = self.hwnd.get();
        let lg = |v: i32| v as f32 / sf;

        if target == 0 {
            // ---- tutup: animasikan pulau mengecil ke posisi pil, lalu ciutkan jendela ----
            let c = plan(&cfg, 0, sf).win;
            if cur_mode == 0 {
                set_window(&ui, c); set_isl(&ui, true, (0.0, 0.0, lg(c.w), lg(c.h)));
                let mut s = self.sh.borrow_mut(); s.abs = c; return;
            }
            set_isl(&ui, false, (lg(c.x - cur_abs.x), lg(c.y - cur_abs.y), lg(c.w), lg(c.h)));
            self.sh.borrow_mut().mode = 0;
            let me = self.clone();
            Timer::single_shot(Duration::from_millis(ANIM_MS + 30), move || {
                if me.sh.borrow().gen != gen { return; }
                let Some(ui) = me.ui.upgrade() else { return };
                ui.global::<AppState>().set_mode(0);
                set_window(&ui, c);
                set_isl(&ui, true, (0.0, 0.0, c.w as f32 / sf, c.h as f32 / sf));
                me.sh.borrow_mut().abs = c;
                win::api::set_noactivate(me.hwnd.get(), true);
            });
            return;
        }

        // ---- buka / ganti ukuran: jendela langsung ke ukuran tujuan, pulau beranimasi dari rect lama ----
        let p = plan(&cfg, target, sf);
        let from = if cur_mode == 0 && !force { (lg(p.pill_x), lg(p.pill_y), lg(plan(&cfg, 0, sf).win.w), lg(plan(&cfg, 0, sf).win.h)) }
                   else { (lg(cur_abs.x - p.win.x), lg(cur_abs.y - p.win.y), lg(cur_abs.w), lg(cur_abs.h)) };
        set_isl(&ui, true, from);
        a.set_mode(target);
        set_window(&ui, p.win);
        win::api::set_noactivate(hw, target == 1);
        if target == 2 { win::api::focus(hw); }
        { let mut s = self.sh.borrow_mut(); s.mode = target; s.abs = p.win; }
        let me = self.clone();
        let to = (0.0, 0.0, lg(p.win.w), lg(p.win.h));
        Timer::single_shot(Duration::from_millis(40), move || {
            if me.sh.borrow().gen != gen { return; }
            if let Some(ui) = me.ui.upgrade() { set_isl(&ui, false, to); }
        });
    }
}

/* ---------------------------------------------------------------- main ---------------------------------------------------------------- */

fn main() {
    let data = island_core::hookproto::data_dir();
    let _ = std::fs::create_dir_all(&data);
    if !win::api::single_instance() {
        let _ = std::fs::write(data.join("show.signal"), b"1");
        return;
    }
    let cfg = Settings::parse(&std::fs::read_to_string(data.join("settings.json")).unwrap_or_default());
    // Renderer software (default) ~ jauh lebih hemat memori daripada GPU. Bila jendela tampak berlatar solid, pilih "Render GPU".
    if std::env::var_os("SLINT_BACKEND").is_none() {
        std::env::set_var("SLINT_BACKEND", if cfg.renderer == "gpu" { "winit-femtovg" } else { "winit-software" });
    }

    let ui = MainWindow::new().expect("gagal membuat jendela");
    push_settings(&ui, &cfg);

    let mut eng = Engine::new_offline(cfg.clone(), &data);
    eng.start_services();
    let first = { let w = Wall::now(); eng.tick(UiState::default(), &w); eng.view.clone() };

    let sh = Rc::new(RefCell::new(Shell { mode: 0, abs: Rect::default(), gen: 0, alert_seen: 0, inside: 0, outside: 0, lost: 0, tick: 0 }));
    let ctl = Ctl { ui: ui.as_weak(), eng: Rc::new(RefCell::new(eng)), sh: sh.clone(), hwnd: Rc::new(Cell::new(0)), last: Rc::new(RefCell::new(View::default())), fast: Rc::new(Timer::default()) };
    let _ = first;

    // posisi awal (pil)
    ui.global::<AppState>().set_mode(0);
    ctl.render();
    ui.show().expect("gagal menampilkan jendela");
    ctl.go(0, true);
    ctl.hwnd.set(win::api::hwnd(TITLE));
    win::api::set_noactivate(ctl.hwnd.get(), true);
    win::api::topmost(ctl.hwnd.get());
    win::api::tray();
    let want_toggle = Arc::new(AtomicBool::new(false));
    { let f = want_toggle.clone(); win::api::hotkey(move || f.store(true, Ordering::SeqCst)); }

    // ---- callback UI -> engine ----
    let a = ui.global::<AppState>();
    macro_rules! cmd0 { ($on:ident, $cmd:expr) => {{ let c = ctl.clone(); a.$on(move || c.send($cmd)); }} }
    cmd0!(on_pomo_toggle, Cmd::PomoToggle); cmd0!(on_pomo_skip, Cmd::PomoSkip); cmd0!(on_pomo_reset, Cmd::PomoReset);
    cmd0!(on_timer_toggle, Cmd::TimerToggle); cmd0!(on_timer_reset, Cmd::TimerReset);
    cmd0!(on_sw_toggle, Cmd::SwToggle); cmd0!(on_sw_lap, Cmd::SwLap); cmd0!(on_sw_reset, Cmd::SwReset);
    cmd0!(on_adzan_test, Cmd::AdzanTest); cmd0!(on_sfx_test, Cmd::SfxTest);
    cmd0!(on_cal_connect, Cmd::CalConnect); cmd0!(on_cal_disconnect, Cmd::CalDisconnect); cmd0!(on_gh_refresh, Cmd::GhRefresh);
    cmd0!(on_clip_clear, Cmd::ClipClear); cmd0!(on_shelf_add, Cmd::ShelfAdd); cmd0!(on_mascot_clicked, Cmd::MascotClicked);
    { let c = ctl.clone(); a.on_timer_add(move |n| c.send(Cmd::TimerAdd(n as i64))); }
    { let c = ctl.clone(); a.on_alarm_add(move |h, m, d, l| c.send(Cmd::AlarmAdd(h as i64, m as i64, d as i64, l.to_string()))); }
    { let c = ctl.clone(); a.on_alarm_toggle(move |i| c.send(Cmd::AlarmToggle(i as i64))); }
    { let c = ctl.clone(); a.on_alarm_del(move |i| c.send(Cmd::AlarmDel(i as i64))); }
    { let c = ctl.clone(); a.on_open_url(move |u| c.send(Cmd::OpenUrl(u.to_string()))); }
    { let c = ctl.clone(); a.on_clip_copy(move |i| c.send(Cmd::ClipCopy(i.max(0) as usize))); }
    { let c = ctl.clone(); a.on_clip_pin(move |i| c.send(Cmd::ClipPin(i.max(0) as usize))); }
    { let c = ctl.clone(); a.on_clip_del(move |i| c.send(Cmd::ClipDel(i.max(0) as usize))); }
    { let c = ctl.clone(); a.on_shelf_open(move |i| c.send(Cmd::ShelfOpen(i.max(0) as usize))); }
    { let c = ctl.clone(); a.on_shelf_del(move |i| c.send(Cmd::ShelfDel(i.max(0) as usize))); }
    { let c = ctl.clone(); a.on_shelf_copy(move |i| c.send(Cmd::ShelfCopy(i.max(0) as usize))); }
    { let c = ctl.clone(); a.on_search(move |q| c.send(Cmd::Search(q.to_string()))); }
    { let c = ctl.clone(); a.on_search_run(move |i| c.send(Cmd::SearchRun(i.max(0) as usize))); }
    { let c = ctl.clone(); a.on_media_cmd(move |m| c.send(Cmd::Media(m.to_string()))); }
    { let c = ctl.clone(); a.on_preview_face(move |i| c.send(Cmd::PreviewFace(i))); }
    { let c = ctl.clone(); a.on_hooks_install(move |k| c.send(Cmd::HooksInstall(k.to_string()))); }
    { let c = ctl.clone(); a.on_hooks_remove(move |k| c.send(Cmd::HooksRemove(k.to_string()))); }
    { let c = ctl.clone(); a.on_approve(move |id, k| {
        c.send(Cmd::Approve(id.to_string(), k));
        if c.eng.borrow().approvals_pending() == 0 && c.sh.borrow().mode == 1 { c.go(0, false); }
    }); }
    { let c = ctl.clone(); a.on_settings_changed(move || {
        let Some(ui) = c.ui.upgrade() else { return };
        let base = c.cfg();
        let new = pull_settings(&ui, &base);
        c.send(Cmd::Settings(Box::new(new)));
        // kota yang cocok daftar mengubah lat/lon; selaraskan kembali nama kota di UI
        push_city_back(&ui, &c.cfg());
    }); }
    { let c = ctl.clone(); a.on_toggle_island(move || { let m = c.sh.borrow().mode; c.go(if m == 0 || m == 1 { 2 } else { 0 }, false); }); }
    { let c = ctl.clone(); a.on_collapse(move || {
        c.sh.borrow_mut().alert_seen = c.eng.borrow().approvals_pending();
        c.go(0, false);
    }); }
    a.on_page_changed(|_| {});
    a.on_hover(|_| {});
    a.on_quit(|| { let _ = slint::quit_event_loop(); });

    // ---- seret-lepas berkas ke Shelf ----
    let dropped: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(vec![]));
    #[cfg(feature = "dragdrop")]
    {
        use slint::winit_030::{winit, EventResult, WinitWindowAccessor};
        let d = dropped.clone();
        ui.window().on_winit_window_event(move |_w, ev| {
            if let winit::event::WindowEvent::DroppedFile(p) = ev { d.borrow_mut().push(p.to_string_lossy().into_owned()); }
            EventResult::Propagate
        });
    }

    // ---- timer 1 detik: data ----
    let t_data = Timer::default();
    {
        let c = ctl.clone();
        t_data.start(TimerMode::Repeated, Duration::from_secs(1), move || {
            let Some(ui) = c.ui.upgrade() else { return };
            let st = c.ui_state(&ui);
            let fx = c.eng.borrow_mut().tick(st, &Wall::now());
            c.render();
            c.effects(fx);
        });
    }
    ctl.eng.borrow_mut().tick(UiState::default(), &Wall::now());
    ctl.render();

    // ---- timer 100 ms: tombol global, hover, fokus, persetujuan baru ----
    let t_ctl = Timer::default();
    {
        let c = ctl.clone(); let signal = data.join("show.signal");
        t_ctl.start(TimerMode::Repeated, Duration::from_millis(100), move || {
            let Some(ui) = c.ui.upgrade() else { return };
            if c.hwnd.get() == 0 { c.hwnd.set(win::api::hwnd(TITLE)); win::api::set_noactivate(c.hwnd.get(), c.sh.borrow().mode != 2); }
            let cfg = c.cfg();
            let n = { let mut s = c.sh.borrow_mut(); s.tick += 1; s.tick };

            // tombol: hotkey, instance kedua, tray
            let mut toggle = want_toggle.swap(false, Ordering::SeqCst);
            if n % 5 == 0 && signal.exists() { let _ = std::fs::remove_file(&signal); toggle = true; }
            match win::api::TRAY_EVENT.swap(0, Ordering::SeqCst) { 1 => toggle = true, 2 => { let _ = slint::quit_event_loop(); return; } _ => {} }
            let mode = c.sh.borrow().mode;
            if toggle { c.go(if mode == 2 { 0 } else { 2 }, false); }
            if n % 50 == 0 { win::api::topmost(c.hwnd.get()); }

            // berkas yang dilepas
            let files: Vec<String> = std::mem::take(&mut *dropped.borrow_mut());
            if !files.is_empty() { c.send(Cmd::ShelfAddPaths(files)); }

            // persetujuan baru dari hook
            let pending = c.eng.borrow().approvals_pending();
            if c.eng.borrow().hook_dirty() {
                c.eng.borrow_mut().refresh_approvals();
                c.render();
                let seen = c.sh.borrow().alert_seen;
                if pending > seen {
                    c.eng.borrow().notify_sound();
                    c.sh.borrow_mut().alert_seen = pending;
                    if c.sh.borrow().mode == 0 { c.go(1, false); }
                }
            }
            if pending == 0 {
                c.sh.borrow_mut().alert_seen = 0;
                if c.sh.borrow().mode == 1 { c.go(0, false); }
            }

            // hover & kehilangan fokus
            let mode = c.sh.borrow().mode;
            let (wx, wy, ww, wh) = win::api::window_rect(c.hwnd.get());
            let (cx, cy) = win::api::cursor();
            let inside = cx >= wx && cx < wx + ww && cy >= wy && cy < wy + wh;
            {
                let mut s = c.sh.borrow_mut();
                if inside { s.inside += 1; s.outside = 0; } else { s.outside += 1; s.inside = 0; }
                if win::api::foreground() != c.hwnd.get() { s.lost += 1; } else { s.lost = 0; }
            }
            if inside && n % 10 == 0 { c.eng.borrow_mut().cmd(Cmd::Interaction, &Wall::now()); }
            let (inside_n, outside_n, lost_n) = { let s = c.sh.borrow(); (s.inside, s.outside, s.lost) };
            let page = ui.global::<AppState>().get_page();
            if cfg.hover {
                if mode == 0 && inside_n >= 3 { c.go(2, false); }
                if mode == 2 && outside_n >= 7 && page != 8 { c.go(0, false); }
            } else if mode == 2 && lost_n >= 4 && !inside {
                c.sh.borrow_mut().lost = 0;
                c.go(if pending > 0 { 1 } else { 0 }, false);
            }
        });
    }

    ui.run().expect("loop UI berhenti");
}

fn push_city_back(ui: &MainWindow, c: &Settings) {
    let a = ui.global::<AppState>();
    if a.get_s_city().as_str() != c.city && prayer_has_city(&c.city) { a.set_s_city(c.city.as_str().into()); }
}
fn prayer_has_city(n: &str) -> bool { island_core::prayer::CITIES.iter().any(|x| x.0 == n) }
