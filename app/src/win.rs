//! FFI Win32 minimal (tanpa crate tambahan). Di non-Windows semua fungsi berupa stub agar `cargo check` tetap jalan.
#![allow(non_snake_case, clippy::upper_case_acronyms)]

#[cfg(windows)]
mod sys {
    use std::ffi::c_void;
    pub type HWND = *mut c_void;
    #[repr(C)] #[derive(Default, Clone, Copy)] pub struct POINT { pub x: i32, pub y: i32 }
    #[repr(C)] #[derive(Default, Clone, Copy)] pub struct RECT { pub left: i32, pub top: i32, pub right: i32, pub bottom: i32 }
    #[repr(C)] #[derive(Default, Clone, Copy)] pub struct SYSTEMTIME { pub y: u16, pub mo: u16, pub dow: u16, pub d: u16, pub h: u16, pub mi: u16, pub s: u16, pub ms: u16 }
    #[repr(C)] #[derive(Default, Clone, Copy)] pub struct FILETIME { pub lo: u32, pub hi: u32 }
    #[repr(C)] pub struct TZI { pub bias: i32, pub sn: [u16; 32], pub sd: SYSTEMTIME, pub sb: i32, pub dn: [u16; 32], pub dd: SYSTEMTIME, pub db: i32 }
    #[repr(C)] pub struct MSG { pub hwnd: HWND, pub message: u32, pub w: usize, pub l: isize, pub time: u32, pub pt: POINT }
    #[repr(C)] pub struct MEMSTAT { pub len: u32, pub load: u32, pub total: u64, pub avail: u64, pub tpf: u64, pub apf: u64, pub tv: u64, pub av: u64, pub aev: u64 }
    #[link(name = "user32")]
    extern "system" {
        pub fn FindWindowW(class: *const u16, title: *const u16) -> HWND;
        pub fn GetWindowLongPtrW(h: HWND, i: i32) -> isize;
        pub fn SetWindowLongPtrW(h: HWND, i: i32, v: isize) -> isize;
        pub fn SetWindowPos(h: HWND, after: HWND, x: i32, y: i32, cx: i32, cy: i32, flags: u32) -> i32;
        pub fn GetCursorPos(p: *mut POINT) -> i32;
        pub fn GetForegroundWindow() -> HWND;
        pub fn SetForegroundWindow(h: HWND) -> i32;
        pub fn SystemParametersInfoW(a: u32, b: u32, c: *mut c_void, d: u32) -> i32;
        pub fn RegisterHotKey(h: HWND, id: i32, mods: u32, vk: u32) -> i32;
        pub fn GetMessageW(m: *mut MSG, h: HWND, a: u32, b: u32) -> i32;
        pub fn GetSystemMetrics(i: i32) -> i32;
        pub fn GetClipboardSequenceNumber() -> u32;
        pub fn RegisterClassW(c: *const WNDCLASSW) -> u16;
        pub fn CreateWindowExW(ex: u32, class: *const u16, title: *const u16, style: u32, x: i32, y: i32, w: i32, h: i32, parent: HWND, menu: *mut c_void, inst: *mut c_void, p: *mut c_void) -> HWND;
        pub fn DefWindowProcW(h: HWND, m: u32, w: usize, l: isize) -> isize;
        pub fn LoadIconW(inst: *mut c_void, name: *const u16) -> *mut c_void;
        pub fn CreatePopupMenu() -> *mut c_void;
        pub fn AppendMenuW(m: *mut c_void, flags: u32, id: usize, text: *const u16) -> i32;
        pub fn TrackPopupMenu(m: *mut c_void, flags: u32, x: i32, y: i32, r: i32, h: HWND, rect: *const RECT) -> i32;
        pub fn DestroyMenu(m: *mut c_void) -> i32;
        pub fn TranslateMessage(m: *const MSG) -> i32;
        pub fn DispatchMessageW(m: *const MSG) -> isize;
        pub fn RegisterWindowMessageW(s: *const u16) -> u32;
        pub fn OpenClipboard(h: HWND) -> i32;
        pub fn CloseClipboard() -> i32;
        pub fn EmptyClipboard() -> i32;
        pub fn GetClipboardData(f: u32) -> *mut c_void;
        pub fn SetClipboardData(f: u32, h: *mut c_void) -> *mut c_void;
        pub fn keybd_event(bVk: u8, bScan: u8, dwFlags: u32, dwExtraInfo: usize);
    }
    #[repr(C)] pub struct WNDCLASSW { pub style: u32, pub proc_: unsafe extern "system" fn(HWND, u32, usize, isize) -> isize, pub cls_extra: i32, pub wnd_extra: i32, pub inst: *mut c_void, pub icon: *mut c_void, pub cursor: *mut c_void, pub bg: *mut c_void, pub menu: *const u16, pub class: *const u16 }
    #[repr(C)] pub struct NID { pub size: u32, pub hwnd: HWND, pub id: u32, pub flags: u32, pub cb: u32, pub icon: *mut c_void, pub tip: [u16; 128], pub state: u32, pub smask: u32, pub info: [u16; 256], pub ver: u32, pub title: [u16; 64], pub iflags: u32, pub guid: [u8; 16], pub bicon: *mut c_void }
    #[repr(C)] pub struct PE32 { pub size: u32, pub usage: u32, pub pid: u32, pub heap: usize, pub module: u32, pub threads: u32, pub parent: u32, pub pri: i32, pub flags: u32, pub exe: [u16; 260] }
    #[link(name = "shell32")]
    extern "system" {
        pub fn Shell_NotifyIconW(msg: u32, d: *mut NID) -> i32;
        pub fn DragQueryFileW(h: *mut c_void, i: u32, buf: *mut u16, cch: u32) -> u32;
        pub fn ShellExecuteW(h: HWND, op: *const u16, file: *const u16, p: *const u16, d: *const u16, show: i32) -> isize;
    }
    #[link(name = "winmm")]
    extern "system" { pub fn PlaySoundW(name: *const u16, m: *mut c_void, flags: u32) -> i32; }
    #[link(name = "kernel32")]
    extern "system" {
        pub fn CreateMutexW(a: *mut c_void, own: i32, name: *const u16) -> *mut c_void;
        pub fn GetTickCount64() -> u64;
        pub fn GetLastError() -> u32;
        pub fn GetLocalTime(t: *mut SYSTEMTIME);
        pub fn GetTimeZoneInformation(t: *mut TZI) -> u32;
        pub fn GetSystemTimes(i: *mut FILETIME, k: *mut FILETIME, u: *mut FILETIME) -> i32;
        pub fn GlobalMemoryStatusEx(m: *mut MEMSTAT) -> i32;
        pub fn GlobalLock(h: *mut c_void) -> *mut c_void;
        pub fn GlobalUnlock(h: *mut c_void) -> i32;
        pub fn GlobalAlloc(f: u32, n: usize) -> *mut c_void;
        pub fn GetModuleHandleW(n: *const u16) -> *mut c_void;
        pub fn CreateToolhelp32Snapshot(f: u32, pid: u32) -> *mut c_void;
        pub fn Process32FirstW(s: *mut c_void, e: *mut PE32) -> i32;
        pub fn Process32NextW(s: *mut c_void, e: *mut PE32) -> i32;
        pub fn OpenProcess(a: u32, inh: i32, pid: u32) -> *mut c_void;
        pub fn GetProcessTimes(h: *mut c_void, c: *mut FILETIME, e: *mut FILETIME, k: *mut FILETIME, u: *mut FILETIME) -> i32;
        pub fn CloseHandle(h: *mut c_void) -> i32;
    }
    pub fn wide(s: &str) -> Vec<u16> { s.encode_utf16().chain(Some(0)).collect() }
}

