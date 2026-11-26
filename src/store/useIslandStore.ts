import { create } from "zustand";
import {
  AiSession,
  CalendarEventItem,
  ChatMessage,
  GitHubSummary,
  IslandMode,
  MediaState,
  NobiMood,
  PermissionRequest,
  PillTab,
  AppSettings,
  SecretStatus,
  SystemStats,
} from "../types/island";
import {
  buildDemoCalendarEvents,
  buildDemoGitHub,
  DEMO_AI_SESSIONS,
  DEMO_MEDIA,
  DEMO_PERMISSIONS,
} from "../lib/demoData";
import { isTauriRuntime, tauriApi } from "../lib/tauriBridge";


export const DEFAULT_SETTINGS: AppSettings = {
  island_position: "top-center",
  auto_hide_idle: false,
  clock_24h: true,
  show_seconds: false,
  start_on_boot: false,
  island_opacity: 96,
  demo_mode: !isTauriRuntime(),
  eye_tracking: true,
  ear_animations: true,
  squish_dizzy: true,
  sound_enabled: true,
  sound_volume: 70,
  monitored_ais: [
    "claude-desktop", "gemini-desktop", "chatgpt-desktop", "antigravity-desktop",
    "hermes-desktop", "openbot-desktop", "antigravity-cli", "copilot-cli",
  ],
  max_compact_logos: 4,
  poll_interval_secs: 3,
  github_default_range: 3,
  github_default_items: 3,
  github_refresh_mins: 10,
  calendar_reminder_mins: 15,
  calendar_week_start: "monday",
  chat_default_provider: "ollama",
  chat_default_model: "llama3.2",
  ollama_endpoint: "http://127.0.0.1:11434",
  chat_system_prompt: "Kamu adalah Nobi, asisten developer yang ringkas dan membantu.",
  max_drop_file_kb: 256,
  pomo_focus_mins: 25,
  pomo_break_mins: 5,
  media_sync_enabled: true,
  media_dance_enabled: true,
  shortcut_toggle_island: "Alt+Space",
  shortcut_open_chat: "Alt+C",
};

interface IslandStore {
  mode: IslandMode;
  activeTab: PillTab;
  mood: NobiMood;
  demoMode: boolean;

  // AI monitor & approvals
  aiSessions: AiSession[];
  permissions: PermissionRequest[];

  // GitHub
  githubRange: 3 | 6 | 12;
  githubData: GitHubSummary | null;
  githubLoading: boolean;
  githubError: string | null;

  // Google Calendar
  calYear: number;
  calMonth: number; // 1..12
  selectedDate: string; // YYYY-MM-DD
  calendarEvents: CalendarEventItem[];
  calendarLoading: boolean;
  calendarError: string | null;

  // Media
  media: MediaState;

  // Pomodoro
  pomoMode: "focus" | "break";
  pomoRunning: boolean;
  pomoSecondsLeft: number;

  // Chat & File Drop
  chatProvider: "ollama" | "openai" | "anthropic" | "gemini";
  chatModel: string;
  chatMessages: ChatMessage[];
  chatLoading: boolean;
  droppedFiles: string[];
  isDraggingFile: boolean;

  // Secrets status
  systemStats: SystemStats;
  cpuHistory: number[];
  settings: AppSettings;
  secretStatus: SecretStatus;

  // Actions
  setMode: (mode: IslandMode) => void;
  setActiveTab: (tab: PillTab) => void;
  setMood: (mood: NobiMood) => void;
  toggleDemoMode: () => void;

  setSystemStats: (stats: SystemStats) => void;
  setAiSessions: (sessions: AiSession[]) => void;
  pushPermission: (req: PermissionRequest) => void;
  resolvePermission: (id: string, decision: "allow" | "deny" | "always") => Promise<void>;

  setGitHubRange: (range: 3 | 6 | 12) => Promise<void>;
  refreshGitHub: () => Promise<void>;

  setCalendarMonth: (year: number, month: number) => Promise<void>;
  setSelectedDate: (date: string) => void;
  refreshCalendar: () => Promise<void>;

