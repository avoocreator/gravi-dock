import React from "react";
import { Cpu, HardDrive, Clock, Activity, Flame } from "lucide-react";
import { useIslandStore } from "../../store/useIslandStore";

function formatUptime(secs: number): string {
  const d = Math.floor(secs / 86400);
  const h = Math.floor((secs % 86400) / 3600);
  const m = Math.floor((secs % 3600) / 60);
  const s = secs % 60;
  if (d > 0) return `${d}h ${h}j ${m}m`;
  return `${String(h).padStart(2, "0")}:${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
}

export const SystemPill: React.FC = () => {
  const { systemStats, cpuHistory } = useIslandStore();
  const st = systemStats;

  // Build SVG sparkline path for CPU history
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

  return (
    <div className="space-y-2.5">
      {/* Row 1: CPU Sparkline + RAM Circular Gauge */}
      <div className="grid grid-cols-2 gap-2">
        {/* CPU Card */}
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

        {/* RAM Card */}
        <div className="macos-card rounded-2xl p-2.5 flex items-center justify-between gap-2">
          <div className="space-y-1 min-w-0">
            <span className="text-[10px] font-semibold uppercase tracking-wider text-zinc-400 flex items-center gap-1">
              <HardDrive className="w-3 h-3 text-emerald-400" /> RAM
            </span>
            <div className="text-sm font-bold text-white tabular-nums">
              {st.ram_used_gb.toFixed(1)}
              <span className="text-[10px] font-normal text-zinc-400"> / {st.ram_total_gb.toFixed(0)} GB</span>
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

      {/* Row 2: GPU + System Runtime */}
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

      {/* Row 3: Mini Task Manager (Top Processes) */}
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
  );
};
