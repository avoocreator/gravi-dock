use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::net::UdpSocket;
use std::time::Instant;
use sysinfo::Networks;
use tauri::{AppHandle, Emitter};
use tokio::time::{sleep, timeout, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaptopBatteryInfo {
    pub has_battery: bool,
    pub percent: u8,
    pub charging: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BluetoothDeviceInfo {
    pub id: String,
    pub name: String,
    pub connected: bool,
    pub battery_percent: Option<u8>,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub ssid: Option<String>,
    pub local_ip: String,
    pub down_kbps: f64,
    pub up_kbps: f64,
    pub ping_ms: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WifiCredentials {
    pub ssid: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareStatus {
    pub laptop_battery: LaptopBatteryInfo,
    pub bluetooth_devices: Vec<BluetoothDeviceInfo>,
    pub network: NetworkInfo,
}

fn read_laptop_battery() -> LaptopBatteryInfo {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
        let mut status = SYSTEM_POWER_STATUS::default();
        if unsafe { GetSystemPowerStatus(&mut status) }.is_ok()
            && status.BatteryLifePercent <= 100
            && status.BatteryFlag != 128
        {
            return LaptopBatteryInfo {
                has_battery: true,
                percent: status.BatteryLifePercent,
                charging: status.ACLineStatus == 1,
            };
        }
        return LaptopBatteryInfo {
            has_battery: false,
            percent: 100,
            charging: true,
        };
    }

    #[cfg(target_os = "linux")]
    {
        for bat in ["BAT0", "BAT1"] {
            let base = std::path::Path::new("/sys/class/power_supply").join(bat);
            if let (Ok(cap), Ok(st)) = (
                std::fs::read_to_string(base.join("capacity")),
                std::fs::read_to_string(base.join("status")),
            ) {
                if let Ok(pct) = cap.trim().parse::<u8>() {
                    return LaptopBatteryInfo {
                        has_battery: true,
                        percent: pct.min(100),
                        charging: st.trim().eq_ignore_ascii_case("Charging")
                            || st.trim().eq_ignore_ascii_case("Full"),
                    };
                }
            }
        }
        return LaptopBatteryInfo {
            has_battery: false,
            percent: 100,
            charging: true,
        };
    }

    #[allow(unreachable_code)]
    LaptopBatteryInfo {
        has_battery: false,
        percent: 100,
        charging: true,
    }
}

fn classify_bt(name: &str) -> String {
    let low = name.to_lowercase();
    if [
        "buds", "pods", "headphone", "headset", "earbud", "speaker", "tws", "wh-", "wf-", "jbl",
        "sony", "bose", "soundcore", "audio",
    ]
    .iter()
    .any(|k| low.contains(k))
    {
        "audio".into()
    } else if ["mouse", "keyboard", "mx ", "keychron", "logi", "razer", "gamepad", "controller"]
        .iter()
        .any(|k| low.contains(k))
    {
        "input".into()
    } else {
        "other".into()
    }
}

fn query_bluetooth_devices_sync() -> Vec<BluetoothDeviceInfo> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let ps = r#"
$ErrorActionPreference = 'SilentlyContinue'
$devs = Get-PnpDevice -Class Bluetooth | Where-Object { $_.FriendlyName -and $_.FriendlyName -notmatch 'Adapter|Radio|Enumerator|Transport|Controller|RFCOMM|Service' }
$batMap = @{}
Get-PnpDevice -Class System | Where-Object { $_.InstanceId -match 'BTHENUM' } | ForEach-Object {
  $p = Get-PnpDeviceProperty -InstanceId $_.InstanceId -KeyName '{104EA319-6EE2-4701-BD47-8DDBF425BBE5} 2'
  if ($p -and $null -ne $p.Data) {
    $batMap[$_.FriendlyName] = [int]$p.Data
  }
}
foreach ($d in $devs) {
  $bat = ""
  $prop = Get-PnpDeviceProperty -InstanceId $d.InstanceId -KeyName '{104EA319-6EE2-4701-BD47-8DDBF425BBE5} 2'
  if ($prop -and $null -ne $prop.Data) { $bat = [int]$prop.Data }
  else {
    foreach ($k in $batMap.Keys) {
      if ($k -like "*$($d.FriendlyName)*" -or $d.FriendlyName -like "*$k*") { $bat = $batMap[$k]; break }
    }
  }
  $conn = if ($d.Status -eq 'OK') { '1' } else { '0' }
  Write-Output "$($d.InstanceId)|||$($d.FriendlyName)|||$conn|||$bat"
}
"#;
        if let Ok(out) = std::process::Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", ps])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
        {
            let text = String::from_utf8_lossy(&out.stdout);
            let mut seen = HashSet::new();
            let mut list = Vec::new();
            for line in text.lines() {
                let parts: Vec<&str> = line.split("|||").collect();
                if parts.len() < 4 {
                    continue;
                }
                let id = parts[0].trim().to_string();
                let name = parts[1].trim().to_string();
                if name.is_empty() || !seen.insert(name.to_lowercase()) {
                    continue;
                }
                let connected = parts[2].trim() == "1";
                let battery_percent = parts[3].trim().parse::<u8>().ok().map(|b| b.min(100));
                list.push(BluetoothDeviceInfo {
                    id,
                    category: classify_bt(&name),
                    name,
                    connected,
                    battery_percent,
                });
            }
            list.sort_by(|a, b| b.connected.cmp(&a.connected).then(a.name.cmp(&b.name)));
            list.truncate(8);
            return list;
        }
        return Vec::new();
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(out) = std::process::Command::new("bluetoothctl")
            .args(["devices", "Connected"])
            .output()
        {
            let text = String::from_utf8_lossy(&out.stdout);
            let mut list = Vec::new();
            for line in text.lines() {
                let parts: Vec<&str> = line.splitn(3, ' ').collect();
                if parts.len() == 3 {
                    let mac = parts[1].to_string();
                    let name = parts[2].trim().to_string();
                    list.push(BluetoothDeviceInfo {
                        id: mac,
                        category: classify_bt(&name),
                        name,
                        connected: true,
                        battery_percent: None,
                    });
                }
            }
            return list;
        }
        return Vec::new();
    }

    #[allow(unreachable_code)]
    Vec::new()
}

fn read_local_ip() -> String {
    if let Ok(sock) = UdpSocket::bind("0.0.0.0:0") {
        if sock.connect("8.8.8.8:80").is_ok() {
            if let Ok(addr) = sock.local_addr() {
                return addr.ip().to_string();
            }
        }
    }
    "127.0.0.1".to_string()
}

fn read_current_ssid() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let out = std::process::Command::new("netsh")
            .args(["wlan", "show", "interfaces"])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .ok()?;
        let text = String::from_utf8_lossy(&out.stdout);
        for line in text.lines() {
            if let Some((k, v)) = line.trim().split_once(':') {
                if k.trim().eq_ignore_ascii_case("ssid") && !v.trim().is_empty() {
                    return Some(v.trim().to_string());
                }
            }
        }
        return None;
    }

    #[cfg(target_os = "linux")]
    {
        let out = std::process::Command::new("iwgetid").arg("-r").output().ok()?;
        let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
        return if s.is_empty() { None } else { Some(s) };
    }

    #[allow(unreachable_code)]
    None
}

