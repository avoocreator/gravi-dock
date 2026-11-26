import React, { useEffect, useRef, useState } from "react";
import {
  Bot,
  CalendarDays,
  Github,
  MessageSquare,
  Music2,
  Settings,
  Timer,
  ChevronUp,
  Upload,
  Cpu,
} from "lucide-react";
import { useIslandStore } from "../store/useIslandStore";
import { isTauriRuntime, tauriApi } from "../lib/tauriBridge";
import { PillTab } from "../types/island";
import { NobiCanvas } from "./NobiCanvas";
import { AiLogo } from "./AiLogo";
import { AgentPill } from "./pills/AgentPill";
import { GitHubPill } from "./pills/GitHubPill";
import { CalendarPill } from "./pills/CalendarPill";
import { ChatPill } from "./pills/ChatPill";
import { MusicPill } from "./pills/MusicPill";
import { PomodoroPill } from "./pills/PomodoroPill";
import { SystemPill } from "./pills/SystemPill";
import { SettingsPill } from "./pills/SettingsPill";

const TABS: Array<{ id: PillTab; label: string; icon: React.ReactNode }> = [
  { id: "agents", label: "AI", icon: <Bot className="w-3.5 h-3.5" /> },
  { id: "github", label: "GitHub", icon: <Github className="w-3.5 h-3.5" /> },
  { id: "calendar", label: "Kalender", icon: <CalendarDays className="w-3.5 h-3.5" /> },
  { id: "chat", label: "Chat", icon: <MessageSquare className="w-3.5 h-3.5" /> },
  { id: "music", label: "Musik", icon: <Music2 className="w-3.5 h-3.5" /> },
  { id: "system", label: "Sistem", icon: <Cpu className="w-3.5 h-3.5" /> },
  { id: "pomodoro", label: "Fokus", icon: <Timer className="w-3.5 h-3.5" /> },
];

function formatClock(clock24h: boolean): string {
  const d = new Date();
  let h = d.getHours();
  const m = String(d.getMinutes()).padStart(2, "0");
  if (clock24h) return `${String(h).padStart(2, "0")}:${m}`;
  const suffix = h >= 12 ? "PM" : "AM";
  h = h % 12 || 12;
  return `${h}:${m} ${suffix}`;
}