pub struct Now { pub y: i64, pub mo: i64, pub d: i64, pub h: i64, pub mi: i64, pub s: i64, pub dow: i64 }

#[cfg(windows)]
pub mod api {
    use super::{sys::*, Now};
    pub fn hwnd(title: &str) -> usize { unsafe { FindWindowW(std::ptr::null(), wide(title).as_ptr()) as usize } }
    /// Aktifkan/matikan click-through (WS_EX_TRANSPARENT) + sembunyikan dari Alt+Tab (TOOLWINDOW).
    pub fn set_passthrough(h: usize, on: bool) {
        if h == 0 { return; }
        unsafe {
            let hw = h as HWND;
            let cur = GetWindowLongPtrW(hw, -20);
            let mut n = cur | 0x80 | 0x80000; // TOOLWINDOW | LAYERED
            if on { n |= 0x20 } else { n &= !0x20 }
            if n != cur { SetWindowLongPtrW(hw, -20, n); }
        }
    }
    pub fn topmost(h: usize) { if h != 0 { unsafe { SetWindowPos(h as HWND, -1isize as HWND, 0, 0, 0, 0, 0x0001 | 0x0002 | 0x0010 | 0x0040); } } }
    pub fn cursor() -> (i32, i32) { let mut p = POINT::default(); unsafe { GetCursorPos(&mut p); } (p.x, p.y) }
    pub fn foreground() -> usize { unsafe { GetForegroundWindow() as usize } }
    pub fn focus(h: usize) { if h != 0 { unsafe { SetForegroundWindow(h as HWND); } } }
    /// Area kerja (di luar taskbar) dalam piksel fisik: (x, y, w, h).
    pub fn work_area() -> (i32, i32, i32, i32) {
        let mut r = RECT::default();
        unsafe {
            if SystemParametersInfoW(0x0030, 0, &mut r as *mut _ as *mut _, 0) == 0 { return (0, 0, GetSystemMetrics(0), GetSystemMetrics(1)); }
        }
        (r.left, r.top, r.right - r.left, r.bottom - r.top)
    }
    /// Return false jika sudah ada instance lain.
    pub fn single_instance() -> bool {
        unsafe { let _ = CreateMutexW(std::ptr::null_mut(), 0, wide("Local\\AntigravityIslandRs").as_ptr()); GetLastError() != 183 }
    }
    /// Thread hotkey Ctrl+Alt+A; memanggil `f` setiap ditekan.
    pub fn hotkey(f: impl Fn() + Send + 'static) {
        std::thread::spawn(move || unsafe {
            if RegisterHotKey(std::ptr::null_mut(), 1, 0x0002 | 0x0001 | 0x4000, 0x41) == 0 { return; }
            let mut m: MSG = std::mem::zeroed();
            while GetMessageW(&mut m, std::ptr::null_mut(), 0, 0) > 0 { if m.message == 0x0312 { f(); } }
        });
    }
    pub fn now() -> Now {
        let mut t = SYSTEMTIME::default(); unsafe { GetLocalTime(&mut t); }
        Now { y: t.y as i64, mo: t.mo as i64, d: t.d as i64, h: t.h as i64, mi: t.mi as i64, s: t.s as i64, dow: t.dow as i64 }
    }
    pub fn tz_hours() -> f64 {
        unsafe {
            let mut z: TZI = std::mem::zeroed();
            let r = GetTimeZoneInformation(&mut z);
            let extra = if r == 2 { z.db } else { z.sb };
            -((z.bias + extra) as f64) / 60.0
        }
    }
    fn ft(f: FILETIME) -> u64 { ((f.hi as u64) << 32) | f.lo as u64 }
    /// (idle, total) kumulatif.
    pub fn cpu_times() -> (u64, u64) {
        let (mut i, mut k, mut u) = (FILETIME::default(), FILETIME::default(), FILETIME::default());
        unsafe { GetSystemTimes(&mut i, &mut k, &mut u); }
        (ft(i), ft(k) + ft(u)) // kernel sudah termasuk idle
    }
    pub fn shell_open(target: &str) { unsafe { ShellExecuteW(std::ptr::null_mut(), wide("open").as_ptr(), wide(target).as_ptr(), std::ptr::null(), std::ptr::null(), 1); } }
    pub fn play_wav(path: &str) {
        unsafe { if path.is_empty() { PlaySoundW(wide("SystemExclamation").as_ptr(), std::ptr::null_mut(), 0x10000 | 1); } else { PlaySoundW(wide(path).as_ptr(), std::ptr::null_mut(), 0x20000 | 1); } }
    }
    pub fn uptime_str() -> String {
        let s = unsafe { GetTickCount64() / 1000 };
        let h = s / 3600;
        let m = (s % 3600) / 60;
        if h > 0 { format!("{}j {}m", h, m) } else { format!("{}m", m) }
    }
    /// Tekan tombol 'y' lalu Enter pada jendela aktif untuk menyetujui izin aksi AI.
    pub fn send_action_confirm() {
        unsafe {
            // 'y' keydown & keyup
            keybd_event(0x59, 0, 0, 0);
            keybd_event(0x59, 0, 2, 0);
            // Enter keydown & keyup
            keybd_event(0x0D, 0, 0, 0);
            keybd_event(0x0D, 0, 2, 0);
        }
    }
    pub fn shell_open_args(file: &str, args: &str) { unsafe { ShellExecuteW(std::ptr::null_mut(), wide("open").as_ptr(), wide(file).as_ptr(), wide(args).as_ptr(), std::ptr::null(), 1); } }
    pub fn clip_seq() -> u32 { unsafe { GetClipboardSequenceNumber() } }
    /// Berkas yang disalin di Explorer (CF_HDROP).
    pub fn clip_files() -> Vec<String> {
        let mut out = vec![];
        unsafe {
            if OpenClipboard(std::ptr::null_mut()) == 0 { return out; }
            let h = GetClipboardData(15);
            if !h.is_null() {
                let n = DragQueryFileW(h, 0xFFFF_FFFF, std::ptr::null_mut(), 0);
                for i in 0..n.min(30) {
                    let mut buf = [0u16; 520];
                    let len = DragQueryFileW(h, i, buf.as_mut_ptr(), 520) as usize;
                    if len > 0 { out.push(String::from_utf16_lossy(&buf[..len.min(520)])); }
                }
            }
            CloseClipboard();
        }
        out
    }
    pub static TRAY_EVENT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0); // 1 buka/tutup, 2 keluar
    static TASKBAR_MSG: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    unsafe fn nid(h: HWND) -> NID {
        let mut d: NID = std::mem::zeroed();
        d.size = std::mem::size_of::<NID>() as u32; d.hwnd = h; d.id = 1; d.flags = 0x1 | 0x2 | 0x4; d.cb = 0x8001;
        d.icon = LoadIconW(std::ptr::null_mut(), 32512usize as *const u16);
        for (i, c) in "Antigravity Island".encode_utf16().enumerate() { d.tip[i] = c; }
        d
    }
    unsafe extern "system" fn tray_proc(h: HWND, m: u32, w: usize, l: isize) -> isize {
        use std::sync::atomic::Ordering::SeqCst;
        if m == 0x8001 {
            match l as u32 {
                0x202 => TRAY_EVENT.store(1, SeqCst),
                0x205 => {
                    let menu = CreatePopupMenu();
                    AppendMenuW(menu, 0, 1, wide("Buka / tutup").as_ptr());
                    AppendMenuW(menu, 0, 2, wide("Keluar").as_ptr());
                    let mut p = POINT::default(); GetCursorPos(&mut p);
                    SetForegroundWindow(h);
                    let cmd = TrackPopupMenu(menu, 0x0100, p.x, p.y, 0, h, std::ptr::null());
                    DestroyMenu(menu);
                    if cmd > 0 { TRAY_EVENT.store(cmd as u32, SeqCst); }
                }
                _ => {}
            }
            return 0;
        }
        let tb = TASKBAR_MSG.load(SeqCst);
        if tb != 0 && m == tb { let mut d = nid(h); Shell_NotifyIconW(0, &mut d); return 0; } // Explorer restart: pasang ulang ikon
        DefWindowProcW(h, m, w, l)
    }
    /// Ikon tray di thread sendiri (jendela pesan tersembunyi).
    pub fn tray() {
        std::thread::spawn(|| unsafe {
            let class = wide("AIslandTray");
            let inst = GetModuleHandleW(std::ptr::null());
            let wc = WNDCLASSW { style: 0, proc_: tray_proc, cls_extra: 0, wnd_extra: 0, inst, icon: std::ptr::null_mut(), cursor: std::ptr::null_mut(), bg: std::ptr::null_mut(), menu: std::ptr::null(), class: class.as_ptr() };
            RegisterClassW(&wc);
            let h = CreateWindowExW(0, class.as_ptr(), class.as_ptr(), 0, 0, 0, 0, 0, -3isize as HWND, std::ptr::null_mut(), inst, std::ptr::null_mut());
            if h.is_null() { return; }
            TASKBAR_MSG.store(RegisterWindowMessageW(wide("TaskbarCreated").as_ptr()), std::sync::atomic::Ordering::SeqCst);
            let mut d = nid(h); Shell_NotifyIconW(0, &mut d);
            let mut m: MSG = std::mem::zeroed();
            while GetMessageW(&mut m, std::ptr::null_mut(), 0, 0) > 0 { TranslateMessage(&m); DispatchMessageW(&m); }
        });
    }
    pub fn clip_get() -> Option<String> {
        unsafe {
            if OpenClipboard(std::ptr::null_mut()) == 0 { return None; }
            let mut out = None;
            let h = GetClipboardData(13);
            if !h.is_null() {
                let p = GlobalLock(h) as *const u16;
                if !p.is_null() {
                    let mut n = 0; while *p.add(n) != 0 && n < 20000 { n += 1; }
                    out = Some(String::from_utf16_lossy(std::slice::from_raw_parts(p, n)));
                    GlobalUnlock(h);
                }
            }
            CloseClipboard();
            out
        }
    }
    pub fn clip_set(text: &str) {
        unsafe {
            let w = wide(text);
            let h = GlobalAlloc(0x0002, w.len() * 2);
            if h.is_null() { return; }
            let p = GlobalLock(h) as *mut u16;
            if p.is_null() { return; }
            std::ptr::copy_nonoverlapping(w.as_ptr(), p, w.len());
            GlobalUnlock(h);
            if OpenClipboard(std::ptr::null_mut()) != 0 { EmptyClipboard(); SetClipboardData(13, h); CloseClipboard(); }
        }
    }
    /// Total CPU time (satuan 100 ns) per nama exe (huruf kecil) yang diminta; indeks sama dengan `names`.
    pub fn proc_cpu(names: &[&str]) -> Vec<u64> {
        let mut out = vec![0u64; names.len()];
        unsafe {
            let snap = CreateToolhelp32Snapshot(0x2, 0);
            if snap.is_null() || snap as isize == -1 { return out; }
            let mut e: PE32 = std::mem::zeroed(); e.size = std::mem::size_of::<PE32>() as u32;
            let mut ok = Process32FirstW(snap, &mut e);
            while ok != 0 {
                let n = e.exe.iter().position(|&c| c == 0).unwrap_or(260);
                let name = String::from_utf16_lossy(&e.exe[..n]).to_lowercase();
                if let Some(i) = names.iter().position(|x| *x == name) {
                    let h = OpenProcess(0x1000, 0, e.pid);
                    if !h.is_null() {
                        let (mut c, mut x, mut k, mut u) = (FILETIME::default(), FILETIME::default(), FILETIME::default(), FILETIME::default());
                        if GetProcessTimes(h, &mut c, &mut x, &mut k, &mut u) != 0 { out[i] += ft(k) + ft(u); }
                        CloseHandle(h);
                    }
                }
                ok = Process32NextW(snap, &mut e);
            }
            CloseHandle(snap);
        }
        out
    }
    /// (persen terpakai 0..1, terpakai GB, total GB)
    pub fn mem() -> (f32, f32, f32) {
        unsafe {
            let mut m: MEMSTAT = std::mem::zeroed(); m.len = std::mem::size_of::<MEMSTAT>() as u32;
            GlobalMemoryStatusEx(&mut m);
            let g = 1024.0 * 1024.0 * 1024.0;
            (m.load as f32 / 100.0, (m.total - m.avail) as f32 / g, m.total as f32 / g)
        }
    }
}

