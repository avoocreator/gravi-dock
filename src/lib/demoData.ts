import {
  AiSession,
  CalendarEventItem,
  ContributionDay,
  GitHubSummary,
  MediaState,
  PermissionRequest,
} from "../types/island";

export const DEMO_AI_SESSIONS: AiSession[] = [
  {
    id: "antigravity-cli",
    name: "Antigravity CLI",
    kind: "cli",
    provider: "antigravity",
    status: "waiting",
    pid: 18420,
    cpu_usage: 4.8,
    memory_mb: 164,
    detail: "Menunggu izin jalankan cargo test",
    started_secs: 740,
  },
  {
    id: "copilot-cli",
    name: "GitHub Copilot CLI",
    kind: "cli",
    provider: "copilot",
    status: "active",
    pid: 21044,
    cpu_usage: 2.3,
    memory_mb: 112,
    detail: "Menganalisis diff src-tauri/src/github.rs",
    started_secs: 320,
  },
  {
    id: "claude-desktop",
    name: "Claude Desktop",
    kind: "desktop",
    provider: "claude",
    status: "active",
    pid: 9012,
    cpu_usage: 3.1,
    memory_mb: 482,
    detail: "Aplikasi desktop aktif",
    started_secs: 4120,
  },
  {
    id: "gemini-desktop",
    name: "Gemini Desktop",
    kind: "desktop",
    provider: "gemini",
    status: "idle",
    pid: 11320,
    cpu_usage: 0.4,
    memory_mb: 310,
    detail: "Berjalan di latar belakang",
    started_secs: 2800,
  },
  {
    id: "chatgpt-desktop",
    name: "ChatGPT Desktop",
    kind: "desktop",
    provider: "chatgpt",
    status: "idle",
    pid: 14210,
    cpu_usage: 0.6,
    memory_mb: 365,
    detail: "Berjalan di latar belakang",
    started_secs: 1950,
  },
  {
    id: "antigravity-desktop",
    name: "Antigravity Desktop",
    kind: "desktop",
    provider: "antigravity",
    status: "active",
    pid: 15800,
    cpu_usage: 5.2,
    memory_mb: 615,
    detail: "Workspace nobi-island terbuka",
    started_secs: 5400,
  },
  {
    id: "hermes-desktop",
    name: "Hermes",
    kind: "desktop",
    provider: "hermes",
    status: "idle",
    pid: 16210,
    cpu_usage: 0.3,
    memory_mb: 210,
    detail: "Berjalan di latar belakang",
    started_secs: 1500,
  },
  {
    id: "openbot-desktop",
    name: "OpenBot",
    kind: "desktop",
    provider: "openbot",
    status: "idle",
    pid: 16904,
    cpu_usage: 0.4,
    memory_mb: 190,
    detail: "Berjalan di latar belakang",
    started_secs: 1200,
  },
];

export const DEMO_PERMISSIONS: PermissionRequest[] = [
  {
    id: "demo-perm-1",
    agent: "Antigravity CLI",
    tool: "shell",
    command: "cargo test --manifest-path src-tauri/Cargo.toml",
    cwd: "~/projects/nobi-island",
    timestamp: Date.now() - 12000,
  },
];