export const IslandShell: React.FC = () => {
  const {
    mode,
    activeTab,
    mood,
    permissions,
    aiSessions,
    pomoRunning,
    pomoSecondsLeft,
    isDraggingFile,
    settings,
    setMode,
    setActiveTab,
    setMood,
    setDraggingFile,
    addDroppedFiles,
  } = useIslandStore();

  const [clock, setClock] = useState(() => formatClock(settings.clock_24h));
  const shellRef = useRef<HTMLDivElement | null>(null);
  const lastBoundsRef = useRef<{ w: number; h: number }>({ w: 0, h: 0 });

  useEffect(() => {
    setClock(formatClock(settings.clock_24h));
    const id = setInterval(() => setClock(formatClock(settings.clock_24h)), 5000);
    return () => clearInterval(id);
  }, [settings.clock_24h]);

  // Fit Tauri overlay window tightly to the visible island pill so the rest of the screen stays clickable
  useEffect(() => {
    if (!isTauriRuntime()) return;
    const el = shellRef.current;
    if (!el) return;

    let shrinkTimer: ReturnType<typeof setTimeout> | null = null;

    const pushBounds = (force = false) => {
      const rect = el.getBoundingClientRect();
      if (rect.width <= 0 || rect.height <= 0) return;

      const targetW = Math.ceil(rect.width) + 12;
      const targetH = Math.ceil(rect.height) + 12;
      const prev = lastBoundsRef.current;

      if (!force && prev.w === targetW && prev.h === targetH) return;

      const isGrowing = targetW > prev.w || targetH > prev.h;
      const commit = () => {
        lastBoundsRef.current = { w: targetW, h: targetH };
        void tauriApi.syncIslandBounds(targetW, targetH, settings.island_position);
      };

      if (shrinkTimer) {
        clearTimeout(shrinkTimer);
        shrinkTimer = null;
      }

      if (force || prev.w === 0 || isGrowing) {
        commit();
      } else {
        shrinkTimer = setTimeout(commit, 420);
      }
    };

    pushBounds(true);
    const observer = new ResizeObserver(() => pushBounds(false));
    observer.observe(el);

    return () => {
      observer.disconnect();
      if (shrinkTimer) clearTimeout(shrinkTimer);
    };
  }, [mode, settings.island_position]);

  // Deduplicate active/waiting AI sessions by provider for the compact pill badges
  const runningSessions = aiSessions.filter(
    (s) => (s.status === "active" || s.status === "waiting") && settings.monitored_ais.includes(s.id)
  );
  const uniqueRunning = Array.from(
    new Map(runningSessions.map((s) => [s.id, s])).values()
  ).slice(0, settings.max_compact_logos);

  const pomoLabel = `${String(Math.floor(pomoSecondsLeft / 60)).padStart(2, "0")}:${String(
    pomoSecondsLeft % 60
  ).padStart(2, "0")}`;

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    if (!isDraggingFile) setDraggingFile(true);
  };

  const handleDragLeave = (e: React.DragEvent) => {
    e.preventDefault();
    setDraggingFile(false);
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    const names = Array.from(e.dataTransfer.files).map((f) => f.name);
    if (names.length > 0) {
      addDroppedFiles(names);
    } else {
      setDraggingFile(false);
    }
  };

  if (mode === "hidden") {
    return (
      <div
        ref={shellRef}
        onMouseEnter={() => setMode("compact")}
        onClick={() => setMode("compact")}
        className="spring-shell macos-island w-28 h-2.5 rounded-b-full cursor-pointer hover:h-3.5 flex items-center justify-center"
        title="Arahkan kursor untuk memunculkan Nobi"
      >
        <div className="w-8 h-1 rounded-full bg-white/25" />
      </div>
    );
  }

  return (
    <div
      ref={shellRef}
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
      className={`spring-shell macos-island relative text-zinc-100 ${
        mode === "compact"
          ? "w-fit min-w-[138px] rounded-full px-3.5 py-1.5 cursor-pointer hover:border-white/25"
          : "w-[500px] max-h-[736px] overflow-y-auto nobi-scroll rounded-[26px] p-3"
      }`}
      onClick={() => {
        if (mode === "compact") setMode("expanded");
      }}
    >
      {isDraggingFile && (
        <div className="absolute inset-0 z-30 rounded-[26px] bg-blue-500/15 backdrop-blur-md border-2 border-dashed border-blue-400 flex flex-col items-center justify-center gap-2 pointer-events-none">
          <NobiCanvas size={56} mood="drop" />
          <div className="text-xs font-semibold text-blue-100 flex items-center gap-1.5">
            <Upload className="w-3.5 h-3.5" /> Lepaskan file untuk dilampirkan ke Chat
          </div>
        </div>
      )}

      {mode === "compact" ? (
        /* Ultra-clean Compact Pill: [Nobi] 21:28 [Active AI Logos w/ spinning loading border] */
        <div className="flex items-center justify-center gap-2.5">
          <NobiCanvas size={30} mood={mood} onMoodChange={setMood} />

          <span className="text-[15px] font-bold tracking-tight text-white tabular-nums leading-none">
            {pomoRunning ? pomoLabel : clock}
          </span>

          {uniqueRunning.length > 0 && (
            <div className="flex items-center gap-1.5 pl-0.5">
              {uniqueRunning.map((s) => (
                <div key={s.id} title={`${s.name} (${s.status})`}>
                  <AiLogo
                    provider={s.provider}
                    size={18}
                    loading={s.status === "active"}
                    waiting={s.status === "waiting"}
                  />
                </div>
              ))}
            </div>
          )}
        </div>
      ) : (
        <div className="space-y-2.5">
          {/* Minimalist macOS Notch Header */}
          <div className="flex items-center justify-between gap-2 px-0.5">
            <div className="flex items-center gap-2.5">
              <NobiCanvas size={38} mood={mood} onMoodChange={setMood} />
              <div>
                <div className="text-[13px] font-semibold tracking-tight text-white flex items-center gap-2 tabular-nums">
                  <span>{clock}</span>
                  {permissions.length > 0 && (
                    <span className="text-[10px] px-1.5 py-0.2 rounded-full bg-amber-500/20 text-amber-300 border border-amber-400/25">
                      {permissions.length} izin
                    </span>
                  )}
                </div>
                <div className="text-[11px] text-zinc-400 tabular-nums">
                  {runningSessions.length} sesi AI berjalan
                  {pomoRunning ? ` · Fokus ${pomoLabel}` : ""}
                </div>
              </div>
            </div>

            <div className="flex items-center gap-1.5">
              <button
                onClick={() => {
                  if (isTauriRuntime()) {
                    void tauriApi.openSettingsWindow();
                  } else {
                    setActiveTab(activeTab === "settings" ? "agents" : "settings");
                  }
                }}
                className={`p-1.5 rounded-full border transition ${
                  activeTab === "settings"
                    ? "bg-white/20 border-white/25 text-white"
                    : "bg-white/[0.06] hover:bg-white/[0.12] border-white/[0.06] text-zinc-400 hover:text-white"
                }`}
                title="Pengaturan (Token, Mode Demo, API Key)"
              >
                <Settings className="w-3.5 h-3.5" />
              </button>
              <button
                onClick={() => setMode("compact")}
                className="px-2.5 py-1 rounded-full bg-white/[0.06] hover:bg-white/[0.12] border border-white/[0.06] text-zinc-400 hover:text-white text-[10px] font-medium flex items-center gap-1 transition"
                title="Lipat ke mode compact"
              >
                <ChevronUp className="w-3 h-3" /> Lipat
              </button>
            </div>
          </div>

          {/* Sleek macOS Segmented Pill Bar */}
          <div className="flex items-center gap-0.5 bg-white/[0.04] p-1 rounded-2xl border border-white/[0.06]">
            {TABS.map((t) => {
              const active = activeTab === t.id;
              return (
                <button
                  key={t.id}
                  onClick={() => setActiveTab(t.id)}
                  className={`flex-1 min-w-fit flex items-center justify-center gap-1 px-2 py-1.5 rounded-xl text-[11px] font-medium transition-all duration-200 ${
                    active
                      ? "bg-white/[0.14] text-white shadow-sm border border-white/[0.1]"
                      : "text-zinc-400 hover:text-zinc-200 hover:bg-white/[0.04]"
                  }`}
                >
                  {t.icon}
                  <span>{t.label}</span>
                </button>
              );
            })}
          </div>

          {/* Active Pill Content */}
          <div key={activeTab} className="tab-reveal">
            {activeTab === "agents" && <AgentPill />}
            {activeTab === "github" && <GitHubPill />}
            {activeTab === "calendar" && <CalendarPill />}
            {activeTab === "chat" && <ChatPill />}
            {activeTab === "music" && <MusicPill />}
            {activeTab === "system" && <SystemPill />}
            {activeTab === "pomodoro" && <PomodoroPill />}
            {activeTab === "settings" && <SettingsPill />}
          </div>
        </div>
      )}
    </div>
  );
};