async fn measure_tcp_ping_ms() -> Option<u32> {
    let start = Instant::now();
    let res = timeout(
        Duration::from_millis(1200),
        tokio::net::TcpStream::connect("1.1.1.1:443"),
    )
    .await;
    match res {
        Ok(Ok(_)) => Some(start.elapsed().as_millis() as u32),
        _ => None,
    }
}

#[tauri::command]
pub async fn get_hardware_status() -> Result<HardwareStatus, String> {
    let laptop_battery = read_laptop_battery();
    let local_ip = read_local_ip();
    let ping_ms = measure_tcp_ping_ms().await;

    let (bluetooth_devices, ssid, down_kbps, up_kbps) = tokio::task::spawn_blocking(|| {
        let mut nets = Networks::new_with_refreshed_list();
        std::thread::sleep(std::time::Duration::from_millis(350));
        nets.refresh(true);
        let mut rx_bytes: u64 = 0;
        let mut tx_bytes: u64 = 0;
        for (_, data) in &nets {
            rx_bytes = rx_bytes.saturating_add(data.received());
            tx_bytes = tx_bytes.saturating_add(data.transmitted());
        }
        let down = (rx_bytes as f64 / 0.35) / 1024.0;
        let up = (tx_bytes as f64 / 0.35) / 1024.0;
        let bt = query_bluetooth_devices_sync();
        let ssid = read_current_ssid();
        (bt, ssid, down, up)
    })
    .await
    .map_err(|e| e.to_string())?;

    Ok(HardwareStatus {
        laptop_battery,
        bluetooth_devices,
        network: NetworkInfo {
            ssid,
            local_ip,
            down_kbps,
            up_kbps,
            ping_ms,
        },
    })
}

