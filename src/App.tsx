import React, { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { IslandShell } from "./components/IslandShell";
import { SettingsWindow } from "./components/SettingsWindow";
import { useIslandStore } from "./store/useIslandStore";
import { isTauriRuntime } from "./lib/tauriBridge";
import { AiSession, AppSettings, PermissionRequest, SystemStats } from "./types/island";

export default function App() {
  const isSettingsWindow = isTauriRuntime()
    ? getCurrentWindow().label === "settings"
    : typeof window !== "undefined" &&
      new URLSearchParams(window.location.search).get("window") === "settings";

  const {
    setSystemStats,
    setAiSessions,
    pushPermission,
    tickPomodoro,
    setDraggingFile,
    addDroppedFiles,
    refreshSecretStatus,
    refreshMedia,
    loadSettings,
    applySettings,
  } = useIslandStore();

  useEffect(() => {
    void loadSettings();
  }, [loadSettings]);

  // Real-time multi-window settings sync listener
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

  // 1-second Pomodoro ticker (main island only)
  useEffect(() => {
    if (isSettingsWindow) return;
    const id = setInterval(() => tickPomodoro(), 1000);
    return () => clearInterval(id);
  }, [tickPomodoro, isSettingsWindow]);

  // Tauri 2 native event listeners (main island only)
  useEffect(() => {
    if (isSettingsWindow || !isTauriRuntime()) return;

    void refreshSecretStatus();
    void refreshMedia();

    const unlisteners: Array<() => void> = [];

    void listen<AiSession[]>("nobi:ai-sessions", (e) => {
      setAiSessions(e.payload);
    }).then((u) => unlisteners.push(u));

    void listen<SystemStats>("nobi:system-stats", (e) => {
      setSystemStats(e.payload);
    }).then((u) => unlisteners.push(u));

    void listen<PermissionRequest>("nobi:permission-request", (e) => {
      pushPermission(e.payload);
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
    pushPermission,
    setDraggingFile,
    addDroppedFiles,
    refreshSecretStatus,
    refreshMedia,
  ]);

  if (isSettingsWindow) {
    return <SettingsWindow />;
  }

  return (
    <div className="w-full h-full flex flex-col items-center justify-start pt-1 bg-transparent">
      <IslandShell />
    </div>
  );
}
