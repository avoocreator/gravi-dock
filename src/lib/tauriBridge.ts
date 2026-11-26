import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import {
  AiSession,
  CalendarEventItem,
  GitHubSummary,
  MediaState,
  PermissionRequest,
  AppSettings,
  SecretStatus,
  SystemStats,
} from "../types/island";

export function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export async function safeInvoke<T>(
  cmd: string,
  args?: Record<string, unknown>
): Promise<T | null> {
  if (!isTauriRuntime()) return null;
  try {
    return await invoke<T>(cmd, args);
  } catch (err) {
    console.warn(`[tauriBridge] ${cmd} failed:`, err);
    throw err;
  }
}

export async function openExternalUrl(url: string): Promise<void> {
  if (!url) return;
  if (isTauriRuntime()) {
    try {
      await openUrl(url);
      return;
    } catch {
      // fallback
    }
  }
  window.open(url, "_blank", "noopener,noreferrer");
}

export const tauriApi = {
  setClickThrough: (ignore: boolean) => safeInvoke<void>("set_click_through", { ignore }),
  openSettingsWindow: () => safeInvoke<void>("open_settings_window"),
  recenterIsland: (position?: string) => safeInvoke<void>("recenter_island", { position }),
  getSettings: () => safeInvoke<AppSettings>("get_settings"),
  saveSettings: (settings: AppSettings) => safeInvoke<AppSettings>("save_settings", { settings }),
  resetSettings: () => safeInvoke<AppSettings>("reset_settings"),
  getSystemStats: () => safeInvoke<SystemStats>("get_system_stats"),
  getAiSessions: () => safeInvoke<AiSession[]>("get_ai_sessions"),
  focusAiWindow: (appName: string) => safeInvoke<string>("focus_ai_window", { appName }),
  getPendingPermissions: () => safeInvoke<PermissionRequest[]>("get_pending_permissions"),
  respondPermission: (id: string, decision: "allow" | "deny" | "always") =>
    safeInvoke<void>("respond_permission", { id, decision }),
  fetchGitHubSummary: (rangeMonths: 3 | 6 | 12) =>
    safeInvoke<GitHubSummary>("fetch_github_summary", { rangeMonths }),
  fetchCalendarEvents: (year: number, month: number) =>
    safeInvoke<CalendarEventItem[]>("fetch_calendar_events", { year, month }),
  getMediaState: () => safeInvoke<MediaState>("get_media_state"),
  mediaControl: (action: "play_pause" | "next" | "previous") =>
    safeInvoke<void>("media_control", { action }),
  sendChatMessage: (payload: {
    provider: string;
    model: string;
    prompt: string;
    attached_files: string[];
  }) => safeInvoke<string>("send_chat_message", { payload }),
  saveSecret: (key: string, value: string) => safeInvoke<void>("save_secret", { key, value }),
  getSecretStatus: () => safeInvoke<SecretStatus>("get_secret_status"),
};