#[tauri::command]
pub async fn get_wifi_credentials() -> Result<WifiCredentials, String> {
    tokio::task::spawn_blocking(|| -> Result<WifiCredentials, String> {
        let ssid = read_current_ssid()
            .ok_or_else(|| "Tidak ada jaringan Wi-Fi yang sedang terhubung.".to_string())?;

        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            let arg = format!("name={}", ssid);
            let out = std::process::Command::new("netsh")
                .args(["wlan", "show", "profile", &arg, "key=clear"])
                .creation_flags(CREATE_NO_WINDOW)
                .output()
                .map_err(|e| e.to_string())?;
            let text = String::from_utf8_lossy(&out.stdout);
            for line in text.lines() {
                if let Some((k, v)) = line.split_once(':') {
                    let key = k.trim().to_lowercase();
                    if key.contains("key content") || key.contains("isi kunci") {
                        return Ok(WifiCredentials {
                            ssid,
                            password: v.trim().to_string(),
                        });
                    }
                }
            }
            return Err("Password Wi-Fi tidak tersimpan atau jaringan terbuka.".into());
        }

        #[allow(unreachable_code)]
        Err("Pembacaan password Wi-Fi hanya didukung di Windows.".into())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn open_bluetooth_settings() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", "ms-settings:bluetooth"])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn();
    }
    Ok(())
}

// Bluetooth watcher
pub async fn start_bt_watcher(app: AppHandle) {
    let mut warned_devices: HashSet<String> = HashSet::new();

    loop {
        sleep(Duration::from_secs(30)).await;
        let cfg = crate::settings::get_settings(app.clone());
        if !cfg.hud_enabled || !cfg.hud_bt_low {
            continue;
        }

        let devices = tokio::task::spawn_blocking(query_bluetooth_devices_sync)
            .await
            .unwrap_or_default();

        for dev in devices {
            if !dev.connected {
                continue;
            }
            if let Some(pct) = dev.battery_percent {
                if pct <= 20 {
                    if warned_devices.insert(dev.name.clone()) {
                        let _ = app.emit(
                            "gravi:hud-event",
                            crate::system_hud::HudEventPayload {
                                kind: "battery_low".into(),
                                title: format!("Baterai {} Lemah", dev.name),
                                subtitle: format!("Sisa {}% · Segera cas perangkat Bluetooth", pct),
                                mood: "low_battery".into(),
                                accent: "rose".into(),
                                badge: Some(format!("{}%", pct)),
                                provider: None,
                                app_name: None,
                            },
                        );
                    }
                } else if pct > 25 {
                    warned_devices.remove(&dev.name);
                }
            }
        }
    }
}
