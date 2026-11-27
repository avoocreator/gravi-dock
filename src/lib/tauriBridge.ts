import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import {
  AiSession,
  AppSettings,
  AudioAppSession,
  CalendarEventItem,
  DetectedAiProvider,
  GitHubSummary,
  HardwareStatus,
  LocalDropStatus,
  MediaState,
  PermissionRequest,
  SecretStatus,
  SystemStats,
  WifiCredentials,
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
    }
  }
  window.open(url, "_blank", "noopener,noreferrer");
}

export const tauriApi = {
  openSettingsWindow: () => safeInvoke<void>("open_settings_window"),
  recenterIsland: (position?: string, style?: string) =>
    safeInvoke<void>("recenter_island", { position, style }),
  updateHitRect: (left: number, top: number, right: number, bottom: number) =>
    safeInvoke<void>("update_hit_rect", { left, top, right, bottom }),
  getSettings: () => safeInvoke<AppSettings>("get_settings"),
  saveSettings: (settings: AppSettings) => safeInvoke<AppSettings>("save_settings", { settings }),
  resetSettings: () => safeInvoke<AppSettings>("reset_settings"),
  getSystemStats: () => safeInvoke<SystemStats>("get_system_stats"),
  getAiSessions: () => safeInvoke<AiSession[]>("get_ai_sessions"),
  focusAiWindow: (appName: string) => safeInvoke<string>("focus_ai_window", { appName }),
  detectAiProviders: (ollamaEndpoint?: string) =>
    safeInvoke<DetectedAiProvider[]>("detect_ai_providers", { ollamaEndpoint }),
  fetchProviderModels: (provider: string, ollamaEndpoint?: string) =>
    safeInvoke<string[]>("fetch_provider_models", { provider, ollamaEndpoint }),
  getPendingPermissions: () => safeInvoke<PermissionRequest[]>("get_pending_permissions"),
  respondPermission: (id: string, decision: "allow" | "deny" | "always") =>
    safeInvoke<void>("respond_permission", { id, decision }),
  fetchGitHubSummary: (rangeMonths: 3 | 6 | 12) =>
    safeInvoke<GitHubSummary>("fetch_github_summary", { rangeMonths }),
  fetchCalendarEvents: (year: number, month: number) =>
    safeInvoke<CalendarEventItem[]>("fetch_calendar_events", { year, month }),
  getMediaState: () => safeInvoke<MediaState>("get_media_state"),
  mediaControl: (
    action: "play_pause" | "next" | "previous" | "volume_up" | "volume_down" | "mute"
  ) => safeInvoke<void>("media_control", { action }),
  mediaSeek: (positionSecs: number) =>
    safeInvoke<void>("media_seek", { positionSecs }),
  sendChatMessage: (payload: {
    provider: string;
    model: string;
    prompt: string;
    attached_files: string[];
    ollama_endpoint?: string;
    system_prompt?: string;
    max_file_kb?: number;
    request_id?: string;
  }) => safeInvoke<string>("send_chat_message", { payload }),
  saveSecret: (key: string, value: string) => safeInvoke<void>("save_secret", { key, value }),
  getSecretStatus: () => safeInvoke<SecretStatus>("get_secret_status"),
  triggerTestHud: (kind: string) => safeInvoke<void>("trigger_test_hud", { kind }),
  replyOsNotification: (appName: string, replyText: string, senderTitle?: string) =>
    safeInvoke<string>("reply_os_notification", { appName, senderTitle, replyText }),
  triggerTestNotification: () => safeInvoke<void>("trigger_test_notification"),
  getLocalDropStatus: () => safeInvoke<LocalDropStatus>("get_local_drop_status"),
  startLocalDrop: () => safeInvoke<LocalDropStatus>("start_local_drop"),
  stopLocalDrop: () => safeInvoke<LocalDropStatus>("stop_local_drop"),
  addLocalDropFiles: (paths: string[]) =>
    safeInvoke<LocalDropStatus>("add_local_drop_files", { paths }),
  removeLocalDropFile: (id: string) =>
    safeInvoke<LocalDropStatus>("remove_local_drop_file", { id }),
  getHardwareStatus: () => safeInvoke<HardwareStatus>("get_hardware_status"),
  getWifiCredentials: () => safeInvoke<WifiCredentials>("get_wifi_credentials"),
  openBluetoothSettings: () => safeInvoke<void>("open_bluetooth_settings"),
  listAudioSessions: () => safeInvoke<AudioAppSession[]>("list_audio_sessions"),
  setAppAudioVolume: (appKey: string, volumePercent: number) =>
    safeInvoke<void>("set_app_audio_volume", { appKey, volumePercent }),
  setAppAudioMute: (appKey: string, muted: boolean) =>
    safeInvoke<void>("set_app_audio_mute", { appKey, muted }),
};