export function buildDemoGitHub(rangeMonths: 3 | 6 | 12): GitHubSummary {
  const daysCount = rangeMonths === 3 ? 91 : rangeMonths === 6 ? 182 : 364;
  const today = new Date();
  const days: ContributionDay[] = [];
  let total = 0;

  for (let i = daysCount - 1; i >= 0; i--) {
    const d = new Date(today);
    d.setDate(today.getDate() - i);
    const seed = (d.getDate() * 17 + (d.getMonth() + 1) * 31 + i * 7) % 13;
    const count = seed < 3 ? 0 : seed < 6 ? 2 : seed < 9 ? 4 : seed < 11 ? 7 : 11;
    const level = (count === 0 ? 0 : count <= 2 ? 1 : count <= 5 ? 2 : count <= 8 ? 3 : 4) as
      | 0
      | 1
      | 2
      | 3
      | 4;
    total += count;
    days.push({
      date: d.toISOString().slice(0, 10),
      count,
      level,
    });
  }

  return {
    username: "demo-dev",
    range_months: rangeMonths,
    total_contributions: total,
    days,
    repos: [
      {
        name: "nobi-island",
        full_name: "demo-dev/nobi-island",
        language: "Rust",
        language_color: "#dea584",
        updated_at: new Date(Date.now() - 1000 * 60 * 18).toISOString(),
        url: "https://github.com",
        stars: 42,
      },
      {
        name: "agent-hook-kit",
        full_name: "demo-dev/agent-hook-kit",
        language: "TypeScript",
        language_color: "#3178c6",
        updated_at: new Date(Date.now() - 1000 * 60 * 60 * 5).toISOString(),
        url: "https://github.com",
        stars: 19,
      },
      {
        name: "dotfiles-hyprland",
        full_name: "demo-dev/dotfiles-hyprland",
        language: "Shell",
        language_color: "#89e051",
        updated_at: new Date(Date.now() - 1000 * 60 * 60 * 28).toISOString(),
        url: "https://github.com",
        stars: 11,
      },
    ],
    commits: [
      {
        sha: "a4f91c2",
        message: "feat(canvas): tambah animasi telinga gradien & eye-tracking",
        repo: "nobi-island",
        committed_at: new Date(Date.now() - 1000 * 60 * 18).toISOString(),
        url: "https://github.com",
      },
      {
        sha: "8b3e0d1",
        message: "feat(github): dukungan grafik kontribusi 3/6/12 bulan",
        repo: "nobi-island",
        committed_at: new Date(Date.now() - 1000 * 60 * 75).toISOString(),
        url: "https://github.com",
      },
      {
        sha: "3c12f7e",
        message: "fix(ipc): pastikan nobi-hook fail-open saat island tutup",
        repo: "agent-hook-kit",
        committed_at: new Date(Date.now() - 1000 * 60 * 240).toISOString(),
        url: "https://github.com",
      },
      {
        sha: "e91a50b",
        message: "chore(calendar): parser iCal VEVENT ringan di Rust",
        repo: "nobi-island",
        committed_at: new Date(Date.now() - 1000 * 60 * 480).toISOString(),
        url: "https://github.com",
      },
    ],
  };
}

export function buildDemoCalendarEvents(year: number, month: number): CalendarEventItem[] {
  const pad = (n: number) => String(n).padStart(2, "0");
  const todayDay = new Date().getDate();
  const clamp = (d: number) => Math.max(1, Math.min(28, d));
  const ym = `${year}-${pad(month)}`;

  return [
    {
      id: "cal-1",
      title: "Daily Standup Engineering",
      date: `${ym}-${pad(clamp(todayDay))}`,
      start_time: "09:30",
      end_time: "10:00",
      location: "Google Meet",
      meeting_url: "https://meet.google.com",
      source: "demo",
    },
    {
      id: "cal-2",
      title: "Review PR Nobi Dynamic Island",
      date: `${ym}-${pad(clamp(todayDay))}`,
      start_time: "14:00",
      end_time: "15:00",
      location: "Discord Dev Channel",
      meeting_url: null,
      source: "demo",
    },
    {
      id: "cal-3",
      title: "Sprint Planning Q4",
      date: `${ym}-${pad(clamp(todayDay + 2))}`,
      start_time: "10:30",
      end_time: "11:30",
      location: "Google Meet",
      meeting_url: "https://meet.google.com",
      source: "demo",
    },
    {
      id: "cal-4",
      title: "Demo Prototype ke Tim",
      date: `${ym}-${pad(clamp(todayDay + 4))}`,
      start_time: "16:00",
      end_time: "17:00",
      location: "Ruang Rapat / Online",
      meeting_url: null,
      source: "demo",
    },
  ];
}

export const DEMO_MEDIA: MediaState = {
  title: "Midnight City (Lo-Fi Coding Mix)",
  artist: "Synthwave Focus Radio",
  is_playing: false,
  position_secs: 64,
  duration_secs: 218,
  player: "Demo Player",
};
