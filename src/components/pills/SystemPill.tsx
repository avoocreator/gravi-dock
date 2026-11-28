import React, { useEffect, useMemo, useState } from "react";
import {
  Cpu,
  HardDrive,
  Clock,
  Activity,
  Flame,
  BatteryCharging,
  Battery,
  BatteryLow,
  Bluetooth,
  Headphones,
  Mouse,
  Wifi,
  ArrowDown,
  ArrowUp,
  RefreshCw,
  KeyRound,
  Copy,
  Check,
  ExternalLink,
} from "lucide-react";
import { useIslandStore } from "../../store/useIslandStore";
import { tauriApi } from "../../lib/tauriBridge";
import { generateQrMatrix } from "../../lib/qrGenerator";
import { WifiCredentials } from "../../types/island";

function formatUptime(secs: number): string {
  const d = Math.floor(secs / 86400);
  const h = Math.floor((secs % 86400) / 3600);
  const m = Math.floor((secs % 3600) / 60);
  const s = secs % 60;
  if (d > 0) return `${d}h ${h}j ${m}m`;
  return `${String(h).padStart(2, "0")}:${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
}

function formatSpeed(kbps: number): string {
  if (kbps >= 1024) return `${(kbps / 1024).toFixed(1)} MB/s`;
  return `${Math.round(kbps)} KB/s`;
}

const WifiQrSvg: React.FC<{ ssid: string; password: string }> = ({ ssid, password }) => {
  const esc = (v: string) => v.replace(/([\\;,:"])/g, "\\$1");
  const payload = `WIFI:T:WPA;S:${esc(ssid)};P:${esc(password)};;`;
  const mat = useMemo(() => generateQrMatrix(payload), [payload]);
  const n = mat.length;
  const pad = 2;
  const total = n + pad * 2;
  return (
    <svg
      width={92}
      height={92}
      viewBox={`0 0 ${total} ${total}`}
      className="rounded-xl bg-white p-1 shrink-0"
      shapeRendering="crispEdges"
    >
      <rect width={total} height={total} fill="#ffffff" />
      {mat.map((row, r) =>
        row.map((dark, c) =>
          dark ? (
            <rect key={`${r}-${c}`} x={c + pad} y={r + pad} width={1} height={1} fill="#09090b" />
          ) : null
        )
      )}
    </svg>
  );
};

export const SystemPill: React.FC = () => {
  const { systemStats, cpuHistory, hardwareStatus, hardwareLoading, refreshHardwareStatus } =
    useIslandStore();
  const [subTab, setSubTab] = useState<"perf" | "bt" | "net">("perf");
  const [wifiCreds, setWifiCreds] = useState<WifiCredentials | null>(null);
  const [wifiErr, setWifiErr] = useState<string | null>(null);
  const [copiedPass, setCopiedPass] = useState(false);

  useEffect(() => {
    if (subTab === "bt" || subTab === "net") {
      void refreshHardwareStatus();
    }
  }, [subTab, refreshHardwareStatus]);

  const st = systemStats;
  const pts = cpuHistory.length > 1 ? cpuHistory : [20, 28, 24, 32, 30];
  const w = 160;
  const h = 38;
  const stepX = w / Math.max(1, pts.length - 1);
  const linePath = pts
    .map((v, i) => {
      const x = i * stepX;
      const y = h - Math.max(2, Math.min(h - 2, (v / 100) * (h - 4)));
      return `${i === 0 ? "M" : "L"}${x.toFixed(1)},${y.toFixed(1)}`;
    })
    .join(" ");
  const areaPath = `${linePath} L${w},${h} L0,${h} Z`;

  const r = 17;
  const circ = 2 * Math.PI * r;
  const ramOffset = circ * (1 - Math.min(1, st.ram_percent / 100));

  const handleRevealWifi = async () => {
    setWifiErr(null);
    try {
      const creds = await tauriApi.getWifiCredentials();
      if (creds) setWifiCreds(creds);
    } catch (err) {
      setWifiErr(String(err));
    }
  };

  return (
    <div className="space-y-2">
      {/* Sub tabs */}
      <div className="flex items-center gap-1 bg-white/[0.03] p-0.5 rounded-xl border border-white/[0.06]">
        {[
          { id: "perf" as const, label: "Performa" },
          { id: "bt" as const, label: "Baterai & BT" },
          { id: "net" as const, label: "Net Quick" },
        ].map((t) => (
          <button
            key={t.id}
            onClick={() => setSubTab(t.id)}
            className={`flex-1 py-1 rounded-lg text-[11px] font-medium transition ${
              subTab === t.id
                ? "bg-white/[0.12] text-white"
                : "text-zinc-400 hover:text-zinc-200"
            }`}
          >
            {t.label}
          </button>
        ))}
      </div>

      {subTab === "perf" && (
        <div className="space-y-2.5">
          {/* CPU & RAM */}
          <div className="grid grid-cols-2 gap-2">
            <div className="macos-card rounded-2xl p-2.5 flex flex-col justify-between gap-1.5">
              <div className="flex items-center justify-between">
                <span className="text-[10px] font-semibold uppercase tracking-wider text-zinc-400 flex items-center gap-1">
                  <Cpu className="w-3 h-3 text-sky-400" /> CPU
                </span>
                <span className="text-xs font-bold text-white tabular-nums">
                  {Math.round(st.cpu_percent)}%
                </span>
              </div>
              <svg viewBox={`0 0 ${w} ${h}`} className="w-full h-9 overflow-visible">
                <defs>
                  <linearGradient id="cpuArea" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="0%" stopColor="#38BDF8" stopOpacity="0.35" />
                    <stop offset="100%" stopColor="#38BDF8" stopOpacity="0" />
                  </linearGradient>
                </defs>
                <path d={areaPath} fill="url(#cpuArea)" />
                <path d={linePath} fill="none" stroke="#38BDF8" strokeWidth="1.8" strokeLinecap="round" />
              </svg>
              <div className="text-[10px] text-zinc-500 tabular-nums">{st.cpu_cores} Core aktif</div>
            </div>

            <div className="macos-card rounded-2xl p-2.5 flex items-center justify-between gap-2">
              <div className="space-y-1 min-w-0">
                <span className="text-[10px] font-semibold uppercase tracking-wider text-zinc-400 flex items-center gap-1">
                  <HardDrive className="w-3 h-3 text-emerald-400" /> RAM
                </span>
                <div className="text-sm font-bold text-white tabular-nums">
                  {st.ram_used_gb.toFixed(1)}
                  <span className="text-[10px] font-normal text-zinc-400">
                    {" "}
                    / {st.ram_total_gb.toFixed(0)} GB
                  </span>
                </div>
                <div className="text-[10px] text-zinc-500 tabular-nums">
                  Tersedia {Math.max(0, st.ram_total_gb - st.ram_used_gb).toFixed(1)} GB
                </div>
              </div>
              <div className="relative w-11 h-11 shrink-0 flex items-center justify-center">
                <svg className="w-11 h-11 -rotate-90">
                  <circle cx="22" cy="22" r={r} fill="none" stroke="rgba(255,255,255,0.08)" strokeWidth="4" />
                  <circle
                    cx="22"
                    cy="22"
                    r={r}
                    fill="none"
                    stroke={st.ram_percent > 85 ? "#FB7185" : "#34D399"}
                    strokeWidth="4"
                    strokeLinecap="round"
                    strokeDasharray={circ}
                    strokeDashoffset={ramOffset}
                    className="transition-all duration-500"
                  />
                </svg>
                <span className="absolute text-[10px] font-bold text-white tabular-nums">
                  {Math.round(st.ram_percent)}%
                </span>
              </div>
            </div>
          </div>

          {/* GPU & Runtime */}
          <div className="grid grid-cols-3 gap-2">
            <div className="col-span-2 macos-card rounded-2xl p-2.5 space-y-1.5">
              <div className="flex items-center justify-between gap-2">
                <span className="text-[10px] font-semibold uppercase tracking-wider text-zinc-400 flex items-center gap-1 truncate">
                  <Flame className="w-3 h-3 text-orange-400 shrink-0" />
                  <span className="truncate">{st.gpu.name}</span>
                </span>
                <span className="text-xs font-bold text-white tabular-nums shrink-0">
                  {Math.round(st.gpu.usage_percent)}%
                </span>
              </div>
              <div className="h-1.5 w-full rounded-full bg-white/10 overflow-hidden">
                <div
                  className="h-full rounded-full bg-gradient-to-r from-orange-400 to-amber-300 transition-all duration-500"
                  style={{ width: `${Math.min(100, st.gpu.usage_percent)}%` }}
                />
              </div>
              <div className="flex items-center justify-between text-[10px] text-zinc-400 tabular-nums">
                <span>
                  {st.gpu.vram_total_mb > 0
                    ? `VRAM ${(st.gpu.vram_used_mb / 1024).toFixed(1)} / ${(st.gpu.vram_total_mb / 1024).toFixed(0)} GB`
                    : "VRAM N/A"}
                </span>
                {st.gpu.temp_c != null && <span>{st.gpu.temp_c}°C</span>}
              </div>
            </div>

            <div className="macos-card rounded-2xl p-2.5 flex flex-col justify-between">
              <span className="text-[10px] font-semibold uppercase tracking-wider text-zinc-400 flex items-center gap-1">
                <Clock className="w-3 h-3 text-violet-400" /> Runtime
              </span>
              <div className="text-sm font-mono font-bold text-white tabular-nums">
                {formatUptime(st.uptime_secs)}
              </div>
              <div className="text-[10px] text-zinc-500">Uptime OS</div>
            </div>
          </div>

          {/* Top processes */}
          <div className="macos-card rounded-2xl p-2.5 space-y-1.5">
            <div className="flex items-center justify-between text-[10px] font-semibold uppercase tracking-wider text-zinc-400">
              <span className="flex items-center gap-1">
                <Activity className="w-3 h-3" /> Proses Teratas
              </span>
              <span>CPU · RAM</span>
            </div>
            <div className="space-y-1">
              {st.top_processes.slice(0, 4).map((p) => (
                <div
                  key={`${p.pid}-${p.name}`}
                  className="flex items-center justify-between gap-2 rounded-xl hover:bg-white/[0.04] px-2 py-1 text-[11px] transition"
                >
                  <span className="text-zinc-200 truncate">{p.name}</span>
                  <div className="flex items-center gap-2.5 font-mono text-[10px] shrink-0 tabular-nums">
                    <span className="text-sky-300">{p.cpu_percent.toFixed(1)}%</span>
                    <span className="text-zinc-400 min-w-[46px] text-right">{p.memory_mb} MB</span>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}

      {subTab === "bt" && (
        <div className="space-y-2.5">
          {/* Battery */}
          <div className="macos-card rounded-2xl p-3 space-y-2">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                {hardwareStatus.laptop_battery.charging ? (
                  <BatteryCharging className="w-4 h-4 text-emerald-400" />
                ) : hardwareStatus.laptop_battery.percent <= 20 ? (
                  <BatteryLow className="w-4 h-4 text-rose-400" />
                ) : (
                  <Battery className="w-4 h-4 text-sky-400" />
                )}
                <div>
                  <div className="text-xs font-semibold text-white">Baterai Laptop</div>
                  <div className="text-[10px] text-zinc-400">
                    {!hardwareStatus.laptop_battery.has_battery
                      ? "Daya AC / Desktop"
                      : hardwareStatus.laptop_battery.charging
                      ? "Sedang mengisi daya"
                      : "Menggunakan baterai"}
                  </div>
                </div>
              </div>
              <span className="text-base font-bold font-mono text-white tabular-nums">
                {hardwareStatus.laptop_battery.percent}%
              </span>
            </div>
            <div className="h-1.5 w-full rounded-full bg-white/10 overflow-hidden">
              <div
                className={`h-full rounded-full transition-all duration-500 ${
                  hardwareStatus.laptop_battery.charging
                    ? "bg-emerald-400"
                    : hardwareStatus.laptop_battery.percent <= 20
                    ? "bg-rose-400"
                    : "bg-sky-400"
                }`}
                style={{ width: `${hardwareStatus.laptop_battery.percent}%` }}
              />
            </div>
          </div>

          {/* Bluetooth */}
          <div className="macos-card rounded-2xl p-2.5 space-y-2">
            <div className="flex items-center justify-between">
              <span className="text-[10px] font-semibold uppercase tracking-wider text-zinc-400 flex items-center gap-1.5">
                <Bluetooth className="w-3 h-3 text-sky-400" /> Perangkat Bluetooth
              </span>
              <div className="flex items-center gap-1">
                <button
                  onClick={() => void refreshHardwareStatus()}
                  className="p-1 rounded-lg bg-white/[0.05] hover:bg-white/10 text-zinc-400 hover:text-white transition"
                  title="Pindai ulang Bluetooth"
                >
                  <RefreshCw className={`w-3 h-3 ${hardwareLoading ? "animate-spin" : ""}`} />
                </button>
                <button
                  onClick={() => void tauriApi.openBluetoothSettings()}
                  className="px-2 py-0.5 rounded-lg bg-white/[0.05] hover:bg-white/10 text-[10px] text-zinc-300 flex items-center gap-1 transition"
                >
                  Pengaturan BT <ExternalLink className="w-2.5 h-2.5" />
                </button>
              </div>
            </div>

            {hardwareStatus.bluetooth_devices.length === 0 ? (
              <div className="text-[11px] text-zinc-500 px-1 py-1.5">
                Tidak ada perangkat Bluetooth terdeteksi. Notifikasi baterai lemah (≤20%) aktif otomatis saat perangkat tersambung.
              </div>
            ) : (
              <div className="space-y-1 max-h-36 overflow-y-auto gravi-scroll">
                {hardwareStatus.bluetooth_devices.map((dev) => {
                  const isLow = dev.battery_percent != null && dev.battery_percent <= 20;
                  return (
                    <div
                      key={dev.id}
                      className="flex items-center justify-between gap-2 rounded-xl bg-white/[0.03] px-2.5 py-1.5"
                    >
                      <div className="flex items-center gap-2 min-w-0">
                        {dev.category === "audio" ? (
                          <Headphones className="w-3.5 h-3.5 text-sky-300 shrink-0" />
                        ) : dev.category === "input" ? (
                          <Mouse className="w-3.5 h-3.5 text-violet-300 shrink-0" />
                        ) : (
                          <Bluetooth className="w-3.5 h-3.5 text-zinc-400 shrink-0" />
                        )}
                        <div className="min-w-0">
                          <div className="text-xs font-medium text-zinc-100 truncate">{dev.name}</div>
                          <div className="text-[10px] text-zinc-500">
                            {dev.connected ? "Terhubung" : "Terpasang"}
                          </div>
                        </div>
                      </div>
                      {dev.battery_percent != null ? (
                        <span
                          className={`text-[10px] font-mono font-semibold px-2 py-0.5 rounded-full border tabular-nums shrink-0 ${
                            isLow
                              ? "bg-rose-500/20 text-rose-300 border-rose-500/35"
                              : "bg-emerald-500/15 text-emerald-300 border-emerald-400/25"
                          }`}
                        >
                          {dev.battery_percent}%
                        </span>
                      ) : (
                        <span className="text-[10px] text-zinc-500 shrink-0">
                          {dev.connected ? "Aktif" : "—"}
                        </span>
                      )}
                    </div>
                  );
                })}
              </div>
            )}
          </div>
        </div>
      )}

      {subTab === "net" && (
        <div className="space-y-2.5">
          {/* Network */}
          <div className="macos-card rounded-2xl p-3 space-y-2.5">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2 min-w-0">
                <Wifi className="w-4 h-4 text-sky-400 shrink-0" />
                <div className="min-w-0">
                  <div className="text-xs font-semibold text-white truncate">
                    {hardwareStatus.network.ssid ?? "Ethernet / Offline"}
                  </div>
                  <div className="text-[10px] font-mono text-zinc-400">
                    IP {hardwareStatus.network.local_ip}
                  </div>
                </div>
              </div>
              <div className="flex items-center gap-1.5 shrink-0">
                <span className="text-[10px] font-mono px-2 py-0.5 rounded-full bg-white/[0.06] border border-white/10 text-emerald-300 tabular-nums">
                  {hardwareStatus.network.ping_ms != null
                    ? `${hardwareStatus.network.ping_ms} ms`
                    : "Offline"}
                </span>
                <button
                  onClick={() => void refreshHardwareStatus()}
                  className="p-1 rounded-lg bg-white/[0.05] hover:bg-white/10 text-zinc-400 hover:text-white transition"
                  title="Cek ulang kecepatan & ping"
                >
                  <RefreshCw className={`w-3 h-3 ${hardwareLoading ? "animate-spin" : ""}`} />
                </button>
              </div>
            </div>

            <div className="grid grid-cols-2 gap-2">
              <div className="rounded-xl bg-black/35 border border-white/[0.06] px-2.5 py-2 flex items-center justify-between">
                <span className="text-[10px] text-zinc-400 flex items-center gap-1">
                  <ArrowDown className="w-3 h-3 text-emerald-400" /> Unduh
                </span>
                <span className="text-xs font-mono font-bold text-white tabular-nums">
                  {formatSpeed(hardwareStatus.network.down_kbps)}
                </span>
              </div>
              <div className="rounded-xl bg-black/35 border border-white/[0.06] px-2.5 py-2 flex items-center justify-between">
                <span className="text-[10px] text-zinc-400 flex items-center gap-1">
                  <ArrowUp className="w-3 h-3 text-sky-400" /> Unggah
                </span>
                <span className="text-xs font-mono font-bold text-white tabular-nums">
                  {formatSpeed(hardwareStatus.network.up_kbps)}
                </span>
              </div>
            </div>
          </div>

          {/* Wi-Fi QR */}
          <div className="macos-card rounded-2xl p-2.5 space-y-2">
            <div className="flex items-center justify-between">
              <span className="text-[10px] font-semibold uppercase tracking-wider text-zinc-400 flex items-center gap-1.5">
                <KeyRound className="w-3 h-3 text-amber-400" /> Bagikan Wi-Fi (QR &amp; Sandi)
              </span>
              <button
                onClick={() => void handleRevealWifi()}
                className="px-2.5 py-1 rounded-xl bg-sky-500/20 hover:bg-sky-500/30 border border-sky-400/30 text-sky-200 text-[10px] font-medium transition"
              >
                {wifiCreds ? "Muat Ulang" : "Tampilkan Sandi & QR"}
              </button>
            </div>

            {wifiErr && <div className="text-[11px] text-amber-300">{wifiErr}</div>}

            {wifiCreds && (
              <div className="flex items-center gap-3 pt-1">
                <WifiQrSvg ssid={wifiCreds.ssid} password={wifiCreds.password} />
                <div className="flex-1 min-w-0 space-y-1.5">
                  <div className="text-xs font-semibold text-white truncate">{wifiCreds.ssid}</div>
                  <div className="flex items-center gap-1.5">
                    <code className="flex-1 text-[11px] font-mono bg-black/45 border border-white/10 rounded-lg px-2 py-1 text-emerald-300 truncate">
                      {wifiCreds.password}
                    </code>
                    <button
                      onClick={() => {
                        void navigator.clipboard?.writeText(wifiCreds.password);
                        setCopiedPass(true);
                        setTimeout(() => setCopiedPass(false), 1600);
                      }}
                      className="p-1.5 rounded-lg bg-white/[0.06] hover:bg-white/15 text-zinc-300 transition shrink-0"
                      title="Salin password Wi-Fi"
                    >
                      {copiedPass ? (
                        <Check className="w-3 h-3 text-emerald-400" />
                      ) : (
                        <Copy className="w-3 h-3" />
                      )}
                    </button>
                  </div>
                  <div className="text-[10px] text-zinc-500">
                    Scan QR dari HP untuk langsung terhubung ke Wi-Fi ini.
                  </div>
                </div>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
};
