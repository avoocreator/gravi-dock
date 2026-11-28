export type IslandMode = "hidden" | "compact" | "expanded";

export type PillTab =
  | "agents"
  | "github"
  | "calendar"
  | "drop"
  | "music"
  | "pomodoro"
  | "system"
  | "settings";

export type GraviMood =
  | "idle"
  | "happy"
  | "love"
  | "wink"
  | "sleepy"
  | "excited"
  | "thinking"
  | "surprised"
  | "cool"
  | "shy"
  | "squish"
  | "dizzy"
  | "alert"
  | "dancing"
  | "focus"
  | "drop"
  | "charging"
  | "low_battery"
  | "battery_full"
  | "wifi"
  | "wifi_off"
  | "capslock"
  | "celebrate"
  | "notif"
  | "panic";

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

export interface DetectedAiProvider {
  id: string;
  name: string;
  installed: boolean;
  running: boolean;
  has_account: boolean;
  source: "local-server" | "app" | "cli" | "api-key" | string;
  status_label: string;
  models: string[];
  default_model: string;
}

export interface HudEventPayload {
  kind: string;
  title: string;
  subtitle: string;
  mood: string;
  accent: string;
  badge?: string | null;
  provider?: string | null;
  app_name?: string | null;
}

export interface OsNotificationItem {
  id: string;
  app_name: string;
  title: string;
  body: string;
  is_messaging: boolean;
  timestamp: number;
}

export interface GroupedOsNotification extends OsNotificationItem {
  count: number;
}

export interface AgentActivityEvent {
  id: string;
  event_type: string;
  agent: string;
  tool?: string | null;
  command?: string | null;
  summary?: string | null;
  file_path?: string | null;
  added?: number | null;
  removed?: number | null;
  cwd?: string | null;
  timestamp: number;
}

export interface CalendarReminderBanner {
  eventId: string;
  title: string;
  startTime: string;
  minsUntil: number;
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
  weekday: number;
}

export interface RepoItem {
  name: string;
  full_name: string;
  language?: string | null;
  language_color?: string | null;
  updated_at: string;
  url: string;
  stars: number;
  is_private?: boolean;
  is_fork?: boolean;
  default_branch?: string | null;
}

export interface CommitItem {
  sha: string;
  message: string;
  repo: string;
  branch?: string;
  committed_at: string;
  url: string;
}

export interface GitHubSummary {
  username: string;
  range_months: 3 | 6 | 12;
  total_contributions: number;
  today_contributions: number;
  current_streak: number;
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
  album: string;
  is_playing: boolean;
  position_secs: number;
  duration_secs: number;
  player: string;
  artwork_url?: string | null;
  has_media: boolean;
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
  island_style: "floating" | "attached";
  open_on_hover: boolean;
  hover_close_delay_ms: number;
  auto_hide_idle: boolean;
  clock_24h: boolean;
  show_seconds: boolean;
  start_on_boot: boolean;
  island_opacity: number;

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
  calendar_ai_provider: string;
  calendar_ai_model: string;

  chat_default_provider: string;
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

  hud_enabled: boolean;
  hud_capslock: boolean;
  hud_battery_charging: boolean;
  hud_battery_low: boolean;
  hud_wifi: boolean;
  hud_ai_done: boolean;
  hud_windows_notif: boolean;
  notif_messaging_only: boolean;
  hud_duration_ms: number;
  hud_bt_low: boolean;
  focus_habit_deadline: string;
  focus_habit_warn_mins: number;
  ambient_auto_focus: boolean;
}

export interface SharedDropFile {
  id: string;
  name: string;
  path: string;
  size_bytes: number;
}

export interface ReceivedDropItem {
  id: string;
  kind: "file" | "text" | string;
  name: string;
  content_or_path: string;
  size_bytes: number;
  timestamp: number;
}

export interface LocalDropStatus {
  running: boolean;
  url: string;
  port: number;
  ip: string;
  connected_clients?: number;
  shared_files: SharedDropFile[];
  received_items: ReceivedDropItem[];
  save_dir: string;
}

export interface LaptopBatteryInfo {
  has_battery: boolean;
  percent: number;
  charging: boolean;
}

export interface BluetoothDeviceInfo {
  id: string;
  name: string;
  connected: boolean;
  battery_percent?: number | null;
  category: "audio" | "input" | "other" | string;
}

export interface NetworkInfo {
  ssid?: string | null;
  local_ip: string;
  down_kbps: number;
  up_kbps: number;
  ping_ms?: number | null;
}

export interface WifiCredentials {
  ssid: string;
  password: string;
}

export interface HardwareStatus {
  laptop_battery: LaptopBatteryInfo;
  bluetooth_devices: BluetoothDeviceInfo[];
  network: NetworkInfo;
}

export interface AudioAppSession {
  app_key: string;
  display_name: string;
  volume_percent: number;
  muted: boolean;
}

export interface FocusHabitItem {
  id: string;
  text: string;
  done: boolean;
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
