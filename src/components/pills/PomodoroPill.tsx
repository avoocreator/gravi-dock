import React from "react";
import { Pause, Play, RotateCcw } from "lucide-react";
import { useIslandStore } from "../../store/useIslandStore";

export const PomodoroPill: React.FC = () => {
  const { pomoMode, pomoRunning, pomoSecondsLeft, togglePomodoro, resetPomodoro } =
    useIslandStore();

  const total = pomoMode === "focus" ? 25 * 60 : 5 * 60;
  const progress = 1 - pomoSecondsLeft / total;
  const mins = String(Math.floor(pomoSecondsLeft / 60)).padStart(2, "0");
  const secs = String(pomoSecondsLeft % 60).padStart(2, "0");

  const radius = 18;
  const circumference = 2 * Math.PI * radius;
  const dashOffset = circumference * (1 - progress);

  return (
    <div className="space-y-2.5">
      <div className="macos-card rounded-2xl px-3.5 py-3 flex items-center justify-between gap-3">
        <div>
          <div className="text-[10px] uppercase tracking-wider text-zinc-400">
            {pomoMode === "focus" ? "Deep Work · 25m" : "Istirahat · 5m"}
          </div>
          <div className="text-2xl font-mono font-semibold tracking-tight text-white tabular-nums mt-0.5">
            {mins}:{secs}
          </div>
        </div>

        <div className="flex items-center gap-2">
          <button
            onClick={() => resetPomodoro()}
            className="w-8 h-8 rounded-full bg-white/[0.08] hover:bg-white/15 text-zinc-300 flex items-center justify-center transition"
            title="Reset Timer"
          >
            <RotateCcw className="w-3.5 h-3.5" />
          </button>

          <button
            onClick={togglePomodoro}
            className="relative w-11 h-11 rounded-full flex items-center justify-center group"
            title={pomoRunning ? "Jeda" : "Mulai"}
          >
            <svg className="absolute inset-0 w-11 h-11 -rotate-90">
              <circle
                cx="22"
                cy="22"
                r={radius}
                fill="none"
                stroke="rgba(255,255,255,0.1)"
                strokeWidth="3"
              />
              <circle
                cx="22"
                cy="22"
                r={radius}
                fill="none"
                stroke={pomoMode === "focus" ? "#FB923C" : "#34D399"}
                strokeWidth="3"
                strokeLinecap="round"
                strokeDasharray={circumference}
                strokeDashoffset={dashOffset}
                className="transition-all duration-500"
              />
            </svg>
            <span
              className={`w-8 h-8 rounded-full flex items-center justify-center text-zinc-950 transition ${
                pomoMode === "focus"
                  ? "bg-orange-400 group-hover:bg-orange-300"
                  : "bg-emerald-400 group-hover:bg-emerald-300"
              }`}
            >
              {pomoRunning ? (
                <Pause className="w-3.5 h-3.5 fill-current" />
              ) : (
                <Play className="w-3.5 h-3.5 fill-current ml-0.5" />
              )}
            </span>
          </button>
        </div>
      </div>

      <div className="flex gap-1.5">
        <button
          onClick={() => resetPomodoro("focus")}
          className={`flex-1 py-1.5 rounded-xl text-[11px] font-medium transition ${
            pomoMode === "focus"
              ? "bg-white/15 text-white border border-white/10"
              : "bg-white/[0.04] text-zinc-400 hover:text-white"
          }`}
        >
          Fokus 25 Menit
        </button>
        <button
          onClick={() => resetPomodoro("break")}
          className={`flex-1 py-1.5 rounded-xl text-[11px] font-medium transition ${
            pomoMode === "break"
              ? "bg-white/15 text-white border border-white/10"
              : "bg-white/[0.04] text-zinc-400 hover:text-white"
          }`}
        >
          Istirahat 5 Menit
        </button>
      </div>
    </div>
  );
};
