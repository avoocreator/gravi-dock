import React, { useState } from "react";
import {
  Pause,
  Play,
  RotateCcw,
  Plus,
  Check,
  Trash2,
  AlarmClock,
  CheckCircle2,
} from "lucide-react";
import { useIslandStore } from "../../store/useIslandStore";

export const PomodoroPill: React.FC = () => {
  const {
    pomoMode,
    pomoRunning,
    pomoSecondsLeft,
    settings,
    focusHabits,
    togglePomodoro,
    resetPomodoro,
    addFocusHabit,
    toggleFocusHabit,
    removeFocusHabit,
    updateSettings,
  } = useIslandStore();

  const [newTask, setNewTask] = useState("");

  const focusMins = Math.max(1, settings.pomo_focus_mins || 25);
  const breakMins = Math.max(1, settings.pomo_break_mins || 5);
  const total = pomoMode === "focus" ? focusMins * 60 : breakMins * 60;
  const progress = Math.min(1, Math.max(0, 1 - pomoSecondsLeft / total));
  const mins = String(Math.floor(pomoSecondsLeft / 60)).padStart(2, "0");
  const secs = String(pomoSecondsLeft % 60).padStart(2, "0");

  const radius = 18;
  const circumference = 2 * Math.PI * radius;
  const dashOffset = circumference * (1 - progress);

  const doneCount = focusHabits.filter((h) => h.done).length;
  const habitPct = focusHabits.length > 0 ? Math.round((doneCount / focusHabits.length) * 100) : 0;

  const handleAddHabit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!newTask.trim()) return;
    addFocusHabit(newTask.trim());
    setNewTask("");
  };

  return (
    <div className="space-y-2.5">
      {/* Timer */}
      <div className="macos-card rounded-2xl px-3.5 py-2.5 flex items-center justify-between gap-3">
        <div>
          <div className="flex items-center gap-2">
            <span className="text-[10px] uppercase tracking-wider text-zinc-400">
              {pomoMode === "focus" ? `Deep Work · ${focusMins}m` : `Istirahat · ${breakMins}m`}
            </span>
            <button
              onClick={() => resetPomodoro(pomoMode === "focus" ? "break" : "focus")}
              className="text-[9px] px-1.5 py-0.2 rounded bg-white/[0.06] hover:bg-white/15 text-zinc-300 transition"
            >
              Ganti {pomoMode === "focus" ? "Istirahat" : "Fokus"}
            </button>
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

      {/* Focus habit */}
      <div className="macos-card rounded-2xl p-2.5 space-y-2">
        <div className="flex items-center justify-between gap-2">
          <div className="flex items-center gap-1.5">
            <CheckCircle2 className="w-3.5 h-3.5 text-emerald-400" />
            <span className="text-[11px] font-semibold text-white">Focus Habit Hari Ini</span>
            {focusHabits.length > 0 && (
              <span className="text-[10px] font-mono text-zinc-400 tabular-nums">
                ({doneCount}/{focusHabits.length})
              </span>
            )}
          </div>

          <div className="flex items-center gap-1 text-[10px] text-zinc-400" title="Jam batas alarm jika target hari ini belum selesai">
            <AlarmClock className="w-3 h-3 text-rose-400" />
            <span>Batas</span>
            <input
              type="time"
              value={settings.focus_habit_deadline || "21:00"}
              onChange={(e) => void updateSettings({ focus_habit_deadline: e.target.value })}
              className="bg-black/45 border border-white/10 rounded-lg px-1.5 py-0.5 font-mono text-[10px] text-zinc-200 outline-none"
            />
          </div>
        </div>

        {focusHabits.length > 0 && (
          <div className="h-1 w-full rounded-full bg-white/10 overflow-hidden">
            <div
              className="h-full rounded-full bg-gradient-to-r from-orange-400 to-emerald-400 transition-all duration-300"
              style={{ width: `${habitPct}%` }}
            />
          </div>
        )}

        <div className="space-y-1 max-h-32 overflow-y-auto nobi-scroll">
          {focusHabits.length === 0 ? (
            <div className="text-[11px] text-zinc-500 px-1 py-1">
              Tambahkan target fokus hari ini. Nobi akan memberi alarm &amp; ekspresi panik sebelum jam batas kalau masih ada yang belum selesai!
            </div>
          ) : (
            focusHabits.map((item) => (
              <div
                key={item.id}
                className="flex items-center justify-between gap-2 rounded-xl hover:bg-white/[0.04] px-2 py-1 transition"
              >
                <button
                  type="button"
                  onClick={() => toggleFocusHabit(item.id)}
                  className="flex items-center gap-2 min-w-0 flex-1 text-left"
                >
                  <span
                    className={`w-4 h-4 rounded-md border flex items-center justify-center shrink-0 transition ${
                      item.done
                        ? "bg-emerald-500 border-emerald-400 text-zinc-950"
                        : "border-white/20 hover:border-white/40"
                    }`}
                  >
                    {item.done && <Check className="w-2.5 h-2.5 stroke-[3]" />}
                  </span>
                  <span
                    className={`text-xs truncate ${
                      item.done ? "line-through text-zinc-500" : "text-zinc-100"
                    }`}
                  >
                    {item.text}
                  </span>
                </button>
                <button
                  type="button"
                  onClick={() => removeFocusHabit(item.id)}
                  className="p-0.5 rounded hover:bg-white/10 text-zinc-500 hover:text-rose-300 transition shrink-0"
                  title="Hapus target"
                >
                  <Trash2 className="w-3 h-3" />
                </button>
              </div>
            ))
          )}
        </div>

        <form onSubmit={handleAddHabit} className="flex items-center gap-1.5 pt-0.5">
          <input
            value={newTask}
            onChange={(e) => setNewTask(e.target.value)}
            placeholder="Tambah target hari ini..."
            className="flex-1 bg-white/[0.05] border border-white/10 rounded-xl px-2.5 py-1 text-xs text-zinc-100 placeholder:text-zinc-500 outline-none focus:border-emerald-400/50"
          />
          <button
            type="submit"
            disabled={!newTask.trim()}
            className="p-1.5 rounded-xl bg-emerald-500 hover:bg-emerald-400 disabled:opacity-40 text-zinc-950 transition"
            title="Tambah target"
          >
            <Plus className="w-3.5 h-3.5" />
          </button>
        </form>
      </div>
    </div>
  );
};