#[cfg(not(windows))]
pub mod api {
    use super::Now;
    pub fn hwnd(_: &str) -> usize { 0 }
    pub fn set_passthrough(_: usize, _: bool) {}
    pub fn topmost(_: usize) {}
    pub fn cursor() -> (i32, i32) { (0, 0) }
    pub fn foreground() -> usize { 0 }
    pub fn focus(_: usize) {}
    pub fn work_area() -> (i32, i32, i32, i32) { (0, 0, 1920, 1080) }
    pub fn single_instance() -> bool { true }
    pub fn hotkey(_: impl Fn() + Send + 'static) {}
    pub static TRAY_EVENT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    pub fn tray() {}
    pub fn clip_files() -> Vec<String> { vec![] }
    pub fn shell_open_args(_: &str, _: &str) {}
    pub fn proc_cpu(n: &[&str]) -> Vec<u64> { vec![0; n.len()] }
    pub fn shell_open(_: &str) {}
    pub fn play_wav(_: &str) {}
    pub fn clip_seq() -> u32 { 0 }
    pub fn clip_get() -> Option<String> { None }
    pub fn clip_set(_: &str) {}
    pub fn now() -> Now { Now { y: 2026, mo: 1, d: 1, h: 12, mi: 0, s: 0, dow: 0 } }
    pub fn tz_hours() -> f64 { 7.0 }
    pub fn cpu_times() -> (u64, u64) { (0, 1) }
    pub fn mem() -> (f32, f32, f32) { (0.0, 0.0, 0.0) }
    pub fn uptime_str() -> String { "1j 30m".into() }
    pub fn send_action_confirm() {}
}
