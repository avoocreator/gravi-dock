export type IslandMode = "hidden" | "compact" | "expanded";

export type PillTab =
  | "agents"
  | "github"
  | "calendar"
  | "chat"
  | "music"
  | "pomodoro"
  | "system"
  | "settings";

export type NobiMood =
  | "idle"
  | "happy"
  | "squish"
  | "dizzy"
  | "alert"
  | "dancing"
  | "focus"
  | "drop";

export interface AiSession {
  id: string;
  name: string;
  kind: "desktop" | "cli";
  provider: "claude" | "gemini" | "chatgpt" | "antigravity" | "copilot" | "hermes" | "openbot";
  status: "active" | "idle" | "waiting";
  pid: number;
  cpu_usage: number;
  memory_mb: number;
  detail: string;
  started_secs: number;
}

export interface PermissionRequest {
  id: string;
  agent: string;
  tool: string;
  command: string;
  cwd?: string;
  timestamp: number;
}

export interface ContributionDay {
  date: string;
  count: number;
  level: 0 | 1 | 2 | 3 | 4;
}

export interface RepoItem {
  name: string;
  full_name: string;
  language?: string | null;
  language_color?: string | null;
  updated_at: string;
  url: string;
  stars: number;
}

export interface CommitItem {
  sha: string;
  message: string;
  repo: string;
  committed_at: string;
  url: string;
}

export interface GitHubSummary {
  username: string;
  range_months: 3 | 6 | 12;
  total_contributions: number;
  days: ContributionDay[];
  repos: RepoItem[];
  commits: CommitItem[];
}

export interface CalendarEventItem {
  id: string;
  title: string;
  date: string;
  start_time: string;
  end_time: string;
  location?: string | null;
  meeting_url?: string | null;
  source: string;
}

export interface MediaState {
  title: string;
  artist: string;
  is_playing: boolean;
  position_secs: number;
  duration_secs: number;
  player: string;
}

export interface ChatMessage {
  id: string;
  role: "user" | "assistant";
  content: string;
  provider: string;
  files?: string[];
  timestamp: number;
}

export interface SecretStatus {
  github_token: boolean;
  google_ical_url: boolean;
  google_access_token: boolean;
  openai_api_key: boolean;
  anthropic_api_key: boolean;
  gemini_api_key: boolean;
}

export interface AppSettings {
  island_position: "top-center" | "top-left" | "top-right";
  auto_hide_idle: boolean;
  clock_24h: boolean;
  show_seconds: boolean;
  start_on_boot: boolean;
  island_opacity: number;
  demo_mode: boolean;

  eye_tracking: boolean;
  ear_animations: boolean;
  squish_dizzy: boolean;
  sound_enabled: boolean;
  sound_volume: number;

  monitored_ais: string[];
  max_compact_logos: number;
  poll_interval_secs: number;

  github_default_range: 3 | 6 | 12;
  github_default_items: number;
  github_refresh_mins: number;

  calendar_reminder_mins: number;
  calendar_week_start: "monday" | "sunday";

  chat_default_provider: "ollama" | "openai" | "anthropic" | "gemini";
  chat_default_model: string;
  ollama_endpoint: string;
  chat_system_prompt: string;
  max_drop_file_kb: number;

  pomo_focus_mins: number;
  pomo_break_mins: number;
  media_sync_enabled: boolean;
  media_dance_enabled: boolean;
  shortcut_toggle_island: string;
  shortcut_open_chat: string;
}

export interface GpuInfo {
  name: string;
  usage_percent: number;
  vram_used_mb: number;
  vram_total_mb: number;
  temp_c?: number | null;
}

export interface TopProcess {
  pid: number;
  name: string;
  cpu_percent: number;
  memory_mb: number;
}

export interface SystemStats {
  cpu_percent: number;
  cpu_cores: number;
  ram_used_gb: number;
  ram_total_gb: number;
  ram_percent: number;
  uptime_secs: number;
  gpu: GpuInfo;
  top_processes: TopProcess[];
}
