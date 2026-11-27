import React, { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { register, unregisterAll } from "@tauri-apps/plugin-global-shortcut";
import { IslandShell } from "./components/IslandShell";
import { SettingsWindow } from "./components/SettingsWindow";
import { useIslandStore } from "./store/useIslandStore";
import { isTauriRuntime, tauriApi } from "./lib/tauriBridge";
import { AgentActivityEvent, AiSession, AppSettings, HudEventPayload, OsNotificationItem, PermissionRequest, ReceivedDropItem, SystemStats } from "./types/island";

export default function App() {
  const isSettingsWindow = isTauriRuntime()
    ? getCurrentWindow().label === "settings"
    : typeof window !== "undefined" &&
      new URLSearchParams(window.location.search).get("window") === "settings";

  const {
    setSystemStats,
    setAiSessions,
    setPermissions,
    pushPermission,
    pushAgentEvent,
    showHud,
    pushOsNotification,
    appendChatChunk,
    pushReceivedDropItem,
    setLocalDropClients,
    checkFocusHabitAlarm,
    checkCalendarReminders,
    tickPomodoro,
    tickMediaProgress,
    setDraggingFile,
    addDroppedFiles,
    refreshSecretStatus,
    refreshAiProviders,
    refreshGitHub,
    refreshCalendar,
    refreshMedia,
    loadSettings,
    applySettings,
    settings,
  } = useIslandStore();

  useEffect(() => {
    void loadSettings();
  }, [loadSettings]);

  // Settings sync
  useEffect(() => {
    if (!isTauriRuntime()) return;
    let unlisten: (() => void) | undefined;
    void listen<AppSettings>("nobi:settings-updated", (e) => {
      applySettings(e.payload);
    }).then((u) => {
      unlisten = u;
    });
    return () => {
      if (unlisten) unlisten();
    };
  }, [applySettings]);

  useEffect(() => {
    if (isSettingsWindow) return;
    const id = setInterval(() => {
      checkCalendarReminders();
      checkFocusHabitAlarm();
    }, 30000);
    return () => clearInterval(id);
  }, [checkCalendarReminders, checkFocusHabitAlarm, isSettingsWindow]);

  useEffect(() => {
    if (isSettingsWindow) return;
    const id = setInterval(() => {
      tickPomodoro();
      tickMediaProgress();
    }, 1000);
    return () => clearInterval(id);
  }, [tickPomodoro, tickMediaProgress, isSettingsWindow]);

  // Global shortcuts
  useEffect(() => {
    if (isSettingsWindow || !isTauriRuntime()) return;
    let cancelled = false;

    const setupShortcuts = async () => {
      try {
        await unregisterAll();
        if (cancelled) return;

        const toggleKey = settings.shortcut_toggle_island?.trim();
        if (toggleKey) {
          await register(toggleKey, (event) => {
            if (event.state === "Pressed") {
              const st = useIslandStore.getState();
              st.setMode(st.mode === "expanded" ? "compact" : "expanded");
            }
          }).catch(() => {});
        }

        const chatKey = settings.shortcut_open_chat?.trim();
        if (chatKey && chatKey !== toggleKey) {
          await register(chatKey, (event) => {
            if (event.state === "Pressed") {
              const st = useIslandStore.getState();
              st.setActiveTab("drop");
            }
          }).catch(() => {});
        }
      } catch {
      }
    };

    void setupShortcuts();
    return () => {
      cancelled = true;
    };
  }, [isSettingsWindow, settings.shortcut_toggle_island, settings.shortcut_open_chat]);

  // Event listeners
  useEffect(() => {
    if (isSettingsWindow || !isTauriRuntime()) return;

    void refreshSecretStatus().then(() => {
      void refreshGitHub();
      void refreshCalendar();
      void refreshAiProviders();
    });
    void refreshMedia();
    void tauriApi
      .getSystemStats()
      .then((s) => {
        if (s) setSystemStats(s);
      })
      .catch(() => {});
    void tauriApi
      .getAiSessions()
      .then((sessions) => {
        if (sessions) setAiSessions(sessions);
      })
      .catch(() => {});
    void tauriApi
      .getPendingPermissions()
      .then((reqs) => {
        if (reqs) setPermissions(reqs);
      })
      .catch(() => {});

    const unlisteners: Array<() => void> = [];

    void listen("nobi:secrets-updated", () => {
      void refreshSecretStatus().then(() => {
        void refreshGitHub();
        void refreshCalendar();
        void refreshAiProviders();
      });
    }).then((u) => unlisteners.push(u));

    void listen<AiSession[]>("nobi:ai-sessions", (e) => {
      setAiSessions(e.payload);
    }).then((u) => unlisteners.push(u));

    void listen<SystemStats>("nobi:system-stats", (e) => {
      setSystemStats(e.payload);
    }).then((u) => unlisteners.push(u));

    void listen<PermissionRequest>("nobi:permission-request", (e) => {
      pushPermission(e.payload);
    }).then((u) => unlisteners.push(u));

    void listen<Omit<AgentActivityEvent, "id" | "timestamp">>("nobi:agent-event", (e) => {
      pushAgentEvent(e.payload);
    }).then((u) => unlisteners.push(u));

    void listen<HudEventPayload>("nobi:hud-event", (e) => {
      showHud(e.payload);
    }).then((u) => unlisteners.push(u));

    void listen<OsNotificationItem>("nobi:os-notification", (e) => {
      pushOsNotification(e.payload);
    }).then((u) => unlisteners.push(u));

    void listen<{ request_id: string; delta: string }>("nobi:chat-chunk", (e) => {
      appendChatChunk(e.payload.request_id, e.payload.delta);
    }).then((u) => unlisteners.push(u));

    void listen<ReceivedDropItem>("nobi:drop-received", (e) => {
      pushReceivedDropItem(e.payload);
    }).then((u) => unlisteners.push(u));

    void listen<number>("nobi:drop-clients", (e) => {
      setLocalDropClients(e.payload);
    }).then((u) => unlisteners.push(u));

    void listen("nobi:toggle-expand", () => {
      const st = useIslandStore.getState();
      st.setMode(st.mode === "expanded" ? "compact" : "expanded");
    }).then((u) => unlisteners.push(u));

    void getCurrentWebview()
      .onDragDropEvent((event) => {
        if (event.payload.type === "over") {
          setDraggingFile(true);
        } else if (event.payload.type === "drop") {
          addDroppedFiles(event.payload.paths);
        } else {
          setDraggingFile(false);
        }
      })
      .then((u) => unlisteners.push(u));

    return () => {
      unlisteners.forEach((u) => u());
    };
  }, [
    isSettingsWindow,
    setSystemStats,
    setAiSessions,
    setPermissions,
    pushPermission,
    pushAgentEvent,
    showHud,
    pushOsNotification,
    appendChatChunk,
    pushReceivedDropItem,
    setLocalDropClients,
    setDraggingFile,
    addDroppedFiles,
    refreshSecretStatus,
    refreshAiProviders,
    refreshGitHub,
    refreshCalendar,
    refreshMedia,
  ]);

  useEffect(() => {
    if (isSettingsWindow || !isTauriRuntime() || !settings.media_sync_enabled) return;
    const id = setInterval(() => void refreshMedia(), 3500);
    return () => clearInterval(id);
  }, [isSettingsWindow, settings.media_sync_enabled, refreshMedia]);

  useEffect(() => {
    if (isSettingsWindow || !isTauriRuntime()) return;
    const mins = Math.max(1, settings.github_refresh_mins || 10);
    const id = setInterval(() => void refreshGitHub(), mins * 60 * 1000);
    return () => clearInterval(id);
  }, [isSettingsWindow, settings.github_refresh_mins, refreshGitHub]);

  const isAttached = settings.island_style === "attached";
  const alignClass =
    settings.island_position === "top-left"
      ? isAttached
        ? "items-start pl-0"
        : "items-start pl-2"
      : settings.island_position === "top-right"
      ? isAttached
        ? "items-end pr-0"
        : "items-end pr-2"
      : "items-center";

  if (isSettingsWindow) {
    return <SettingsWindow />;
  }

  return (
    <div
      className={`w-full h-full flex flex-col ${alignClass} justify-start ${
        isAttached ? "pt-0" : "pt-1"
      } bg-transparent`}
    >
      <IslandShell />
    </div>
  );
}
