import { create } from "zustand";
import {
  AgentActivityEvent,
  AiSession,
  CalendarReminderBanner,
  AppSettings,
  CalendarEventItem,
  ChatMessage,
  DetectedAiProvider,
  FocusHabitItem,
  GitHubSummary,
  HardwareStatus,
  GroupedOsNotification,
  HudEventPayload,
  IslandMode,
  LocalDropStatus,
  MediaState,
  GraviMood,
  OsNotificationItem,
  PermissionRequest,
  PillTab,
  ReceivedDropItem,
  SecretStatus,
  SystemStats,
} from "../types/island";
import { isTauriRuntime, tauriApi } from "../lib/tauriBridge";
import { playGraviSound } from "../lib/soundEffects";

export const DEFAULT_SETTINGS: AppSettings = {
  island_position: "top-center",
  island_style: "floating",
  open_on_hover: true,
  hover_close_delay_ms: 450,
  auto_hide_idle: false,
  clock_24h: true,
  show_seconds: false,
  start_on_boot: false,
  island_opacity: 96,
  eye_tracking: true,
  ear_animations: true,
  squish_dizzy: true,
  sound_enabled: true,
  sound_volume: 70,
  monitored_ais: [
    "claude-desktop",
    "gemini-desktop",
    "chatgpt-desktop",
    "antigravity-desktop",
    "hermes-desktop",
    "openbot-desktop",
    "antigravity-cli",
    "copilot-cli",
  ],
  max_compact_logos: 4,
  poll_interval_secs: 3,
  github_default_range: 3,
  github_default_items: 3,
  github_refresh_mins: 10,
  calendar_reminder_mins: 15,
  calendar_week_start: "monday",
  calendar_ai_provider: "ollama",
  calendar_ai_model: "llama3.2",
  chat_default_provider: "ollama",
  chat_default_model: "llama3.2",
  ollama_endpoint: "http://127.0.0.1:11434",
  chat_system_prompt: "Kamu adalah Gravi, asisten developer yang ringkas dan membantu.",
  max_drop_file_kb: 256,
  pomo_focus_mins: 25,
  pomo_break_mins: 5,
  media_sync_enabled: true,
  media_dance_enabled: true,
  shortcut_toggle_island: "Alt+Space",
  shortcut_open_chat: "Alt+C",
  hud_enabled: true,
  hud_capslock: true,
  hud_battery_charging: true,
  hud_battery_low: true,
  hud_wifi: true,
  hud_ai_done: true,
  hud_windows_notif: true,
  notif_messaging_only: false,
  hud_duration_ms: 3500,
  hud_bt_low: true,
  focus_habit_deadline: "21:00",
  focus_habit_warn_mins: 30,
  ambient_auto_focus: false,
};

