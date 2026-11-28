import React, { useEffect, useLayoutEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import {
  Bot,
  CalendarDays,
  Github,
  QrCode,
  AlarmClock,
  Music2,
  Settings,
  Timer,
  ChevronUp,
  Upload,
  Cpu,
  BellRing,
  X,
  Send,
  Wifi,
  WifiOff,
  BatteryCharging,
  BatteryLow,
  BatteryFull,
  ArrowUpSquare,
  Sparkles,
  Play,
  Pause,
} from "lucide-react";
import { useIslandStore } from "../store/useIslandStore";
import { isTauriRuntime, tauriApi } from "../lib/tauriBridge";
import { AiSession, GraviMood, PillTab } from "../types/island";
import { GraviCanvas } from "./GraviCanvas";
import { AiLogo } from "./AiLogo";
import { AgentPill } from "./pills/AgentPill";
import { GitHubPill } from "./pills/GitHubPill";
import { CalendarPill } from "./pills/CalendarPill";
import { LocalDropPill } from "./pills/LocalDropPill";
import { MusicPill } from "./pills/MusicPill";
import { PomodoroPill } from "./pills/PomodoroPill";
import { SystemPill } from "./pills/SystemPill";
import { SettingsPill } from "./pills/SettingsPill";

const TABS: Array<{ id: PillTab; label: string; icon: React.ReactNode }> = [
  { id: "agents", label: "AI", icon: <Bot className="w-3.5 h-3.5" /> },
  { id: "github", label: "GitHub", icon: <Github className="w-3.5 h-3.5" /> },
  { id: "calendar", label: "Kalender", icon: <CalendarDays className="w-3.5 h-3.5" /> },
  { id: "drop", label: "Drop", icon: <QrCode className="w-3.5 h-3.5" /> },
  { id: "music", label: "Musik", icon: <Music2 className="w-3.5 h-3.5" /> },
  { id: "system", label: "Sistem", icon: <Cpu className="w-3.5 h-3.5" /> },
  { id: "pomodoro", label: "Fokus", icon: <Timer className="w-3.5 h-3.5" /> },
];

function formatClock(clock24h: boolean, showSeconds: boolean): string {
  const d = new Date();
  let h = d.getHours();
  const m = String(d.getMinutes()).padStart(2, "0");
  const s = String(d.getSeconds()).padStart(2, "0");
  const secPart = showSeconds ? `:${s}` : "";
  if (clock24h) return `${String(h).padStart(2, "0")}:${m}${secPart}`;
  const suffix = h >= 12 ? "PM" : "AM";
  h = h % 12 || 12;
  return `${h}:${m}${secPart} ${suffix}`;
}

function hudIcon(kind: string): React.ReactNode {
  switch (kind) {
    case "capslock":
      return <ArrowUpSquare className="w-3.5 h-3.5 text-sky-400" />;
    case "battery_charging":
      return <BatteryCharging className="w-3.5 h-3.5 text-emerald-400" />;
    case "battery_low":
      return <BatteryLow className="w-3.5 h-3.5 text-rose-400" />;
    case "battery_full":
      return <BatteryFull className="w-3.5 h-3.5 text-emerald-400" />;
    case "wifi_connected":
      return <Wifi className="w-3.5 h-3.5 text-sky-400" />;
    case "wifi_disconnected":
      return <WifiOff className="w-3.5 h-3.5 text-rose-400" />;
    case "habit_alarm":
      return <AlarmClock className="w-3.5 h-3.5 text-rose-400" />;
    default:
      return <Sparkles className="w-3.5 h-3.5 text-emerald-300" />;
  }
}

function accentBadgeClass(accent: string): string {
  switch (accent) {
    case "emerald":
      return "bg-emerald-500/20 text-emerald-300 border-emerald-400/30";
    case "rose":
      return "bg-rose-500/20 text-rose-300 border-rose-400/30";
    case "amber":
      return "bg-amber-500/20 text-amber-300 border-amber-400/30";
    case "violet":
      return "bg-violet-500/20 text-violet-300 border-violet-400/30";
    default:
      return "bg-sky-500/20 text-sky-300 border-sky-400/30";
  }
}

export const IslandShell: React.FC = () => {
  const {
    mode,
    activeTab,
    mood,
    permissions,
    activeReminder,
    dismissCalendarReminder,
    activeHud,
    dismissHud,
    activeNotif,
    notifQueue,
    dismissOsNotification,
    replyToNotification,
    aiSessions,
    pomoMode,
    pomoRunning,
    pomoSecondsLeft,
    togglePomodoro,
    media,
    isDraggingFile,
    settings,
    setMode,
    setActiveTab,
    setMood,
    setDraggingFile,
    addDroppedFiles,
  } = useIslandStore();

  const [clock, setClock] = useState(() =>
    formatClock(settings.clock_24h, settings.show_seconds)
  );
  const [replyText, setReplyText] = useState("");
  const [replyStatus, setReplyStatus] = useState<string | null>(null);
  const [isHoveringPopup, setIsHoveringPopup] = useState(false);

  const shellRef = useRef<HTMLDivElement | null>(null);
  const compactMeasureRef = useRef<HTMLDivElement | null>(null);
  const expandedInnerRef = useRef<HTMLDivElement | null>(null);
  const notifInnerRef = useRef<HTMLDivElement | null>(null);

  const [compactSize, setCompactSize] = useState<{ w: number; h: number }>({ w: 152, h: 42 });
  const [expandedHeight, setExpandedHeight] = useState<number>(340);
  const [notifHeight, setNotifHeight] = useState<number>(92);
  const [renderExpandedDom, setRenderExpandedDom] = useState<boolean>(mode === "expanded");
  const collapseUnmountTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const hoverOpenTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const hoverCloseTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const idleHideTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const hudAutoDismissRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const notifAutoDismissRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const suppressHoverUntilLeaveRef = useRef(false);

  const isAttached = settings.island_style === "attached";
  const opacityAlpha = Math.max(0.75, Math.min(1, (settings.island_opacity ?? 96) / 100));

  const pomoLabel = `${String(Math.floor(pomoSecondsLeft / 60)).padStart(2, "0")}:${String(
    pomoSecondsLeft % 60
  ).padStart(2, "0")}`;
  const pomoTotalSecs =
    (pomoMode === "focus"
      ? Math.max(1, settings.pomo_focus_mins || 25)
      : Math.max(1, settings.pomo_break_mins || 5)) * 60;
  const pomoActiveInCompact = pomoRunning || pomoSecondsLeft < pomoTotalSecs;
  const pomoPct = Math.min(1, Math.max(0, 1 - pomoSecondsLeft / pomoTotalSecs));

  const showNotifPopup = mode !== "expanded" && activeNotif !== null;
  const showHudPopup = mode !== "expanded" && !showNotifPopup && activeHud !== null;

  const clearHoverTimers = () => {
    if (hoverOpenTimerRef.current) {
      clearTimeout(hoverOpenTimerRef.current);
      hoverOpenTimerRef.current = null;
    }
    if (hoverCloseTimerRef.current) {
      clearTimeout(hoverCloseTimerRef.current);
      hoverCloseTimerRef.current = null;
    }
  };

  useEffect(() => {
    return () => {
      clearHoverTimers();
      if (collapseUnmountTimerRef.current) clearTimeout(collapseUnmountTimerRef.current);
      if (idleHideTimerRef.current) clearTimeout(idleHideTimerRef.current);
      if (hudAutoDismissRef.current) clearTimeout(hudAutoDismissRef.current);
      if (notifAutoDismissRef.current) clearTimeout(notifAutoDismissRef.current);
    };
  }, []);

  useEffect(() => {
    if (!settings.open_on_hover) clearHoverTimers();
  }, [settings.open_on_hover]);

  useEffect(() => {
    setClock(formatClock(settings.clock_24h, settings.show_seconds));
    const interval = settings.show_seconds ? 1000 : 5000;
    const id = setInterval(
      () => setClock(formatClock(settings.clock_24h, settings.show_seconds)),
      interval
    );
    return () => clearInterval(id);
  }, [settings.clock_24h, settings.show_seconds]);

  useEffect(() => {
    if (hudAutoDismissRef.current) {
      clearTimeout(hudAutoDismissRef.current);
      hudAutoDismissRef.current = null;
    }
    if (!activeHud || isHoveringPopup) return;
    const dur = Math.max(1800, settings.hud_duration_ms || 3500);
    hudAutoDismissRef.current = setTimeout(() => {
      dismissHud();
    }, dur);
    return () => {
      if (hudAutoDismissRef.current) clearTimeout(hudAutoDismissRef.current);
    };
  }, [activeHud, isHoveringPopup, settings.hud_duration_ms, dismissHud]);

  useEffect(() => {
    if (notifAutoDismissRef.current) {
      clearTimeout(notifAutoDismissRef.current);
      notifAutoDismissRef.current = null;
    }
    if (!activeNotif || isHoveringPopup || replyText.trim().length > 0) return;
    const dur = Math.max(4500, (settings.hud_duration_ms || 3500) + 1800);
    notifAutoDismissRef.current = setTimeout(() => {
      dismissOsNotification();
    }, dur);
    return () => {
      if (notifAutoDismissRef.current) clearTimeout(notifAutoDismissRef.current);
    };
  }, [activeNotif, isHoveringPopup, replyText, settings.hud_duration_ms, dismissOsNotification]);

  useLayoutEffect(() => {
    if (collapseUnmountTimerRef.current) {
      clearTimeout(collapseUnmountTimerRef.current);
      collapseUnmountTimerRef.current = null;
    }
    if (mode === "expanded") {
      setRenderExpandedDom(true);
    } else if (mode === "compact") {
      collapseUnmountTimerRef.current = setTimeout(() => {
        setRenderExpandedDom(false);
      }, 360);
    } else {
      setRenderExpandedDom(false);
    }
  }, [mode]);

  useEffect(() => {
    const el = compactMeasureRef.current;
    if (!el) return;
    const update = () => {
      const rect = el.getBoundingClientRect();
      if (rect.width > 0 && rect.height > 0) {
        const w = Math.max(138, Math.ceil(rect.width) + 2);
        const h = Math.max(40, Math.ceil(rect.height) + 2);
        setCompactSize((prev) => (prev.w === w && prev.h === h ? prev : { w, h }));
      }
    };
    update();
    const ro = new ResizeObserver(update);
    ro.observe(el);
    return () => ro.disconnect();
  }, [clock, pomoRunning, pomoActiveInCompact, pomoLabel, aiSessions.length, settings.max_compact_logos, media.is_playing]);

  useEffect(() => {
    if (!renderExpandedDom) return;
    const el = expandedInnerRef.current;
    if (!el) return;
    const update = () => {
      const h = Math.ceil(el.scrollHeight) + 2;
      if (h > 60) {
        setExpandedHeight((prev) => (Math.abs(prev - h) <= 1 ? prev : Math.min(724, h)));
      }
    };
    update();
    const ro = new ResizeObserver(update);
    ro.observe(el);
    return () => ro.disconnect();
  }, [renderExpandedDom, activeTab, activeReminder]);

  useEffect(() => {
    if (!showNotifPopup) return;
    const el = notifInnerRef.current;
    if (!el) return;
    const update = () => {
      const h = Math.ceil(el.scrollHeight) + 2;
      if (h > 48) {
        setNotifHeight((prev) => (Math.abs(prev - h) <= 1 ? prev : Math.min(220, h)));
      }
    };
    update();
    const ro = new ResizeObserver(update);
    ro.observe(el);
    return () => ro.disconnect();
  }, [showNotifPopup, activeNotif, replyStatus]);

  useEffect(() => {
    if (idleHideTimerRef.current) {
      clearTimeout(idleHideTimerRef.current);
      idleHideTimerRef.current = null;
    }
    if (
      !settings.auto_hide_idle ||
      mode !== "compact" ||
      pomoRunning ||
      permissions.length > 0 ||
      isDraggingFile ||
      activeHud ||
      activeNotif
    ) {
      return;
    }
    idleHideTimerRef.current = setTimeout(() => {
      const st = useIslandStore.getState();
      if (
        st.mode === "compact" &&
        st.settings.auto_hide_idle &&
        !st.pomoRunning &&
        !st.activeHud &&
        !st.activeNotif
      ) {
        st.setMode("hidden");
      }
    }, 15000);
    return () => {
      if (idleHideTimerRef.current) clearTimeout(idleHideTimerRef.current);
    };
  }, [
    settings.auto_hide_idle,
    mode,
    pomoRunning,
    permissions.length,
    isDraggingFile,
    activeHud,
    activeNotif,
  ]);

  // Hitbox sync
  useEffect(() => {
    const el = shellRef.current;
    if (!el) return;

    const pushHitRect = () => {
      if (!isTauriRuntime()) return;
      if (isDraggingFile) {
        void tauriApi.updateHitRect(0, 0, 540, 760);
        return;
      }
      const rect = el.getBoundingClientRect();
      if (rect.width <= 0 || rect.height <= 0) return;
      void tauriApi.updateHitRect(
        Math.floor(rect.left),
        Math.floor(rect.top),
        Math.ceil(rect.right),
        Math.ceil(rect.bottom)
      );
    };

    pushHitRect();
    const ro = new ResizeObserver(pushHitRect);
    ro.observe(el);
    const t1 = setTimeout(pushHitRect, 200);
    const t2 = setTimeout(pushHitRect, 480);

    return () => {
      ro.disconnect();
      clearTimeout(t1);
      clearTimeout(t2);
    };
  }, [
    mode,
    compactSize,
    expandedHeight,
    notifHeight,
    showHudPopup,
    showNotifPopup,
    isDraggingFile,
    settings.island_position,
    settings.island_style,
  ]);

  const runningSessions = aiSessions.filter(
    (s) =>
      (s.status === "active" || s.status === "waiting") &&
      settings.monitored_ais.includes(s.id)
  );
  const uniqueRunning = Array.from(
    new Map(runningSessions.map((s) => [s.id, s])).values()
  ).slice(0, settings.max_compact_logos);

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
    if (names.length > 0) addDroppedFiles(names);
    else setDraggingFile(false);
  };

  const scheduleHoverClose = () => {
    if (!settings.open_on_hover || mode !== "expanded") return;
    if (permissions.length > 0 || isDraggingFile) return;

    if (hoverCloseTimerRef.current) clearTimeout(hoverCloseTimerRef.current);
    const delayMs = Math.max(120, settings.hover_close_delay_ms ?? 450);
    hoverCloseTimerRef.current = setTimeout(() => {
      hoverCloseTimerRef.current = null;
      const st = useIslandStore.getState();
      const activeTag = document.activeElement?.tagName ?? "";
      const isEditing =
        activeTag === "INPUT" || activeTag === "TEXTAREA" || activeTag === "SELECT";
      if (
        st.mode === "expanded" &&
        st.permissions.length === 0 &&
        !st.isDraggingFile &&
        !isEditing
      ) {
        st.setMode("compact");
      }
    }, delayMs);
  };

  useEffect(() => {
    if (!settings.open_on_hover || mode !== "expanded") return;

    const onDocLeave = () => scheduleHoverClose();
    const onDocEnter = () => {
      if (hoverCloseTimerRef.current) {
        clearTimeout(hoverCloseTimerRef.current);
        hoverCloseTimerRef.current = null;
      }
    };
    const onWinBlur = () => {
      const activeTag = document.activeElement?.tagName ?? "";
      if (activeTag !== "SELECT") scheduleHoverClose();
    };

    document.documentElement.addEventListener("mouseleave", onDocLeave);
    document.documentElement.addEventListener("mouseenter", onDocEnter);
    window.addEventListener("blur", onWinBlur);

    let unlistenLeave: (() => void) | undefined;
    let unlistenEnter: (() => void) | undefined;
    if (isTauriRuntime()) {
      void listen("gravi:cursor-leave", onDocLeave).then((u) => {
        unlistenLeave = u;
      });
      void listen("gravi:cursor-enter", onDocEnter).then((u) => {
        unlistenEnter = u;
      });
    }

    return () => {
      document.documentElement.removeEventListener("mouseleave", onDocLeave);
      document.documentElement.removeEventListener("mouseenter", onDocEnter);
      window.removeEventListener("blur", onWinBlur);
      if (unlistenLeave) unlistenLeave();
      if (unlistenEnter) unlistenEnter();
    };
  }, [
    mode,
    settings.open_on_hover,
    settings.hover_close_delay_ms,
    permissions.length,
    isDraggingFile,
  ]);

  const handleMouseEnter = () => {
    setIsHoveringPopup(true);
    if (hoverCloseTimerRef.current) {
      clearTimeout(hoverCloseTimerRef.current);
      hoverCloseTimerRef.current = null;
    }
    if (showHudPopup || showNotifPopup) return;
    if (!settings.open_on_hover || mode !== "compact" || suppressHoverUntilLeaveRef.current) {
      return;
    }
    if (hoverOpenTimerRef.current) clearTimeout(hoverOpenTimerRef.current);
    hoverOpenTimerRef.current = setTimeout(() => {
      hoverOpenTimerRef.current = null;
      setMode("expanded");
    }, 110);
  };

  const handleMouseLeave = () => {
    setIsHoveringPopup(false);
    if (hoverOpenTimerRef.current) {
      clearTimeout(hoverOpenTimerRef.current);
      hoverOpenTimerRef.current = null;
    }
    suppressHoverUntilLeaveRef.current = false;
    scheduleHoverClose();
  };

  const handleSendQuickReply = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!replyText.trim() || !activeNotif) return;
    const msg = replyText;
    setReplyText("");
    const res = await replyToNotification(msg);
    setReplyStatus(res);
    setTimeout(() => {
      setReplyStatus(null);
      dismissOsNotification();
    }, 1100);
  };

  if (mode === "hidden" && !showHudPopup && !showNotifPopup) {
    return (
      <div
        ref={shellRef}
        onMouseEnter={() => setMode("compact")}
        onClick={() => setMode("compact")}
        style={{ backgroundColor: `rgba(8, 8, 12, ${opacityAlpha})` }}
        className={`island-spring-close macos-island ${
          isAttached ? "macos-island-attached rounded-b-full" : "rounded-full"
        } w-28 h-2.5 cursor-pointer hover:h-3.5 flex items-center justify-center`}
        title="Arahkan kursor untuk memunculkan Gravi"
      >
        <div className="w-8 h-1 rounded-full bg-white/30" />
      </div>
    );
  }

  const isExpanded = mode === "expanded";
  const targetWidth = isExpanded
    ? 500
    : showNotifPopup
    ? 376
    : showHudPopup
    ? 330
    : compactSize.w;
  const targetHeight = isExpanded
    ? expandedHeight
    : showNotifPopup
    ? notifHeight
    : showHudPopup
    ? 58
    : compactSize.h;
  const targetRadius = isExpanded
    ? isAttached
      ? "0px 0px 26px 26px"
      : "26px"
    : showNotifPopup || showHudPopup
    ? isAttached
      ? "0px 0px 24px 24px"
      : "24px"
    : isAttached
    ? "0px 0px 22px 22px"
    : "22px";

  const validProvider = (p?: string | null): AiSession["provider"] | null => {
    if (
      p === "claude" ||
      p === "gemini" ||
      p === "chatgpt" ||
      p === "antigravity" ||
      p === "copilot" ||
      p === "hermes" ||
      p === "openbot"
    ) {
      return p;
    }
    return null;
  };

  return (
    <div
      ref={shellRef}
      style={{
        width: targetWidth,
        height: targetHeight,
        borderRadius: targetRadius,
        backgroundColor: `rgba(8, 8, 12, ${opacityAlpha})`,
      }}
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
      className={`${
        isExpanded || showHudPopup || showNotifPopup
          ? "island-spring-open"
          : "island-spring-close"
      } macos-island ${
        isAttached ? "macos-island-attached" : ""
      } relative overflow-hidden text-zinc-100 ${
        !isExpanded && !showNotifPopup ? "cursor-pointer hover:border-white/25" : ""
      }`}
      onClick={() => {
        if (!isExpanded && !showHudPopup && !showNotifPopup) {
          setMode("expanded");
        }
      }}
    >
      {/* Drop overlay */}
      {isDraggingFile && (
        <div
          className={`absolute inset-0 z-30 ${
            isAttached ? "rounded-b-[26px]" : "rounded-[26px]"
          } bg-blue-500/15 backdrop-blur-md border-2 border-dashed border-blue-400 flex flex-col items-center justify-center gap-2 pointer-events-none`}
        >
          <GraviCanvas size={56} mood="drop" />
          <div className="text-xs font-semibold text-blue-100 flex items-center gap-1.5">
            <Upload className="w-3.5 h-3.5" /> Lepaskan file untuk dibagikan via Local Drop
          </div>
        </div>
      )}

      {/* Compact pill */}
      <div
        className={`absolute inset-x-0 top-0 flex items-center justify-center island-layer-fade ${
          isExpanded || showHudPopup || showNotifPopup
            ? "opacity-0 scale-95 pointer-events-none"
            : "opacity-100 scale-100"
        }`}
        style={{ height: compactSize.h }}
      >
        <div
          ref={compactMeasureRef}
          className="px-3.5 py-1.5 flex items-center justify-center gap-2.5 w-max whitespace-nowrap"
        >
          <GraviCanvas size={30} mood={mood} onMoodChange={setMood} />

          <span className="text-[15px] font-bold tracking-tight text-white tabular-nums leading-none">
            {clock}
          </span>

          {pomoActiveInCompact && (
            <button
              type="button"
              onClick={(e) => {
                e.stopPropagation();
                togglePomodoro();
              }}
              className={`flex items-center gap-1.5 px-2 py-0.5 rounded-full border text-[11px] font-mono font-semibold tabular-nums transition ${
                pomoMode === "focus"
                  ? "bg-orange-500/15 border-orange-400/35 text-orange-300 hover:bg-orange-500/25"
                  : "bg-emerald-500/15 border-emerald-400/35 text-emerald-300 hover:bg-emerald-500/25"
              }`}
              title={pomoRunning ? "Klik untuk jeda Pomodoro" : "Klik untuk lanjutkan Pomodoro"}
            >
              <svg className="w-3.5 h-3.5 -rotate-90 shrink-0" viewBox="0 0 16 16">
                <circle cx="8" cy="8" r="6" fill="none" stroke="rgba(255,255,255,0.15)" strokeWidth="2" />
                <circle
                  cx="8"
                  cy="8"
                  r="6"
                  fill="none"
                  stroke={pomoMode === "focus" ? "#FB923C" : "#34D399"}
                  strokeWidth="2"
                  strokeLinecap="round"
                  strokeDasharray={37.7}
                  strokeDashoffset={37.7 * (1 - pomoPct)}
                />
              </svg>
              <span>{pomoLabel}</span>
              {pomoRunning ? (
                <Pause className="w-2.5 h-2.5 fill-current" />
              ) : (
                <Play className="w-2.5 h-2.5 fill-current" />
              )}
            </button>
          )}

          {media.is_playing && (
            <div className="flex items-end gap-[2px] h-3 px-0.5" title={`${media.title} — ${media.artist}`}>
              <span className="w-[2.5px] h-full bg-emerald-400 rounded-full eq-bar" style={{ animationDelay: "0ms" }} />
              <span className="w-[2.5px] h-full bg-sky-400 rounded-full eq-bar" style={{ animationDelay: "180ms" }} />
              <span className="w-[2.5px] h-full bg-emerald-300 rounded-full eq-bar" style={{ animationDelay: "340ms" }} />
            </div>
          )}

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
      </div>

      {/* HUD popup */}
      {showHudPopup && activeHud && (
        <div
          onClick={(e) => {
            e.stopPropagation();
            if (activeHud.app_name) {
              void tauriApi.focusAiWindow(activeHud.app_name);
            }
            dismissHud();
          }}
          className="w-[330px] h-[58px] px-3.5 flex items-center justify-between gap-2.5 tab-reveal"
        >
          <div className="flex items-center gap-2.5 min-w-0">
            <GraviCanvas size={36} mood={(activeHud.mood as GraviMood) || "idle"} onMoodChange={setMood} />
            <div className="min-w-0">
              <div className="text-xs font-semibold text-white truncate flex items-center gap-1.5">
                {hudIcon(activeHud.kind)}
                <span className="truncate">{activeHud.title}</span>
              </div>
              <div className="text-[11px] text-zinc-400 truncate">{activeHud.subtitle}</div>
            </div>
          </div>

          <div className="flex items-center gap-1.5 shrink-0">
            {validProvider(activeHud.provider) && (
              <AiLogo provider={validProvider(activeHud.provider)!} size={18} />
            )}
            {activeHud.badge && (
              <span
                className={`text-[10px] font-semibold px-2 py-0.5 rounded-full border tabular-nums ${accentBadgeClass(
                  activeHud.accent
                )}`}
              >
                {activeHud.badge}
              </span>
            )}
          </div>
        </div>
      )}

      {/* Notification popup */}
      {showNotifPopup && activeNotif && (
        <div ref={notifInnerRef} className="w-[376px] p-3 space-y-2 tab-reveal">
          <div className="flex items-start justify-between gap-2">
            <div className="flex items-start gap-2.5 min-w-0">
              <GraviCanvas size={36} mood="notif" onMoodChange={setMood} />
              <div className="min-w-0 space-y-0.5">
                <div className="flex items-center gap-1.5 flex-wrap">
                  <span className="text-[10px] font-semibold px-1.5 py-0.2 rounded-md bg-sky-500/20 text-sky-300 border border-sky-400/30">
                    {activeNotif.app_name}
                  </span>
                  <span className="text-xs font-semibold text-white truncate max-w-[165px]">
                    {activeNotif.title}
                  </span>
                  {activeNotif.count > 1 && (
                    <span className="text-[10px] font-semibold px-1.5 py-0.2 rounded-full bg-rose-500/20 text-rose-300 border border-rose-400/30 tabular-nums">
                      +{activeNotif.count - 1} pesan
                    </span>
                  )}
                </div>
                {activeNotif.body && (
                  <div className="text-[11px] text-zinc-300 line-clamp-2 leading-snug">
                    {activeNotif.body}
                  </div>
                )}
              </div>
            </div>

            <div className="flex items-center gap-1 shrink-0">
              {notifQueue.length > 0 && (
                <span className="text-[9px] px-1.5 py-0.5 rounded-full bg-white/10 text-zinc-300 tabular-nums">
                  +{notifQueue.length} lain
                </span>
              )}
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  dismissOsNotification();
                }}
                className="p-1 rounded-full hover:bg-white/10 text-zinc-400 hover:text-white transition"
                title="Tutup notifikasi"
              >
                <X className="w-3.5 h-3.5" />
              </button>
            </div>
          </div>

          {activeNotif.is_messaging && (
            <form onSubmit={handleSendQuickReply} className="flex items-center gap-1.5 pt-0.5">
              <input
                value={replyText}
                onChange={(e) => setReplyText(e.target.value)}
                onClick={(e) => e.stopPropagation()}
                placeholder={`Balas cepat ke ${activeNotif.app_name}...`}
                className="flex-1 bg-white/[0.07] border border-white/15 rounded-xl px-2.5 py-1 text-[11px] text-zinc-100 placeholder:text-zinc-500 outline-none focus:border-sky-400/60"
              />
              <button
                type="submit"
                disabled={!replyText.trim()}
                className="p-1.5 rounded-xl bg-sky-500 hover:bg-sky-400 disabled:opacity-40 text-zinc-950 transition shrink-0"
                title="Kirim balasan"
              >
                <Send className="w-3 h-3" />
              </button>
            </form>
          )}

          {replyStatus && (
            <div className="text-[10px] text-emerald-300 px-1">{replyStatus}</div>
          )}
        </div>
      )}

      {/* Expanded panel */}
      {renderExpandedDom && (
        <div
          className={`w-[500px] max-h-[724px] overflow-y-auto gravi-scroll island-layer-fade ${
            isExpanded
              ? "opacity-100 scale-100 translate-y-0"
              : "opacity-0 scale-[0.96] -translate-y-1.5 pointer-events-none"
          }`}
        >
          <div ref={expandedInnerRef} className="p-3 space-y-2.5">
            {/* Header */}
            <div className="flex items-center justify-between gap-2 px-0.5">
              <div className="flex items-center gap-2.5">
                <GraviCanvas size={38} mood={mood} onMoodChange={setMood} />
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
                  title="Pengaturan (Token, API Key, Tampilan)"
                >
                  <Settings className="w-3.5 h-3.5" />
                </button>
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    clearHoverTimers();
                    suppressHoverUntilLeaveRef.current = true;
                    setMode("compact");
                  }}
                  className="px-2.5 py-1 rounded-full bg-white/[0.06] hover:bg-white/[0.12] border border-white/[0.06] text-zinc-400 hover:text-white text-[10px] font-medium flex items-center gap-1 transition"
                  title="Lipat ke mode compact"
                >
                  <ChevronUp className="w-3 h-3" /> Lipat
                </button>
              </div>
            </div>

            {activeReminder && (
              <div className="rounded-xl bg-amber-500/15 border border-amber-400/30 px-2.5 py-1.5 flex items-center justify-between gap-2 text-[11px]">
                <button
                  onClick={() => {
                    setActiveTab("calendar");
                    dismissCalendarReminder();
                  }}
                  className="flex items-center gap-2 min-w-0 text-left"
                >
                  <BellRing className="w-3.5 h-3.5 text-amber-300 shrink-0" />
                  <span className="truncate text-amber-100 font-medium">
                    {activeReminder.minsUntil === 0
                      ? `Dimulai sekarang: ${activeReminder.title}`
                      : `${activeReminder.minsUntil}m lagi (${activeReminder.startTime}): ${activeReminder.title}`}
                  </span>
                </button>
                <button
                  onClick={dismissCalendarReminder}
                  className="p-0.5 rounded hover:bg-white/10 text-amber-300 shrink-0"
                  title="Tutup pengingat"
                >
                  <X className="w-3 h-3" />
                </button>
              </div>
            )}

            {/* Tab bar */}
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

            {/* Tab content */}
            <div key={activeTab} className="tab-reveal">
              {activeTab === "agents" && <AgentPill />}
              {activeTab === "github" && <GitHubPill />}
              {activeTab === "calendar" && <CalendarPill />}
              {activeTab === "drop" && <LocalDropPill />}
              {activeTab === "music" && <MusicPill />}
              {activeTab === "system" && <SystemPill />}
              {activeTab === "pomodoro" && <PomodoroPill />}
              {activeTab === "settings" && <SettingsPill />}
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