  toggleMediaPlay: () => Promise<void>;
  skipMedia: (dir: "next" | "previous") => Promise<void>;
  refreshMedia: () => Promise<void>;

  togglePomodoro: () => void;
  resetPomodoro: (mode?: "focus" | "break") => void;
  tickPomodoro: () => void;

  setChatProvider: (provider: "ollama" | "openai" | "anthropic" | "gemini") => void;
  setChatModel: (model: string) => void;
  sendChat: (prompt: string) => Promise<void>;

  addDroppedFiles: (paths: string[]) => void;
  removeDroppedFile: (path: string) => void;
  clearDroppedFiles: () => void;
  setDraggingFile: (dragging: boolean) => void;

  refreshSecretStatus: () => Promise<void>;
  loadSettings: () => Promise<void>;
  applySettings: (next: AppSettings) => void;
  updateSettings: (partial: Partial<AppSettings>) => Promise<void>;
  resetAllSettings: () => Promise<void>;
}

const now = new Date();
const todayIso = now.toISOString().slice(0, 10);

export const useIslandStore = create<IslandStore>((set, get) => ({
  mode: "compact",
  activeTab: "agents",
  mood: "idle",
  demoMode: !isTauriRuntime(),

  aiSessions: DEMO_AI_SESSIONS,
  permissions: DEMO_PERMISSIONS,

  githubRange: 3, // Default 3 bulan sesuai permintaan
  githubData: buildDemoGitHub(3),
  githubLoading: false,
  githubError: null,

  calYear: now.getFullYear(),
  calMonth: now.getMonth() + 1,
  selectedDate: todayIso,
  calendarEvents: buildDemoCalendarEvents(now.getFullYear(), now.getMonth() + 1),
  calendarLoading: false,
  calendarError: null,

  media: DEMO_MEDIA,

  pomoMode: "focus",
  pomoRunning: false,
  pomoSecondsLeft: 25 * 60,

  chatProvider: "ollama",
  chatModel: "llama3.2",
  chatMessages: [
    {
      id: "welcome-1",
      role: "assistant",
      provider: "nobi",
      content:
        "Halo! Aku Nobi. Kamu bisa tanya apa saja, lampirkan file lewat drag & drop ke island, atau ganti model AI di atas.",
      timestamp: Date.now(),
    },
  ],
  chatLoading: false,
  droppedFiles: [],
  isDraggingFile: false,

  systemStats: {
    cpu_percent: 28,
    cpu_cores: 12,
    ram_used_gb: 9.4,
    ram_total_gb: 16,
    ram_percent: 58.7,
    uptime_secs: 14235,
    gpu: {
      name: "NVIDIA GeForce RTX 4060",
      usage_percent: 22,
      vram_used_mb: 2450,
      vram_total_mb: 8192,
      temp_c: 48,
    },
    top_processes: [
      { pid: 1402, name: "Antigravity.exe", cpu_percent: 6.4, memory_mb: 615 },
      { pid: 2218, name: "Claude.exe", cpu_percent: 4.1, memory_mb: 482 },
      { pid: 4110, name: "Code.exe", cpu_percent: 3.5, memory_mb: 440 },
      { pid: 9012, name: "firefox.exe", cpu_percent: 2.8, memory_mb: 390 },
    ],
  },
  cpuHistory: [18, 24, 21, 30, 27, 34, 29, 26, 31, 28],
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

  toggleDemoMode: () => {
    const next = !get().demoMode;
    set({ demoMode: next });
    if (next) {
      set({
        aiSessions: DEMO_AI_SESSIONS,
        permissions: DEMO_PERMISSIONS,
        githubData: buildDemoGitHub(get().githubRange),
        githubError: null,
        calendarEvents: buildDemoCalendarEvents(get().calYear, get().calMonth),
        calendarError: null,
      });
    } else {
      void get().refreshGitHub();
      void get().refreshCalendar();
      void get().refreshMedia();
    }
  },

  setSystemStats: (systemStats) => {
    set((s) => ({
      systemStats,
      cpuHistory: [...s.cpuHistory.slice(-17), Math.round(systemStats.cpu_percent)],
    }));
  },

  setAiSessions: (aiSessions) => {
    if (!get().demoMode) {
      set({ aiSessions });
    }
  },

  pushPermission: (req) => {
    set((s) => ({
      permissions: [req, ...s.permissions.filter((p) => p.id !== req.id)],
      mood: "alert",
      mode: "expanded",
      activeTab: "agents",
    }));
  },

  resolvePermission: async (id, decision) => {
    if (!get().demoMode && isTauriRuntime()) {
      await tauriApi.respondPermission(id, decision).catch(() => {});
    }
    set((s) => {
      const remaining = s.permissions.filter((p) => p.id !== id);
      return {
        permissions: remaining,
        mood: decision === "deny" ? "squish" : "happy",
      };
    });
    setTimeout(() => {
      if (get().permissions.length === 0) {
        set({ mood: get().media.is_playing ? "dancing" : "idle" });
      }
    }, 1600);
  },

  setGitHubRange: async (range) => {
    set({ githubRange: range });
    await get().refreshGitHub();
  },

  refreshGitHub: async () => {
    const { demoMode, githubRange } = get();
    if (demoMode || !isTauriRuntime()) {
      set({ githubData: buildDemoGitHub(githubRange), githubError: null, githubLoading: false });
      return;
    }
    set({ githubLoading: true, githubError: null });
    try {
      const data = await tauriApi.fetchGitHubSummary(githubRange);
      if (data) set({ githubData: data, githubLoading: false });
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
    const { demoMode, calYear, calMonth } = get();
    if (demoMode || !isTauriRuntime()) {
      set({
        calendarEvents: buildDemoCalendarEvents(calYear, calMonth),
        calendarError: null,
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

  toggleMediaPlay: async () => {
    const nextPlaying = !get().media.is_playing;
    if (!get().demoMode && isTauriRuntime()) {
      await tauriApi.mediaControl("play_pause").catch(() => {});
    }
    set((s) => ({
      media: { ...s.media, is_playing: nextPlaying },
      mood: nextPlaying ? "dancing" : "idle",
    }));
  },

  skipMedia: async (dir) => {
    if (!get().demoMode && isTauriRuntime()) {
      await tauriApi.mediaControl(dir).catch(() => {});
      await get().refreshMedia();
      return;
    }
    const tracks = [
      { title: "Midnight City (Lo-Fi Coding Mix)", artist: "Synthwave Focus Radio" },
      { title: "Neon Rain Over Tokyo", artist: "Ambient Dev Beats" },
      { title: "Rust & Coffee", artist: "Deep Work Collective" },
    ];
    const idx = Math.floor(Math.random() * tracks.length);
    set((s) => ({
      media: { ...s.media, ...tracks[idx], position_secs: 0, is_playing: true },
      mood: "dancing",
    }));
  },

  refreshMedia: async () => {
    if (get().demoMode || !isTauriRuntime()) return;
    try {
      const state = await tauriApi.getMediaState();
      if (state) {
        set({ media: state, mood: state.is_playing ? "dancing" : get().mood });
      }
    } catch {
      // keep existing state when no media session active
    }
  },

  togglePomodoro: () => {
    set((s) => {
      const running = !s.pomoRunning;
      return {
        pomoRunning: running,
        mood: running ? "focus" : "idle",
      };
    });
  },

  resetPomodoro: (mode) => {
    const target = mode ?? get().pomoMode;
    set({
      pomoMode: target,
      pomoRunning: false,
      pomoSecondsLeft: target === "focus" ? 25 * 60 : 5 * 60,
      mood: "idle",
    });
  },

  tickPomodoro: () => {
    const { pomoRunning, pomoSecondsLeft, pomoMode } = get();
    if (!pomoRunning) return;
    if (pomoSecondsLeft <= 1) {
      const nextMode = pomoMode === "focus" ? "break" : "focus";
      set({
        pomoMode: nextMode,
        pomoRunning: false,
        pomoSecondsLeft: nextMode === "focus" ? 25 * 60 : 5 * 60,
        mood: "happy",
      });
    } else {
      set({ pomoSecondsLeft: pomoSecondsLeft - 1 });
    }
  },

  setChatProvider: (chatProvider) => {
    const defaultModels: Record<string, string> = {
      ollama: "llama3.2",
      openai: "gpt-4o-mini",
      anthropic: "claude-3-5-sonnet-latest",
      gemini: "gemini-2.5-flash",
    };
    set({ chatProvider, chatModel: defaultModels[chatProvider] });
  },

  setChatModel: (chatModel) => set({ chatModel }),

  sendChat: async (prompt) => {
    const trimmed = prompt.trim();
    if (!trimmed) return;
    const { chatProvider, chatModel, droppedFiles, demoMode } = get();

    const userMsg: ChatMessage = {
      id: `u-${Date.now()}`,
      role: "user",
      content: trimmed,
      provider: chatProvider,
      files: droppedFiles.length > 0 ? [...droppedFiles] : undefined,
      timestamp: Date.now(),
    };

    set((s) => ({
      chatMessages: [...s.chatMessages, userMsg],
      chatLoading: true,
      droppedFiles: [],
    }));

    if (demoMode || !isTauriRuntime()) {
      setTimeout(() => {
        const reply: ChatMessage = {
          id: `a-${Date.now()}`,
          role: "assistant",
          provider: chatProvider,
          content: `[Mode Demo · ${chatProvider}/${chatModel}] Pesanmu diterima: "${trimmed}".${
            userMsg.files ? ` (${userMsg.files.length} file terlampir terbaca).` : ""
          } Matikan Demo Mode & isi API Key/Ollama di Pengaturan untuk respons live.`,
          timestamp: Date.now(),
        };
        set((s) => ({
          chatMessages: [...s.chatMessages, reply],
          chatLoading: false,
          mood: "happy",
        }));
      }, 650);
      return;
    }

    try {
      const answer = await tauriApi.sendChatMessage({
        provider: chatProvider,
        model: chatModel,
        prompt: trimmed,
        attached_files: userMsg.files ?? [],
      });
      set((s) => ({
        chatMessages: [
          ...s.chatMessages,
          {
            id: `a-${Date.now()}`,
            role: "assistant",
            provider: chatProvider,
            content: answer ?? "(Tidak ada respons)",
            timestamp: Date.now(),
          },
        ],
        chatLoading: false,
      }));
    } catch (err) {
      set((s) => ({
        chatMessages: [
          ...s.chatMessages,
          {
            id: `err-${Date.now()}`,
            role: "assistant",
            provider: chatProvider,
            content: `Gagal menghubungi ${chatProvider}: ${String(err)}`,
            timestamp: Date.now(),
          },
        ],
        chatLoading: false,
      }));
    }
  },

  addDroppedFiles: (paths) => {
    set((s) => ({
      droppedFiles: Array.from(new Set([...s.droppedFiles, ...paths])),
      isDraggingFile: false,
      mood: "happy",
      mode: "expanded",
      activeTab: "chat",
    }));
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
    if (!isTauriRuntime()) return;
    try {
      const st = await tauriApi.getSecretStatus();
      if (st) set({ secretStatus: st });
    } catch {
      // ignore
    }
  },

  loadSettings: async () => {
    if (!isTauriRuntime()) return;
    try {
      const loaded = await tauriApi.getSettings();
      if (loaded) get().applySettings(loaded);
    } catch {
      // keep defaults
    }
  },

  applySettings: (next) => {
    set((s) => ({
      settings: next,
      demoMode: next.demo_mode,
      githubRange: next.github_default_range,
      chatProvider: next.chat_default_provider,
      chatModel: next.chat_default_model,
      pomoSecondsLeft: s.pomoRunning
        ? s.pomoSecondsLeft
        : (s.pomoMode === "focus" ? next.pomo_focus_mins : next.pomo_break_mins) * 60,
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
}));
