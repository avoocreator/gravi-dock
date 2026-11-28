use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tauri::{AppHandle, Emitter};
use tokio::time::{sleep, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsNotificationItem {
    pub id: String,
    pub app_name: String,
    pub title: String,
    pub body: String,
    pub is_messaging: bool,
    pub timestamp: i64,
}

const KNOWN_MESSAGING_APPS: &[(&str, &str)] = &[
    ("whatsapp", "WhatsApp"),
    ("telegram", "Telegram"),
    ("discord", "Discord"),
    ("line", "LINE"),
    ("messenger", "Messenger"),
    ("instagram", "Instagram"),
    ("slack", "Slack"),
    ("teams", "Teams"),
    ("signal", "Signal"),
    ("viber", "Viber"),
    ("wechat", "WeChat"),
    ("kakao", "KakaoTalk"),
    ("skype", "Skype"),
    ("element", "Element"),
    ("beeper", "Beeper"),
    ("phone link", "Phone Link"),
    ("messages", "Messages"),
    ("twitter", "X"),
    ("facebook", "Facebook"),
    ("snapchat", "Snapchat"),
];

fn is_excluded_system_app(app_name: &str) -> bool {
    let low = app_name.to_lowercase();
    [
        "snipping",
        "screenshot",
        "screen sketch",
        "windows",
        "explorer",
        "security",
        "defender",
        "settings",
        "system",
        "store",
        "update",
        "onedrive",
        "clock",
        "alarm",
        "spotify",
        "vlc",
        "steam",
        "nvidia",
        "realtek",
        "bluetooth",
        "battery",
        "power",
        "usb",
        "clipboard",
    ]
    .iter()
    .any(|b| low.contains(b))
}

fn is_browser_app(app_name: &str) -> bool {
    let low = app_name.to_lowercase();
    ["chrome", "edge", "firefox", "brave", "opera", "vivaldi", "arc"]
        .iter()
        .any(|b| low.contains(b))
}

fn classify_notification(app_name: &str, title: &str, body: &str) -> (bool, String) {
    if is_excluded_system_app(app_name) {
        return (false, app_name.to_string());
    }

    let app_low = app_name.to_lowercase();
    for (needle, pretty) in KNOWN_MESSAGING_APPS {
        if app_low.contains(needle) {
            return (true, (*pretty).to_string());
        }
    }

    // Browser push
    if is_browser_app(app_name) {
        let combined = format!("{} {}", title, body).to_lowercase();
        for (needle, pretty) in KNOWN_MESSAGING_APPS {
            if combined.contains(needle) {
                return (true, (*pretty).to_string());
            }
        }
        if combined.contains("web.whatsapp.com") {
            return (true, "WhatsApp".to_string());
        }
        if combined.contains("web.telegram.org") {
            return (true, "Telegram".to_string());
        }
        if combined.contains("discord.com") {
            return (true, "Discord".to_string());
        }
        if combined.contains("instagram.com") {
            return (true, "Instagram".to_string());
        }
        if combined.contains("messenger.com") {
            return (true, "Messenger".to_string());
        }
    }

    (false, app_name.to_string())
}

#[cfg(target_os = "windows")]
fn poll_windows_notifications() -> Vec<OsNotificationItem> {
    use windows::UI::Notifications::Management::{
        UserNotificationListener, UserNotificationListenerAccessStatus,
    };
    use windows::UI::Notifications::{KnownNotificationBindings, NotificationKinds};

    let Ok(listener) = UserNotificationListener::Current() else {
        return Vec::new();
    };

    if listener.GetAccessStatus() != Ok(UserNotificationListenerAccessStatus::Allowed) {
        if listener.GetAccessStatus() == Ok(UserNotificationListenerAccessStatus::Unspecified) {
            let _ = listener.RequestAccessAsync().ok().and_then(|op| op.get().ok());
        }
        if listener.GetAccessStatus() != Ok(UserNotificationListenerAccessStatus::Allowed) {
            return Vec::new();
        }
    }

    let Ok(op) = listener.GetNotificationsAsync(NotificationKinds::Toast) else {
        return Vec::new();
    };
    let Ok(list) = op.get() else {
        return Vec::new();
    };

    let Ok(generic_binding_name) = KnownNotificationBindings::ToastGeneric() else {
        return Vec::new();
    };

    let mut out = Vec::new();
    let count = list.Size().unwrap_or(0);
    for i in 0..count {
        let Ok(n) = list.GetAt(i) else { continue };
        let id = n.Id().unwrap_or(i).to_string();
        let raw_app_name = n
            .AppInfo()
            .and_then(|a| a.DisplayInfo())
            .and_then(|d| d.DisplayName())
            .map(|s| s.to_string())
            .unwrap_or_else(|_| "Windows".to_string());

        let mut lines = Vec::new();
        if let Ok(inner) = n.Notification() {
            if let Ok(visual) = inner.Visual() {
                if let Ok(binding) = visual.GetBinding(&generic_binding_name) {
                    if let Ok(elems) = binding.GetTextElements() {
                        let ec = elems.Size().unwrap_or(0);
                        for j in 0..ec {
                            if let Ok(el) = elems.GetAt(j) {
                                if let Ok(txt) = el.Text() {
                                    let s = txt.to_string();
                                    if !s.trim().is_empty() {
                                        lines.push(s);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let title = lines.first().cloned().unwrap_or_else(|| raw_app_name.clone());
        let body = if lines.len() > 1 {
            lines[1..].join(" — ")
        } else {
            String::new()
        };

        let (is_messaging, display_app) = classify_notification(&raw_app_name, &title, &body);

        out.push(OsNotificationItem {
            id,
            app_name: display_app,
            title,
            body,
            is_messaging,
            timestamp: chrono::Utc::now().timestamp_millis(),
        });
    }
    out
}

pub async fn start_notification_watcher(app: AppHandle) {
    let mut seen_ids: HashSet<String> = HashSet::new();
    let mut initialized = false;

    loop {
        sleep(Duration::from_secs(2)).await;

        let cfg = crate::settings::get_settings(app.clone());
        if !cfg.hud_enabled || !cfg.hud_windows_notif {
            continue;
        }

        #[cfg(target_os = "windows")]
        let items = tokio::task::spawn_blocking(poll_windows_notifications)
            .await
            .unwrap_or_default();

        #[cfg(not(target_os = "windows"))]
        let items: Vec<OsNotificationItem> = Vec::new();

        if !initialized {
            for item in &items {
                seen_ids.insert(item.id.clone());
            }
            initialized = true;
            continue;
        }

        for item in items {
            if seen_ids.insert(item.id.clone()) {
                if cfg.notif_messaging_only && !item.is_messaging {
                    continue;
                }
                let _ = app.emit("gravi:os-notification", &item);
            }
        }

        if seen_ids.len() > 300 {
            seen_ids.clear();
        }
    }
}

#[tauri::command]
pub async fn reply_os_notification(
    app_name: String,
    sender_title: Option<String>,
    reply_text: String,
) -> Result<String, String> {
    let text = reply_text.trim().to_string();
    if text.is_empty() {
        return Err("Pesan balasan kosong.".into());
    }

    #[cfg(target_os = "windows")]
    {
        let app_clone = app_name.clone();
        let sender_clone = sender_title.unwrap_or_default();
        let txt_clone = text.clone();
        let outcome = tokio::task::spawn_blocking(move || -> Result<String, String> {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;

            let safe_app = app_clone.replace('\'', "''");
            let safe_sender = sender_clone.replace('\'', "''");
            let safe_txt = txt_clone.replace('\'', "''");

            // Reply automation
            let ps = format!(
                r#"
$ErrorActionPreference = 'SilentlyContinue'
Set-Clipboard -Value '{txt}'
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
Add-Type -TypeDefinition @"
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;
public class ChatWindowFinder {{
    public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);
    [DllImport("user32.dll")] public static extern bool EnumWindows(EnumWindowsProc lpEnumFunc, IntPtr lParam);
    [DllImport("user32.dll")] public static extern int GetWindowText(IntPtr hWnd, StringBuilder lpString, int nMaxCount);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hWnd);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint pid);
    [DllImport("user32.dll")] public static extern bool AttachThreadInput(uint idAttach, uint idAttachTo, bool fAttach);
    [DllImport("kernel32.dll")] public static extern uint GetCurrentThreadId();
    private static readonly string[] Browsers = new[] {{ "chrome", "msedge", "brave", "firefox", "opera", "vivaldi", "arc" }};
    private static readonly string[] Blocked = new[] {{ "winword", "excel", "powerpnt", "code", "notepad", "notepad++", "gravi" }};

    public static string GetTitle(IntPtr hWnd) {{
        var sb = new StringBuilder(256);
        GetWindowText(hWnd, sb, 256);
        return sb.ToString();
    }}

    public static void FocusHwnd(IntPtr hWnd) {{
        if (hWnd == IntPtr.Zero) return;
        uint dummy;
        uint targetThread = GetWindowThreadProcessId(hWnd, out dummy);
        uint curThread = GetCurrentThreadId();
        AttachThreadInput(curThread, targetThread, true);
        ShowWindow(hWnd, 9);
        SetForegroundWindow(hWnd);
        AttachThreadInput(curThread, targetThread, false);
    }}

    public static IntPtr FindDesktopOrPwa(string query) {{
        IntPtr found = IntPtr.Zero;
        string q = query.ToLowerInvariant().Trim();
        if (string.IsNullOrEmpty(q)) return IntPtr.Zero;
        EnumWindows((hWnd, lParam) => {{
            if (!IsWindowVisible(hWnd)) return true;
            uint pid = 0;
            GetWindowThreadProcessId(hWnd, out pid);
            string proc = "";
            try {{ proc = System.Diagnostics.Process.GetProcessById((int)pid).ProcessName.ToLowerInvariant(); }} catch {{}}
            foreach (var b in Blocked) if (proc == b) return true;
            string title = GetTitle(hWnd).ToLowerInvariant();
            if (title.Contains("google docs") || title.Contains("google sheets") || title.Contains("google slides")) return true;
            bool isBrowser = false;
            foreach (var br in Browsers) if (proc == br) isBrowser = true;
            if (!isBrowser && (proc.Contains(q) || title.Contains(q))) {{
                found = hWnd;
                return false;
            }}
            if (isBrowser && title.Contains(q)) {{
                found = hWnd;
                return false;
            }}
            return true;
        }}, IntPtr.Zero);
        return found;
    }}

    public static IntPtr[] ListBrowserWindows() {{
        var list = new List<IntPtr>();
        EnumWindows((hWnd, lParam) => {{
            if (!IsWindowVisible(hWnd)) return true;
            uint pid = 0;
            GetWindowThreadProcessId(hWnd, out pid);
            string proc = "";
            try {{ proc = System.Diagnostics.Process.GetProcessById((int)pid).ProcessName.ToLowerInvariant(); }} catch {{}}
            foreach (var br in Browsers) {{
                if (proc == br) {{ list.Add(hWnd); break; }}
            }}
            return true;
        }}, IntPtr.Zero);
        return list.ToArray();
    }}
}}
"@

$appQuery = '{app}'
$senderQuery = '{sender}'
$targetHwnd = [ChatWindowFinder]::FindDesktopOrPwa($appQuery)

# Tier 2: If not the active browser title or desktop app, scan browser tabs via UI Automation and switch to the matching tab!
if ($targetHwnd -eq [IntPtr]::Zero) {{
    $browsers = [ChatWindowFinder]::ListBrowserWindows()
    $tabCond = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ControlTypeProperty, [System.Windows.Automation.ControlType]::TabItem)
    foreach ($bh in $browsers) {{
        $root = [System.Windows.Automation.AutomationElement]::FromHandle($bh)
        if ($null -eq $root) {{ continue }}
        $tabs = $root.FindAll([System.Windows.Automation.TreeScope]::Descendants, $tabCond)
        foreach ($tab in $tabs) {{
            $tname = ($tab.Current.Name).ToLowerInvariant()
            if ($tname.Contains($appQuery.ToLowerInvariant())) {{
                [ChatWindowFinder]::FocusHwnd($bh)
                Start-Sleep -Milliseconds 180
                $selPat = $null
                if ($tab.TryGetCurrentPattern([System.Windows.Automation.SelectionItemPattern]::Pattern, [ref]$selPat)) {{
                    $selPat.Select()
                    Start-Sleep -Milliseconds 260
                    $targetHwnd = $bh
                    break
                }}
            }}
        }}
        if ($targetHwnd -ne [IntPtr]::Zero) {{ break }}
    }}
}}

if ($targetHwnd -eq [IntPtr]::Zero) {{
    Write-Output "NOT_FOUND"
    exit
}}

[ChatWindowFinder]::FocusHwnd($targetHwnd)
Start-Sleep -Milliseconds 300

# Safety verification: make 100% sure the active window title is NOT Google Docs/Sheets and matches our chat app
$finalTitle = ([ChatWindowFinder]::GetTitle($targetHwnd)).ToLowerInvariant()
if ($finalTitle.Contains("google docs") -or $finalTitle.Contains("google sheets") -or (-not $finalTitle.Contains($appQuery.ToLowerInvariant()))) {{
    Write-Output "UNVERIFIED_WINDOW"
    exit
}}

# Locate the chat message input box (works for WhatsApp Web contenteditable, Discord, Telegram Web, and Desktop apps)
$rootEl = [System.Windows.Automation.AutomationElement]::FromHandle($targetHwnd)
$editCond = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ControlTypeProperty, [System.Windows.Automation.ControlType]::Edit)
$docCond = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ControlTypeProperty, [System.Windows.Automation.ControlType]::Document)
$orCond = New-Object System.Windows.Automation.OrCondition($editCond, $docCond)
$inputs = $rootEl.FindAll([System.Windows.Automation.TreeScope]::Descendants, $orCond)

$bestInput = $null
$bestScore = -1
foreach ($el in $inputs) {{
    if (-not $el.Current.IsKeyboardFocusable -or -not $el.Current.IsEnabled) {{ continue }}
    $name = ($el.Current.Name).ToLowerInvariant()
    if ($name -match 'search|cari|address|alamat|url|filter|omnibox') {{ continue }}
    $rect = $el.Current.BoundingRectangle
    if ($rect.Width -lt 60 -or $rect.Height -lt 14) {{ continue }}
    $score = [int]$rect.Top
    if ($name -match 'message|pesan|ketik|type a message|reply|balas|chat|write') {{ $score += 10000 }}
    if ($score -gt $bestScore) {{
        $bestScore = $score
        $bestInput = $el
    }}
}}

if ($null -ne $bestInput) {{
    $bestInput.SetFocus()
    Start-Sleep -Milliseconds 140
    $ws = New-Object -ComObject WScript.Shell
    $ws.SendKeys('^v')
    Start-Sleep -Milliseconds 120
    $ws.SendKeys('{{ENTER}}')
    Write-Output "SENT"
}} else {{
    Write-Output "FOCUSED_ONLY"
}}
"#,
                txt = safe_txt,
                app = safe_app,
                sender = safe_sender
            );

            let out = std::process::Command::new("powershell")
                .args(["-NoProfile", "-NonInteractive", "-Command", &ps])
                .creation_flags(CREATE_NO_WINDOW)
                .output()
                .map_err(|e| format!("Gagal memproses balasan untuk {}: {}", app_clone, e))?;

            let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if stdout.contains("SENT") {
                Ok(format!("✓ Balasan terkirim ke {}", app_clone))
            } else if stdout.contains("FOCUSED_ONLY") {
                Ok(format!(
                    "{} dibuka — klik chat tujuan lalu Ctrl+V (balasan sudah disalin)",
                    app_clone
                ))
            } else {
                Ok(format!(
                    "Tab/aplikasi {} tidak ditemukan — balasan disalin ke clipboard (Ctrl+V)",
                    app_clone
                ))
            }
        })
        .await
        .map_err(|e| e.to_string())??;

        return Ok(outcome);
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = (sender_title, crate::ai_monitor::focus_ai_window(app_name.clone()));
        Ok(format!("Membuka {} untuk membalas", app_name))
    }
}

#[tauri::command]
pub fn get_notification_access_status() -> String {
    #[cfg(target_os = "windows")]
    {
        use windows::UI::Notifications::Management::{
            UserNotificationListener, UserNotificationListenerAccessStatus,
        };
        let Ok(listener) = UserNotificationListener::Current() else {
            return "unsupported".into();
        };
        match listener.GetAccessStatus() {
            Ok(UserNotificationListenerAccessStatus::Allowed) => "allowed".into(),
            Ok(UserNotificationListenerAccessStatus::Denied) => "denied".into(),
            Ok(UserNotificationListenerAccessStatus::Unspecified) => "unspecified".into(),
            _ => "unsupported".into(),
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        "unsupported".into()
    }
}

#[tauri::command]
pub fn open_notification_settings() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", "ms-settings:notifications"])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn();
    }
    Ok(())
}

#[tauri::command]
pub fn trigger_test_notification(app: AppHandle) {
    let _ = app.emit(
        "gravi:os-notification",
        OsNotificationItem {
            id: format!("test-{}", chrono::Utc::now().timestamp_millis()),
            app_name: "WhatsApp".into(),
            title: "Budi Santoso".into(),
            body: "Halo! Jadi lanjut meeting jam 8 malam ini?".into(),
            is_messaging: true,
            timestamp: chrono::Utc::now().timestamp_millis(),
        },
    );
}
