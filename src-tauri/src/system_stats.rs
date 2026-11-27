use serde::{Deserialize, Serialize};
use std::process::Command;
use sysinfo::{ProcessesToUpdate, System};
use tauri::{AppHandle, Emitter};
use tokio::time::{sleep, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub name: String,
    pub usage_percent: f32,
    pub vram_used_mb: u64,
    pub vram_total_mb: u64,
    pub temp_c: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopProcess {
    pub pid: u32,
    pub name: String,
    pub cpu_percent: f32,
    pub memory_mb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStats {
    pub cpu_percent: f32,
    pub cpu_cores: usize,
    pub ram_used_gb: f32,
    pub ram_total_gb: f32,
    pub ram_percent: f32,
    pub uptime_secs: u64,
    pub gpu: GpuInfo,
    pub top_processes: Vec<TopProcess>,
}

fn query_gpu() -> GpuInfo {
    // NVIDIA
    #[cfg(target_os = "windows")]
    let nv = {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        Command::new("nvidia-smi")
            .args([
                "--query-gpu=name,utilization.gpu,memory.used,memory.total,temperature.gpu",
                "--format=csv,noheader,nounits",
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
    };

    #[cfg(not(target_os = "windows"))]
    let nv = Command::new("nvidia-smi")
        .args([
            "--query-gpu=name,utilization.gpu,memory.used,memory.total,temperature.gpu",
            "--format=csv,noheader,nounits",
        ])
        .output();

    if let Ok(out) = nv {
        if out.status.success() {
            let line = String::from_utf8_lossy(&out.stdout)
                .lines()
                .next()
                .unwrap_or("")
                .trim()
                .to_string();
            let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
            if parts.len() >= 5 {
                return GpuInfo {
                    name: parts[0].to_string(),
                    usage_percent: parts[1].parse::<f32>().unwrap_or(0.0),
                    vram_used_mb: parts[2].parse::<u64>().unwrap_or(0),
                    vram_total_mb: parts[3].parse::<u64>().unwrap_or(0),
                    temp_c: parts[4].parse::<u32>().ok(),
                };
            }
        }
    }

    // Linux DRM
    #[cfg(target_os = "linux")]
    {
        for card in ["card0", "card1"] {
            let busy_path = format!("/sys/class/drm/{}/device/gpu_busy_percent", card);
            if let Ok(txt) = std::fs::read_to_string(&busy_path) {
                let usage = txt.trim().parse::<f32>().unwrap_or(0.0);
                return GpuInfo {
                    name: format!("Integrated / DRM ({})", card),
                    usage_percent: usage,
                    vram_used_mb: 0,
                    vram_total_mb: 0,
                    temp_c: None,
                };
            }
        }
    }

    GpuInfo {
        name: "GPU (Driver/Sensor N/A)".to_string(),
        usage_percent: 0.0,
        vram_used_mb: 0,
        vram_total_mb: 0,
        temp_c: None,
    }
}

pub fn collect_stats(sys: &mut System, gpu_cache: &GpuInfo) -> SystemStats {
    sys.refresh_cpu_usage();
    sys.refresh_memory();
    sys.refresh_processes(ProcessesToUpdate::All, true);

    let cpu_percent = sys.global_cpu_usage();
    let cpu_cores = sys.cpus().len();
    let total_mem = sys.total_memory() as f32;
    let used_mem = sys.used_memory() as f32;
    let ram_total_gb = total_mem / (1024.0 * 1024.0 * 1024.0);
    let ram_used_gb = used_mem / (1024.0 * 1024.0 * 1024.0);
    let ram_percent = if total_mem > 0.0 {
        (used_mem / total_mem) * 100.0
    } else {
        0.0
    };

    let mut procs: Vec<TopProcess> = sys
        .processes()
        .iter()
        .map(|(pid, p)| TopProcess {
            pid: pid.as_u32(),
            name: p.name().to_string_lossy().to_string(),
            cpu_percent: p.cpu_usage(),
            memory_mb: p.memory() / (1024 * 1024),
        })
        .collect();

    procs.sort_by(|a, b| b.memory_mb.cmp(&a.memory_mb));
    procs.truncate(5);

    SystemStats {
        cpu_percent,
        cpu_cores,
        ram_used_gb,
        ram_total_gb,
        ram_percent,
        uptime_secs: System::uptime(),
        gpu: gpu_cache.clone(),
        top_processes: procs,
    }
}

pub async fn start_system_watcher(app: AppHandle) {
    let mut sys = System::new_all();
    let mut gpu_cache = query_gpu();
    let mut tick: u32 = 0;

    loop {
        if tick % 3 == 0 {
            gpu_cache = query_gpu();
        }
        tick = tick.wrapping_add(1);

        let stats = collect_stats(&mut sys, &gpu_cache);
        let _ = app.emit("nobi:system-stats", &stats);
        sleep(Duration::from_secs(2)).await;
    }
}

#[tauri::command]
pub fn get_system_stats() -> SystemStats {
    let mut sys = System::new_all();
    let gpu = query_gpu();
    collect_stats(&mut sys, &gpu)
}