function toLocalIsoDate(d: Date): string {
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(
    d.getDate()
  ).padStart(2, "0")}`;
}

function loadTodayHabits(): FocusHabitItem[] {
  if (typeof window === "undefined") return [];
  try {
    const raw = window.localStorage.getItem(`gravi-habits-${toLocalIsoDate(new Date())}`);
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    return [];
  }
}

function saveTodayHabits(items: FocusHabitItem[]) {
  if (typeof window === "undefined") return;
  try {
    window.localStorage.setItem(
      `gravi-habits-${toLocalIsoDate(new Date())}`,
      JSON.stringify(items)
    );
  } catch {
  }
}

const WELCOME_MSG: ChatMessage = {
  id: "welcome-1",
  role: "assistant",
  provider: "gravi",
  content:
    "Halo! Aku Gravi. Kamu bisa tanya apa saja, lampirkan file lewat drag & drop ke island, atau pilih provider AI yang terdeteksi di atas.",
  timestamp: Date.now(),
};

const FALLBACK_PROVIDERS: DetectedAiProvider[] = [
  {
    id: "ollama",
    name: "Ollama (Lokal)",
    installed: true,
    running: false,
    has_account: false,
    source: "local-server",
    status_label: "Lokal",
    models: ["llama3.2", "qwen2.5", "gemma2"],
    default_model: "llama3.2",
  },
  {
    id: "gemini",
    name: "Google Gemini",
    installed: false,
    running: false,
    has_account: false,
    source: "api-key",
    status_label: "Tambah API Key",
    models: ["gemini-2.5-flash", "gemini-2.5-pro", "gemini-2.0-flash"],
    default_model: "gemini-2.5-flash",
  },
  {
    id: "anthropic",
    name: "Anthropic Claude",
    installed: false,
    running: false,
    has_account: false,
    source: "api-key",
    status_label: "Tambah API Key",
    models: ["claude-sonnet-4-20250514", "claude-3-5-sonnet-latest", "claude-3-5-haiku-latest"],
    default_model: "claude-3-5-sonnet-latest",
  },
  {
    id: "openai",
    name: "OpenAI / ChatGPT",
    installed: false,
    running: false,
    has_account: false,
    source: "api-key",
    status_label: "Tambah API Key",
    models: ["gpt-4o-mini", "gpt-4o", "o4-mini"],
    default_model: "gpt-4o-mini",
  },
];

interface IslandStore {
  mode: IslandMode;
  activeTab: PillTab;
  mood: GraviMood;

  // AI
  aiSessions: AiSession[];
  detectedProviders: DetectedAiProvider[];
  providersLoading: boolean;
  permissions: PermissionRequest[];
  agentEvents: AgentActivityEvent[];

  // GitHub
  githubRange: 3 | 6 | 12;
  githubData: GitHubSummary | null;
  githubLoading: boolean;
  githubError: string | null;

  // Calendar
  calYear: number;
  calMonth: number;
  selectedDate: string;
  calendarEvents: CalendarEventItem[];
  calendarLoading: boolean;
  calendarError: string | null;
  calendarAiProvider: string;
  calendarAiModel: string;
  calendarAiReply: string | null;
  calendarAiLoading: boolean;
  calendarAiError: string | null;
  activeReminder: CalendarReminderBanner | null;
  activeHud: HudEventPayload | null;
  activeNotif: GroupedOsNotification | null;
  notifQueue: GroupedOsNotification[];
  lastNotifPopAt: number;
  remindedEventIds: string[];

  // Media
  media: MediaState;

  // Pomodoro
  pomoMode: "focus" | "break";
  pomoRunning: boolean;
  pomoSecondsLeft: number;

  // Chat
  chatProvider: string;
  chatModel: string;
  chatMessages: ChatMessage[];
  chatLoading: boolean;
  droppedFiles: string[];
  isDraggingFile: boolean;

  // Local Drop
  localDrop: LocalDropStatus;
  localDropLoading: boolean;
  hardwareStatus: HardwareStatus;
  hardwareLoading: boolean;
  focusHabits: FocusHabitItem[];
  focusHabitAlarmFiredDate: string | null;

  // System
  systemStats: SystemStats;
  cpuHistory: number[];
  settings: AppSettings;
  secretStatus: SecretStatus;

  // Actions
  setMode: (mode: IslandMode) => void;
  setActiveTab: (tab: PillTab) => void;
  setMood: (mood: GraviMood) => void;

  setSystemStats: (stats: SystemStats) => void;
  setAiSessions: (sessions: AiSession[]) => void;
  refreshAiProviders: () => Promise<void>;
  refreshProviderModels: (provider: string) => Promise<string[]>;
  setPermissions: (reqs: PermissionRequest[]) => void;

  pushPermission: (req: PermissionRequest) => void;
  resolvePermission: (id: string, decision: "allow" | "deny" | "always") => Promise<void>;
  pushAgentEvent: (ev: Omit<AgentActivityEvent, "id" | "timestamp">) => void;
  clearAgentEvents: () => void;

  setGitHubRange: (range: 3 | 6 | 12) => Promise<void>;
  refreshGitHub: (overrideRange?: 3 | 6 | 12) => Promise<void>;

  setCalendarMonth: (year: number, month: number) => Promise<void>;
  setSelectedDate: (date: string) => void;
  refreshCalendar: () => Promise<void>;
  setCalendarAiProvider: (provider: string) => void;
  setCalendarAiModel: (model: string) => void;
  askCalendarAi: (question: string) => Promise<void>;
  clearCalendarAiReply: () => void;
  checkCalendarReminders: () => void;
  dismissCalendarReminder: () => void;
  showHud: (hud: HudEventPayload) => void;
  dismissHud: () => void;
  pushOsNotification: (item: OsNotificationItem) => void;
  dismissOsNotification: () => void;
  replyToNotification: (replyText: string) => Promise<string>;

  toggleMediaPlay: () => Promise<void>;
  skipMedia: (dir: "next" | "previous") => Promise<void>;
  seekMedia: (positionSecs: number) => Promise<void>;
  adjustMediaVolume: (action: "volume_up" | "volume_down" | "mute") => Promise<void>;
  tickMediaProgress: () => void;
  refreshMedia: () => Promise<void>;

  togglePomodoro: () => void;
  resetPomodoro: (mode?: "focus" | "break") => void;
  tickPomodoro: () => void;

  setChatProvider: (provider: string) => void;
  setChatModel: (model: string) => void;
  sendChat: (prompt: string) => Promise<void>;
  appendChatChunk: (requestId: string, delta: string) => void;
  clearChat: () => void;

  addDroppedFiles: (paths: string[]) => void;
  removeDroppedFile: (path: string) => void;
  clearDroppedFiles: () => void;
  setDraggingFile: (dragging: boolean) => void;

  refreshSecretStatus: () => Promise<SecretStatus | null>;
  loadSettings: () => Promise<void>;
  applySettings: (next: AppSettings) => void;
  updateSettings: (partial: Partial<AppSettings>) => Promise<void>;
  resetAllSettings: () => Promise<void>;

  refreshLocalDrop: () => Promise<void>;
  toggleLocalDropServer: () => Promise<void>;
  removeLocalDropFile: (id: string) => Promise<void>;
  pushReceivedDropItem: (item: ReceivedDropItem) => void;
  setLocalDropClients: (count: number) => void;

  refreshHardwareStatus: () => Promise<void>;

  addFocusHabit: (text: string) => void;
  toggleFocusHabit: (id: string) => void;
  removeFocusHabit: (id: string) => void;
  checkFocusHabitAlarm: () => void;
}

const now = new Date();

export const useIslandStore = create<IslandStore>((set, get) => ({
  mode: "compact",
  activeTab: "agents",
  mood: "idle",

  aiSessions: [],
  detectedProviders: FALLBACK_PROVIDERS,
  providersLoading: false,
  permissions: [],
  agentEvents: [],

  githubRange: 3,
  githubData: null,
  githubLoading: false,
  githubError: null,

  calYear: now.getFullYear(),
  calMonth: now.getMonth() + 1,
  selectedDate: toLocalIsoDate(now),
  calendarEvents: [],
  calendarLoading: false,
  calendarError: null,
  calendarAiProvider: "ollama",
  calendarAiModel: "llama3.2",
  calendarAiReply: null,
  calendarAiLoading: false,
  calendarAiError: null,
  activeReminder: null,
  activeHud: null,
  activeNotif: null,
  notifQueue: [],
  lastNotifPopAt: 0,
  remindedEventIds: [],

  media: {
    title: "Tidak ada media aktif",
    artist: "Putar musik di Spotify, YouTube, atau pemutar lain",
    album: "",
    is_playing: false,
    position_secs: 0,
    duration_secs: 0,
    player: "",
    artwork_url: null,
    has_media: false,
  },

  pomoMode: "focus",
  pomoRunning: false,
  pomoSecondsLeft: 25 * 60,

  chatProvider: "ollama",
  chatModel: "llama3.2",
  chatMessages: [WELCOME_MSG],
  chatLoading: false,
  droppedFiles: [],
  isDraggingFile: false,

  systemStats: {
    cpu_percent: 0,
    cpu_cores: 0,
    ram_used_gb: 0,
    ram_total_gb: 0,
    ram_percent: 0,
    uptime_secs: 0,
    gpu: {
      name: "Mendeteksi GPU...",
      usage_percent: 0,
      vram_used_mb: 0,
      vram_total_mb: 0,
      temp_c: null,
    },
    top_processes: [],
  },
  localDrop: {
    running: false,
    url: "",
    port: 0,
    ip: "127.0.0.1",
    shared_files: [],
    received_items: [],
    save_dir: "",
  },
  localDropLoading: false,
  hardwareStatus: {
    laptop_battery: { has_battery: true, percent: 100, charging: false },
    bluetooth_devices: [],
    network: { ssid: null, local_ip: "127.0.0.1", down_kbps: 0, up_kbps: 0, ping_ms: null },
  },
  hardwareLoading: false,
  focusHabits: loadTodayHabits(),
  focusHabitAlarmFiredDate: null,
  cpuHistory: [],
  settings: DEFAULT_SETTINGS,
  secretStatus: {
    github_token: false,
    google_ical_url: false,
    google_access_token: false,
    openai_api_key: false,
    anthropic_api_key: false,
    gemini_api_key: false,
  },

  setMode: (mode) => set({ mode }),
  setActiveTab: (activeTab) => set({ activeTab, mode: "expanded" }),
  setMood: (mood) => set({ mood }),

  setSystemStats: (systemStats) => {
    set((s) => ({
      systemStats,
      cpuHistory: [...s.cpuHistory.slice(-17), Math.round(systemStats.cpu_percent)],
    }));
  },

  setAiSessions: (aiSessions) => set({ aiSessions }),

  refreshAiProviders: async () => {
    if (!isTauriRuntime()) return;
    set({ providersLoading: true });
    try {
      const endpoint = get().settings.ollama_endpoint;
      const list = await tauriApi.detectAiProviders(endpoint);
      if (list && list.length > 0) {
        const curCalProv = get().calendarAiProvider;
        const curChatProv = get().chatProvider;
        const matchedCal = list.find((p) => p.id === curCalProv) ?? list.find((p) => p.has_account) ?? list[0];
        const matchedChat = list.find((p) => p.id === curChatProv) ?? list.find((p) => p.has_account) ?? list[0];

        const nextCalModel =
          matchedCal.models.includes(get().calendarAiModel) && get().calendarAiModel
            ? get().calendarAiModel
            : matchedCal.default_model;
        const nextChatModel =
          matchedChat.models.includes(get().chatModel) && get().chatModel
            ? get().chatModel
            : matchedChat.default_model;

        set({
          detectedProviders: list,
          providersLoading: false,
          calendarAiProvider: matchedCal.id,
          calendarAiModel: nextCalModel,
          chatProvider: matchedChat.id,
          chatModel: nextChatModel,
        });
      } else {
        set({ providersLoading: false });
      }
    } catch {
      set({ providersLoading: false });
    }
  },

  refreshProviderModels: async (provider) => {
    if (!isTauriRuntime()) {
      const found = get().detectedProviders.find((p) => p.id === provider);
      return found?.models ?? [];
    }
    try {
      const models = await tauriApi.fetchProviderModels(provider, get().settings.ollama_endpoint);
      if (models && models.length > 0) {
        set((s) => ({
          detectedProviders: s.detectedProviders.map((p) =>
            p.id === provider ? { ...p, models, default_model: models[0] } : p
          ),
        }));
        return models;
      }
    } catch {
    }
    return get().detectedProviders.find((p) => p.id === provider)?.models ?? [];
  },

  setPermissions: (permissions) => set({ permissions }),

  pushPermission: (req) => {
    const { settings } = get();
    playGraviSound("alert", settings.sound_enabled, settings.sound_volume);
    set((s) => ({
      permissions: [req, ...s.permissions.filter((p) => p.id !== req.id)],
      mood: "alert",
      mode: "expanded",
      activeTab: "agents",
    }));
  },

  resolvePermission: async (id, decision) => {
    if (isTauriRuntime()) {
      await tauriApi.respondPermission(id, decision).catch((err) => {
        console.warn("[gravi] respondPermission failed:", err);
      });
    }
    const { settings } = get();
    playGraviSound(
      decision === "deny" ? "pop" : "happy",
      settings.sound_enabled,
      settings.sound_volume
    );
    set((s) => {
      const remaining = s.permissions.filter((p) => p.id !== id);
      return {
        permissions: remaining,
        mood: decision === "deny" ? "squish" : "happy",
      };
    });
    setTimeout(() => {
      const st = get();
      if (st.permissions.length === 0) {
        set({
          mood: st.media.is_playing && st.settings.media_dance_enabled ? "dancing" : "idle",
        });
      }
    }, 1600);
  },

  pushAgentEvent: (ev) => {
    set((s) => ({
      agentEvents: [
        {
          ...ev,
          id: `ev-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`,
          timestamp: Date.now(),
        },
        ...s.agentEvents,
      ].slice(0, 12),
    }));
  },

  clearAgentEvents: () => set({ agentEvents: [] }),

  setGitHubRange: async (range) => {
    set({ githubRange: range });
    await get().refreshGitHub(range);
  },

  refreshGitHub: async (overrideRange) => {
    if (!isTauriRuntime()) return;
    if (!get().secretStatus.github_token) {
      set({ githubData: null, githubError: null, githubLoading: false });
      return;
    }
    const targetRange = overrideRange ?? get().githubRange;
    set({ githubLoading: true, githubError: null });
    try {
      const data = await tauriApi.fetchGitHubSummary(targetRange);
      if (data && get().githubRange === targetRange) {
        set({ githubData: data, githubLoading: false });
      } else {
        set({ githubLoading: false });
      }
    } catch (err) {
      set({
        githubLoading: false,
        githubError: String(err),
      });
    }
  },

  setCalendarMonth: async (year, month) => {
    set({ calYear: year, calMonth: month });
    await get().refreshCalendar();
  },

  setSelectedDate: (selectedDate) => set({ selectedDate }),

  refreshCalendar: async () => {
    if (!isTauriRuntime()) return;
    const { secretStatus, calYear, calMonth } = get();
    if (!secretStatus.google_ical_url && !secretStatus.google_access_token) {
      set({
        calendarEvents: [],
        calendarError: "Google Calendar belum terhubung.",
        calendarLoading: false,
      });
      return;
    }
    set({ calendarLoading: true, calendarError: null });
    try {
      const events = await tauriApi.fetchCalendarEvents(calYear, calMonth);
      if (events) set({ calendarEvents: events, calendarLoading: false });
    } catch (err) {
      set({
        calendarLoading: false,
        calendarError: String(err),
      });
    }
  },

  setCalendarAiProvider: (provider) => {
    const found = get().detectedProviders.find((p) => p.id === provider);
    const nextModel = found?.default_model || found?.models[0] || "";
    set({ calendarAiProvider: provider, calendarAiModel: nextModel, calendarAiError: null });
    void get().updateSettings({
      calendar_ai_provider: provider,
      calendar_ai_model: nextModel,
    });
  },

  setCalendarAiModel: (model) => {
    set({ calendarAiModel: model, calendarAiError: null });
    void get().updateSettings({ calendar_ai_model: model });
  },

  clearCalendarAiReply: () => set({ calendarAiReply: null, calendarAiError: null }),

  dismissCalendarReminder: () => set({ activeReminder: null }),

  showHud: (hud) => {
    const { settings } = get();
    if (!settings.hud_enabled) return;
    if (hud.kind === "battery_low") {
      playGraviSound("alert", settings.sound_enabled, settings.sound_volume);
    } else if (hud.kind === "ai_done" || hud.kind === "battery_full") {
      playGraviSound("chime", settings.sound_enabled, settings.sound_volume);
    } else {
      playGraviSound("pop", settings.sound_enabled, settings.sound_volume);
    }
    set({
      activeHud: hud,
      mood: (hud.mood as GraviMood) || "idle",
    });
  },

  dismissHud: () => {
    const { media, settings } = get();
    set({
      activeHud: null,
      mood: media.is_playing && settings.media_dance_enabled ? "dancing" : "idle",
    });
  },

  pushOsNotification: (item) => {
    const { settings, activeNotif, notifQueue, lastNotifPopAt } = get();
    if (!settings.hud_enabled || !settings.hud_windows_notif) return;
    const nowTs = Date.now();

    if (
      activeNotif &&
      activeNotif.app_name.toLowerCase() === item.app_name.toLowerCase() &&
      nowTs - activeNotif.timestamp < 15000
    ) {
      set({
        activeNotif: {
          ...activeNotif,
          title: item.title,
          body: item.body || activeNotif.body,
          timestamp: nowTs,
          count: activeNotif.count + 1,
        },
      });
      return;
    }

    if (activeNotif || nowTs - lastNotifPopAt < 5000) {
      const existingIdx = notifQueue.findIndex(
        (q) => q.app_name.toLowerCase() === item.app_name.toLowerCase()
      );
      if (existingIdx >= 0) {
        const nextQ = [...notifQueue];
        nextQ[existingIdx] = {
          ...nextQ[existingIdx],
          title: item.title,
          body: item.body || nextQ[existingIdx].body,
          timestamp: nowTs,
          count: nextQ[existingIdx].count + 1,
        };
        set({ notifQueue: nextQ });
      } else {
        set({
          notifQueue: [...notifQueue.slice(-5), { ...item, count: 1 }],
        });
      }
      return;
    }

    playGraviSound("pop", settings.sound_enabled, settings.sound_volume);
    set({
      activeNotif: { ...item, count: 1 },
      activeHud: null,
      lastNotifPopAt: nowTs,
      mood: "notif",
    });
  },

  dismissOsNotification: () => {
    const { notifQueue, media, settings } = get();
    if (notifQueue.length > 0) {
      const [next, ...rest] = notifQueue;
      set({
        activeNotif: next,
        notifQueue: rest,
        lastNotifPopAt: Date.now(),
        mood: "notif",
      });
    } else {
      set({
        activeNotif: null,
        mood: media.is_playing && settings.media_dance_enabled ? "dancing" : "idle",
      });
    }
  },

  replyToNotification: async (replyText) => {
    const { activeNotif } = get();
    if (!activeNotif) return "";
    if (!isTauriRuntime()) return "Balasan disiapkan (mode preview)";
    try {
      const res = await tauriApi.replyOsNotification(activeNotif.app_name, replyText, activeNotif.title);
      return res ?? `Terkirim ke ${activeNotif.app_name}`;
    } catch (err) {
      return `Gagal mengirim: ${String(err)}`;
    }
  },

  checkCalendarReminders: () => {
    const { calendarEvents, settings, remindedEventIds } = get();
    const leadMins = settings.calendar_reminder_mins ?? 0;
    if (leadMins <= 0 || calendarEvents.length === 0) return;

    const nowDate = new Date();
    const todayIso = toLocalIsoDate(nowDate);
    const nowTotalMins = nowDate.getHours() * 60 + nowDate.getMinutes();

    for (const ev of calendarEvents) {
      if (ev.date !== todayIso) continue;
      const key = `${ev.date}:${ev.id}`;
      if (remindedEventIds.includes(key)) continue;
      const m = /^(\d{2}):(\d{2})/.exec(ev.start_time);
      if (!m) continue;
      const evMins = Number(m[1]) * 60 + Number(m[2]);
      const diff = evMins - nowTotalMins;
      if (diff >= 0 && diff <= leadMins) {
        playGraviSound("alert", settings.sound_enabled, settings.sound_volume);
        set((s) => ({
          activeReminder: {
            eventId: ev.id,
            title: ev.title,
            startTime: ev.start_time,
            minsUntil: diff,
          },
          remindedEventIds: [...s.remindedEventIds, key],
          mood: "alert",
          mode: "expanded",
        }));
        break;
      }
    }
  },

  askCalendarAi: async (question) => {
    const q = question.trim();
    if (!q) return;
    const {
      calendarAiProvider,
      calendarAiModel,
      calendarEvents,
      selectedDate,
      calYear,
      calMonth,
      settings,
    } = get();

    set({ calendarAiLoading: true, calendarAiError: null, mood: "thinking" });

    if (!isTauriRuntime()) {
      set({
        calendarAiLoading: false,
        calendarAiError: "Jalankan lewat aplikasi desktop Tauri untuk menghubungi AI.",
        mood: "idle",
      });
      return;
    }

    const dayEvents = calendarEvents.filter((e) => e.date === selectedDate);
    const formatEv = (e: CalendarEventItem) =>
      `- [${e.date}] ${e.start_time}${e.end_time ? `-${e.end_time}` : ""}: ${e.title}${
        e.location ? ` (${e.location})` : ""
      }`;

    const contextPrompt = [
      `Tanggal dipilih: ${selectedDate} (Bulan ${calMonth}/${calYear})`,
      `Agenda pada tanggal dipilih (${dayEvents.length}):`,
      dayEvents.length > 0 ? dayEvents.map(formatEv).join("\n") : "- Tidak ada jadwal pada tanggal ini.",
      "",
      `Semua agenda bulan ini (${calendarEvents.length}):`,
      calendarEvents.length > 0
        ? calendarEvents.slice(0, 25).map(formatEv).join("\n")
        : "- Belum ada data kalender bulan ini.",
      "",
      `Permintaan pengguna: ${q}`,
    ].join("\n");

    try {
      const reply = await tauriApi.sendChatMessage({
        provider: calendarAiProvider,
        model: calendarAiModel,
        prompt: contextPrompt,
        attached_files: [],
        ollama_endpoint: settings.ollama_endpoint,
        system_prompt:
          "Kamu adalah Gravi, asisten kalender yang ringkas, jelas, dan membantu dalam bahasa Indonesia. Jawab padat (maksimal 5 baris/poin) berdasarkan data kalender yang diberikan.",
        max_file_kb: settings.max_drop_file_kb,
      });
      set({
        calendarAiReply: reply ?? "(Respons kosong)",
        calendarAiLoading: false,
        mood: "excited",
      });
      setTimeout(() => {
        if (get().mood === "excited") set({ mood: "idle" });
      }, 1500);
    } catch (err) {
      set({
        calendarAiLoading: false,
        calendarAiError: String(err),
        mood: "idle",
      });
    }
  },

  toggleMediaPlay: async () => {
    if (!isTauriRuntime()) return;
    const nextPlaying = !get().media.is_playing;
    await tauriApi.mediaControl("play_pause").catch((err) => {
      console.warn("[gravi] mediaControl failed:", err);
    });
    set((s) => ({
      media: { ...s.media, is_playing: nextPlaying },
      mood: nextPlaying && s.settings.media_dance_enabled ? "dancing" : "idle",
    }));
    setTimeout(() => {
      void get().refreshMedia();
    }, 350);
  },

  skipMedia: async (dir) => {
    if (!isTauriRuntime()) return;
    await tauriApi.mediaControl(dir).catch((err) => {
      console.warn("[gravi] mediaControl failed:", err);
    });
    setTimeout(() => {
      void get().refreshMedia();
    }, 450);
  },

  seekMedia: async (positionSecs) => {
    const clamped = Math.max(0, Math.round(positionSecs));
    set((s) => ({
      media: { ...s.media, position_secs: clamped },
    }));
    if (!isTauriRuntime()) return;
    await tauriApi.mediaSeek(clamped).catch((err) => {
      console.warn("[gravi] mediaSeek failed:", err);
    });
    setTimeout(() => {
      void get().refreshMedia();
    }, 350);
  },

  adjustMediaVolume: async (action) => {
    if (!isTauriRuntime()) return;
    await tauriApi.mediaControl(action).catch((err) => {
      console.warn("[gravi] volume control failed:", err);
    });
  },

  tickMediaProgress: () => {
    const { media } = get();
    if (!media.is_playing || !media.has_media) return;
    if (media.duration_secs > 0 && media.position_secs >= media.duration_secs) return;
    set((s) => ({
      media: {
        ...s.media,
        position_secs: s.media.position_secs + 1,
      },
    }));
  },

  refreshMedia: async () => {
    if (!isTauriRuntime() || !get().settings.media_sync_enabled) return;
    try {
      const state = await tauriApi.getMediaState();
      if (state) {
        const curMood = get().mood;
        const canDance = get().settings.media_dance_enabled;
        const nextMood: GraviMood =
          state.is_playing && canDance
            ? curMood === "idle" || curMood === "dancing"
              ? "dancing"
              : curMood
            : curMood === "dancing"
            ? "idle"
            : curMood;
        set({ media: state, mood: nextMood });
      }
    } catch {
    }
  },

  togglePomodoro: () => {
    const { settings, pomoRunning } = get();
    const nextRunning = !pomoRunning;
    playGraviSound("pop", settings.sound_enabled, settings.sound_volume);
    set({
      pomoRunning: nextRunning,
      mood: nextRunning ? "focus" : "idle",
    });
  },

  resetPomodoro: (mode) => {
    const { pomoMode, settings } = get();
    const target = mode ?? pomoMode;
    const focusMins = Math.max(1, settings.pomo_focus_mins || 25);
    const breakMins = Math.max(1, settings.pomo_break_mins || 5);
    set({
      pomoMode: target,
      pomoRunning: false,
      pomoSecondsLeft: (target === "focus" ? focusMins : breakMins) * 60,
      mood: "idle",
    });
  },

  tickPomodoro: () => {
    const { pomoRunning, pomoSecondsLeft, pomoMode, settings } = get();
    if (!pomoRunning) return;
    if (pomoSecondsLeft <= 1) {
      const nextMode = pomoMode === "focus" ? "break" : "focus";
      const focusMins = Math.max(1, settings.pomo_focus_mins || 25);
      const breakMins = Math.max(1, settings.pomo_break_mins || 5);
      playGraviSound("chime", settings.sound_enabled, settings.sound_volume);
      set({
        pomoMode: nextMode,
        pomoRunning: false,
        pomoSecondsLeft: (nextMode === "focus" ? focusMins : breakMins) * 60,
        mood: "excited",
      });
    } else {
      set({ pomoSecondsLeft: pomoSecondsLeft - 1 });
    }
  },

  setChatProvider: (chatProvider) => {
    const found = get().detectedProviders.find((p) => p.id === chatProvider);
    const defaultModels: Record<string, string> = {
      ollama: "llama3.2",
      lmstudio: "local-model",
      openai: "gpt-4o-mini",
      anthropic: "claude-3-5-sonnet-latest",
      gemini: "gemini-2.5-flash",
      "github-models": "gpt-4o-mini",
      "claude-cli": "sonnet",
      "gemini-cli": "gemini-2.5-flash",
    };
    const nextModel = found?.default_model || defaultModels[chatProvider] || "";
    set({ chatProvider, chatModel: nextModel });
  },

  setChatModel: (chatModel) => set({ chatModel }),

  clearChat: () => set({ chatMessages: [{ ...WELCOME_MSG, timestamp: Date.now() }] }),

  appendChatChunk: (requestId, delta) => {
    if (!delta) return;
    set((s) => ({
      chatMessages: s.chatMessages.map((m) =>
        m.id === requestId ? { ...m, content: m.content + delta } : m
      ),
    }));
  },

  sendChat: async (prompt) => {
    const trimmed = prompt.trim();
    if (!trimmed) return;
    const { chatProvider, chatModel, droppedFiles, settings } = get();
    const nowTs = Date.now();
    const reqId = `a-${nowTs}`;

    const userMsg: ChatMessage = {
      id: `u-${nowTs}`,
      role: "user",
      content: trimmed,
      provider: chatProvider,
      files: droppedFiles.length > 0 ? [...droppedFiles] : undefined,
      timestamp: nowTs,
    };

    const assistantPlaceholder: ChatMessage = {
      id: reqId,
      role: "assistant",
      provider: chatProvider,
      content: "",
      timestamp: nowTs + 1,
    };

    set((s) => ({
      chatMessages: [...s.chatMessages, userMsg, assistantPlaceholder],
      chatLoading: true,
      droppedFiles: [],
      mood: "thinking",
    }));

    if (!isTauriRuntime()) {
      set((s) => ({
        chatMessages: s.chatMessages.map((m) =>
          m.id === reqId
            ? { ...m, content: "Jalankan aplikasi lewat Tauri desktop untuk menghubungi AI." }
            : m
        ),
        chatLoading: false,
        mood: "idle",
      }));
      return;
    }

    try {
      const answer = await tauriApi.sendChatMessage({
        provider: chatProvider,
        model: chatModel,
        prompt: trimmed,
        attached_files: userMsg.files ?? [],
        ollama_endpoint: settings.ollama_endpoint,
        system_prompt: settings.chat_system_prompt,
        max_file_kb: settings.max_drop_file_kb,
        request_id: reqId,
      });
      set((s) => ({
        chatMessages: s.chatMessages.map((m) =>
          m.id === reqId
            ? { ...m, content: m.content.trim().length > 0 ? m.content : (answer ?? "(Tidak ada respons)") }
            : m
        ),
        chatLoading: false,
        mood: "happy",
      }));
      setTimeout(() => {
        if (get().mood === "happy") set({ mood: "idle" });
      }, 1500);
    } catch (err) {
      set((s) => ({
        chatMessages: s.chatMessages.map((m) =>
          m.id === reqId ? { ...m, content: `Gagal menghubungi ${chatProvider}: ${String(err)}` } : m
        ),
        chatLoading: false,
        mood: "squish",
      }));
    }
  },

  addDroppedFiles: (paths) => {
    const { settings } = get();
    playGraviSound("happy", settings.sound_enabled, settings.sound_volume);
    set((s) => ({
      droppedFiles: Array.from(new Set([...s.droppedFiles, ...paths])),
      isDraggingFile: false,
      mood: "happy",
      mode: "expanded",
      activeTab: "drop",
    }));
    if (isTauriRuntime() && paths.length > 0) {
      void tauriApi.addLocalDropFiles(paths).then((st) => {
        if (st) set({ localDrop: st });
      }).catch(() => {});
    }
  },

  removeDroppedFile: (path) =>
    set((s) => ({ droppedFiles: s.droppedFiles.filter((f) => f !== path) })),

  clearDroppedFiles: () => set({ droppedFiles: [] }),

  setDraggingFile: (isDraggingFile) =>
    set((s) => ({
      isDraggingFile,
      mood: isDraggingFile ? "drop" : s.mood === "drop" ? "idle" : s.mood,
    })),

  refreshSecretStatus: async () => {
    if (!isTauriRuntime()) return null;
    try {
      const st = await tauriApi.getSecretStatus();
      if (st) {
        set({ secretStatus: st });
        return st;
      }
    } catch {
    }
    return null;
  },

  loadSettings: async () => {
    if (!isTauriRuntime()) return;
    try {
      const loaded = await tauriApi.getSettings();
      if (loaded) get().applySettings({ ...DEFAULT_SETTINGS, ...loaded });
    } catch {
    }
  },

  applySettings: (next) => {
    const merged: AppSettings = { ...DEFAULT_SETTINGS, ...next };
    set((s) => ({
      settings: merged,
      githubRange:
        merged.github_default_range !== s.settings.github_default_range
          ? merged.github_default_range
          : s.githubRange,
      chatProvider: merged.chat_default_provider,
      chatModel: merged.chat_default_model,
      calendarAiProvider: merged.calendar_ai_provider || s.calendarAiProvider,
      calendarAiModel: merged.calendar_ai_model || s.calendarAiModel,
      pomoSecondsLeft: s.pomoRunning
        ? s.pomoSecondsLeft
        : (s.pomoMode === "focus" ? merged.pomo_focus_mins : merged.pomo_break_mins) * 60,
    }));
  },

  updateSettings: async (partial) => {
    const merged: AppSettings = { ...get().settings, ...partial };
    get().applySettings(merged);
    if (isTauriRuntime()) {
      await tauriApi.saveSettings(merged).catch(() => {});
    }
  },

  resetAllSettings: async () => {
    get().applySettings(DEFAULT_SETTINGS);
    if (isTauriRuntime()) {
      await tauriApi.resetSettings().catch(() => {});
    }
  },

  refreshLocalDrop: async () => {
    if (!isTauriRuntime()) return;
    try {
      const st = await tauriApi.getLocalDropStatus();
      if (st) set({ localDrop: st });
    } catch {
    }
  },

  toggleLocalDropServer: async () => {
    if (!isTauriRuntime()) return;
    const running = get().localDrop.running;
    set({ localDropLoading: true });
    try {
      const st = running ? await tauriApi.stopLocalDrop() : await tauriApi.startLocalDrop();
      if (st) set({ localDrop: st, localDropLoading: false });
      else set({ localDropLoading: false });
    } catch {
      set({ localDropLoading: false });
    }
  },

  removeLocalDropFile: async (id) => {
    if (!isTauriRuntime()) return;
    try {
      const st = await tauriApi.removeLocalDropFile(id);
      if (st) set({ localDrop: st });
    } catch {
    }
  },

  setLocalDropClients: (count) =>
    set((s) => ({ localDrop: { ...s.localDrop, connected_clients: count } })),

  pushReceivedDropItem: (item) => {
    const { settings } = get();
    playGraviSound("chime", settings.sound_enabled, settings.sound_volume);
    set((s) => ({
      localDrop: {
        ...s.localDrop,
        received_items: [item, ...s.localDrop.received_items.filter((x) => x.id !== item.id)].slice(0, 20),
      },
      activeHud: {
        kind: "ai_done",
        title: item.kind === "text" ? "Teks Diterima dari HP" : "File Diterima dari HP",
        subtitle: item.kind === "text" ? item.content_or_path : item.name,
        mood: "celebrate",
        accent: "emerald",
        badge: "Local Drop",
      },
      mood: "celebrate",
    }));
  },

  refreshHardwareStatus: async () => {
    if (!isTauriRuntime()) return;
    set({ hardwareLoading: true });
    try {
      const st = await tauriApi.getHardwareStatus();
      if (st) set({ hardwareStatus: st, hardwareLoading: false });
      else set({ hardwareLoading: false });
    } catch {
      set({ hardwareLoading: false });
    }
  },

  addFocusHabit: (text) => {
    const next: FocusHabitItem[] = [
      ...get().focusHabits,
      { id: `h-${Date.now()}`, text, done: false },
    ];
    saveTodayHabits(next);
    set({ focusHabits: next, focusHabitAlarmFiredDate: null });
  },

  toggleFocusHabit: (id) => {
    const { settings } = get();
    const next = get().focusHabits.map((h) => (h.id === id ? { ...h, done: !h.done } : h));
    saveTodayHabits(next);
    const allDone = next.length > 0 && next.every((h) => h.done);
    if (allDone) {
      playGraviSound("chime", settings.sound_enabled, settings.sound_volume);
      set({ focusHabits: next, mood: "celebrate" });
      setTimeout(() => {
        if (get().mood === "celebrate") set({ mood: "idle" });
      }, 2200);
    } else {
      playGraviSound("pop", settings.sound_enabled, settings.sound_volume);
      set({ focusHabits: next });
    }
  },

  removeFocusHabit: (id) => {
    const next = get().focusHabits.filter((h) => h.id !== id);
    saveTodayHabits(next);
    set({ focusHabits: next });
  },

  checkFocusHabitAlarm: () => {
    const { focusHabits, settings, focusHabitAlarmFiredDate } = get();
    const unfinished = focusHabits.filter((h) => !h.done);
    if (unfinished.length === 0) return;

    const nowDate = new Date();
    const todayIso = toLocalIsoDate(nowDate);
    if (focusHabitAlarmFiredDate === todayIso) return;

    const dl = settings.focus_habit_deadline || "21:00";
    const m = /^(\d{2}):(\d{2})/.exec(dl);
    if (!m) return;
    const deadlineMins = Number(m[1]) * 60 + Number(m[2]);
    const nowMins = nowDate.getHours() * 60 + nowDate.getMinutes();
    const warnMins = Math.max(5, settings.focus_habit_warn_mins || 30);

    if (nowMins >= deadlineMins - warnMins) {
      playGraviSound("alert", settings.sound_enabled, settings.sound_volume);
      set({
        focusHabitAlarmFiredDate: todayIso,
        mood: "panic",
        activeHud: {
          kind: "habit_alarm",
          title: "Target Hari Ini Belum Selesai!",
          subtitle: `Sisa ${unfinished.length} target sebelum jam ${dl}`,
          mood: "panic",
          accent: "rose",
          badge: `${unfinished.length} sisa`,
        },
      });
    }
  },
}));
