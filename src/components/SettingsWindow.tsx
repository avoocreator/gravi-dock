import React, { useEffect, useState } from "react";
import {
  Sliders,
  Sparkles,
  Bot,
  Github,
  CalendarDays,
  MessageSquare,
  Timer,
  Search,
  CheckCircle2,
  RotateCcw,
  Save,
  ScanSearch,
  Bell,
  Copy,
  ExternalLink,
} from "lucide-react";
import { useIslandStore } from "../store/useIslandStore";
import { AppSettings, HookStatus } from "../types/island";
import { isTauriRuntime, tauriApi } from "../lib/tauriBridge";

type SettingsSection =
  | "general"
  | "character"
  | "ai"
  | "github"
  | "calendar"
  | "chat"
  | "hud"
  | "utilities";

const SECTIONS: Array<{ id: SettingsSection; label: string; keywords: string; icon: React.ReactNode }> = [
  { id: "general", label: "Umum & Island", keywords: "posisi floating attached notch menempel mengambang hover buka otomatis jam detik format 24 12 transparansi", icon: <Sliders className="w-4 h-4" /> },
  { id: "character", label: "Karakter & Suara", keywords: "gravi eye tracking telinga squish dizzy ekspresi volume suara", icon: <Sparkles className="w-4 h-4" /> },
  { id: "ai", label: "AI Monitor & Hooks", keywords: "claude gemini chatgpt antigravity copilot hermes openbot ipc hook", icon: <Bot className="w-4 h-4" /> },
  { id: "github", label: "GitHub", keywords: "pat token kontribusi 3b 6b 1t repo commit refresh", icon: <Github className="w-4 h-4" /> },
  { id: "calendar", label: "Google Calendar & AI", keywords: "ical oauth agenda ai provider model pengingat pekan senin minggu", icon: <CalendarDays className="w-4 h-4" /> },
  { id: "chat", label: "Mini Chat & File", keywords: "ollama lmstudio anthropic openai gemini api key prompt drop file", icon: <MessageSquare className="w-4 h-4" /> },
  { id: "hud", label: "HUD & Notifikasi", keywords: "hud capslock baterai charge lowbat wifi selesai ai notifikasi windows whatsapp anti spam", icon: <Bell className="w-4 h-4" /> },
  { id: "utilities", label: "Media, Fokus & Pintasan", keywords: "pomodoro durasi musik menari shortcut keyboard reset", icon: <Timer className="w-4 h-4" /> },
];

const AI_TARGETS = [
  { id: "claude-desktop", label: "Claude Desktop" },
  { id: "gemini-desktop", label: "Gemini Desktop" },
  { id: "chatgpt-desktop", label: "ChatGPT Desktop" },
  { id: "antigravity-desktop", label: "Antigravity Desktop" },
  { id: "hermes-desktop", label: "Hermes Desktop" },
  { id: "openbot-desktop", label: "OpenBot Desktop" },
  { id: "antigravity-cli", label: "Antigravity CLI" },
  { id: "copilot-cli", label: "GitHub Copilot CLI" },
];

export const SettingsWindow: React.FC = () => {
  const {
    settings,
    secretStatus,
    detectedProviders,
    providersLoading,
    updateSettings,
    resetAllSettings,
    refreshSecretStatus,
    refreshAiProviders,
  } = useIslandStore();
  const [section, setSection] = useState<SettingsSection>("general");
  const [search, setSearch] = useState("");
  const [secretsInput, setSecretsInput] = useState<Record<string, string>>({});
  const [statusBanner, setStatusBanner] = useState<string | null>(null);
  const [hookStatus, setHookStatus] = useState<HookStatus | null>(null);
  const [notifAccess, setNotifAccess] = useState<string>("allowed");

  useEffect(() => {
    void refreshSecretStatus();
    void refreshAiProviders();
    if (isTauriRuntime()) {
      void tauriApi.getHookStatus().then((h) => {
        if (h) setHookStatus(h);
      });
      void tauriApi.getNotificationAccessStatus().then((st) => {
        if (st) setNotifAccess(st);
      });
    }
  }, [refreshSecretStatus, refreshAiProviders]);

  const filteredSections = SECTIONS.filter((s) => {
    const q = search.trim().toLowerCase();
    if (!q) return true;
    return s.label.toLowerCase().includes(q) || s.keywords.includes(q);
  });

  const patch = (partial: Partial<AppSettings>) => {
    void updateSettings(partial);
    setStatusBanner("Pengaturan tersinkronisasi real-time ke Gravi");
    setTimeout(() => setStatusBanner(null), 1800);
  };

  const saveKey = async (key: string, label: string) => {
    const val = (secretsInput[key] ?? "").trim();
    if (!val) {
      setStatusBanner(`Masukkan ${label} terlebih dahulu sebelum menyimpan`);
      setTimeout(() => setStatusBanner(null), 2200);
      return;
    }
    try {
      await tauriApi.saveSecret(key, val);
      await refreshSecretStatus();
      await refreshAiProviders();
      setSecretsInput((prev) => ({ ...prev, [key]: "" }));
      setStatusBanner(`${label} tersimpan aman di OS Keyring`);
      setTimeout(() => setStatusBanner(null), 2400);
    } catch (err) {
      setStatusBanner(`Gagal menyimpan ${label}: ${String(err)}`);
    }
  };

  const removeKey = async (key: string, label: string) => {
    try {
      await tauriApi.saveSecret(key, "");
      await refreshSecretStatus();
      await refreshAiProviders();
      setStatusBanner(`${label} dihapus dari OS Keyring`);
      setTimeout(() => setStatusBanner(null), 2400);
    } catch (err) {
      setStatusBanner(`Gagal menghapus ${label}: ${String(err)}`);
    }
  };

  const toggleAiTarget = (id: string) => {
    const has = settings.monitored_ais.includes(id);
    const next = has
      ? settings.monitored_ais.filter((x) => x !== id)
      : [...settings.monitored_ais, id];
    patch({ monitored_ais: next });
  };

  const calProviderObj =
    detectedProviders.find((p) => p.id === settings.calendar_ai_provider) ?? detectedProviders[0];

  return (
    <div className="w-screen h-screen bg-[#0B0B0F] text-zinc-100 flex overflow-hidden select-none">
      {/* Sidebar */}
      <aside className="w-60 bg-[#111117] border-r border-white/[0.08] p-3 flex flex-col gap-3 shrink-0">
        <div className="px-1">
          <div className="text-sm font-bold tracking-tight text-white">Pengaturan Gravi</div>
          <div className="text-[11px] text-zinc-400">Sinkronisasi real-time ke Island</div>
        </div>

        <div className="relative">
          <Search className="w-3.5 h-3.5 text-zinc-500 absolute left-2.5 top-1/2 -translate-y-1/2" />
          <input
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Cari pengaturan..."
            className="w-full bg-white/[0.06] border border-white/[0.08] rounded-xl pl-8 pr-2.5 py-1.5 text-xs text-zinc-100 placeholder:text-zinc-500 outline-none focus:border-blue-400/50"
          />
        </div>

        <nav className="flex-1 space-y-1 overflow-y-auto gravi-scroll">
          {filteredSections.map((s) => (
            <button
              key={s.id}
              onClick={() => setSection(s.id)}
              className={`w-full flex items-center gap-2.5 px-2.5 py-2 rounded-xl text-xs font-medium transition ${
                section === s.id
                  ? "bg-blue-500/20 text-blue-200 border border-blue-400/30"
                  : "text-zinc-400 hover:text-zinc-100 hover:bg-white/[0.05]"
              }`}
            >
              {s.icon}
              <span className="truncate">{s.label}</span>
            </button>
          ))}
        </nav>

        <button
          onClick={() => {
            void resetAllSettings();
            setStatusBanner("Pengaturan dikembalikan ke default");
            setTimeout(() => setStatusBanner(null), 2000);
          }}
          className="w-full flex items-center justify-center gap-1.5 py-2 rounded-xl bg-white/[0.05] hover:bg-rose-500/20 border border-white/[0.08] hover:border-rose-500/30 text-xs text-zinc-300 hover:text-rose-200 transition"
        >
          <RotateCcw className="w-3.5 h-3.5" /> Reset ke Default
        </button>
      </aside>

      {/* Content */}
      <main className="flex-1 p-5 overflow-y-auto gravi-scroll space-y-4">
        {statusBanner && (
          <div className="rounded-xl bg-emerald-500/15 border border-emerald-400/30 px-3 py-2 text-xs text-emerald-200 flex items-center gap-2">
            <CheckCircle2 className="w-4 h-4 shrink-0" />
            <span>{statusBanner}</span>
          </div>
        )}

        {section === "general" && (
          <div className="space-y-3">
            <h2 className="text-base font-semibold text-white">Umum &amp; Tampilan Island</h2>
            <div className="macos-card rounded-2xl p-4 space-y-4 text-xs">
              <div className="space-y-2">
                <div>
                  <div className="font-medium text-white">Gaya Pil Dynamic Island</div>
                  <div className="text-[11px] text-zinc-400">
                    Pilih apakah pil mengambang (floating) atau menempel langsung di tepi atas layar (notch)
                  </div>
                </div>
                <div className="grid grid-cols-2 gap-2.5">
                  <button
                    type="button"
                    onClick={() => {
                      patch({ island_style: "floating" });
                      void tauriApi.recenterIsland(settings.island_position, "floating");
                    }}
                    className={`rounded-xl p-3 text-left border transition space-y-2 ${
                      settings.island_style === "floating"
                        ? "bg-blue-500/15 border-blue-400/50 text-white"
                        : "bg-white/[0.03] border-white/[0.08] text-zinc-400 hover:border-white/20"
                    }`}
                  >
                    <div className="h-7 rounded-lg bg-black/50 border border-white/10 flex items-center justify-center">
                      <div className="w-16 h-3.5 rounded-full bg-zinc-200/80" />
                    </div>
                    <div>
                      <div className="font-semibold text-xs text-white">Mengambang (Floating)</div>
                      <div className="text-[10px] text-zinc-400">Pil melayang dengan jarak tipis dari tepi atas</div>
                    </div>
                  </button>

                  <button
                    type="button"
                    onClick={() => {
                      patch({ island_style: "attached" });
                      void tauriApi.recenterIsland(settings.island_position, "attached");
                    }}
                    className={`rounded-xl p-3 text-left border transition space-y-2 ${
                      settings.island_style === "attached"
                        ? "bg-blue-500/15 border-blue-400/50 text-white"
                        : "bg-white/[0.03] border-white/[0.08] text-zinc-400 hover:border-white/20"
                    }`}
                  >
                    <div className="h-7 rounded-lg bg-black/50 border border-white/10 flex items-start justify-center">
                      <div className="w-16 h-3.5 rounded-b-lg bg-zinc-200/80" />
                    </div>
                    <div>
                      <div className="font-semibold text-xs text-white">Menempel Tepi (Notch)</div>
                      <div className="text-[10px] text-zinc-400">Menyatu rata dengan bingkai atas layar</div>
                    </div>
                  </button>
                </div>
              </div>

              <div className="flex items-center justify-between pt-2 border-t border-white/[0.06]">
                <div>
                  <div className="font-medium text-white">Posisi Dynamic Island</div>
                  <div className="text-[11px] text-zinc-400">Atur letak pil di tepi atas layar</div>
                </div>
                <select
                  value={settings.island_position}
                  onChange={(e) => {
                    const pos = e.target.value as AppSettings["island_position"];
                    patch({ island_position: pos });
                    void tauriApi.recenterIsland(pos, settings.island_style);
                  }}
                  className="bg-zinc-900 border border-white/10 rounded-xl px-2.5 py-1.5 text-xs text-zinc-100 outline-none"
                >
                  <option value="top-center">Atas Tengah</option>
                  <option value="top-left">Atas Kiri</option>
                  <option value="top-right">Atas Kanan</option>
                </select>
              </div>

              <div className="flex items-center justify-between">
                <div>
                  <div className="font-medium text-white">Buka Otomatis saat Hover</div>
                  <div className="text-[11px] text-zinc-400">Island terbuka saat kursor diarahkan dan menutup saat kursor pergi</div>
                </div>
                <input
                  type="checkbox"
                  checked={settings.open_on_hover}
                  onChange={(e) => patch({ open_on_hover: e.target.checked })}
                  className="w-4 h-4 accent-blue-500"
                />
              </div>

              {settings.open_on_hover && (
                <div className="space-y-1 pl-3 border-l-2 border-blue-500/30">
                  <div className="flex justify-between">
                    <span className="font-medium text-white">Durasi Tutup Otomatis saat Hover</span>
                    <span className="font-mono text-zinc-400">
                      {settings.hover_close_delay_ms} ms ({(settings.hover_close_delay_ms / 1000).toFixed(2)} dtk)
                    </span>
                  </div>
                  <input
                    type="range"
                    min={150}
                    max={3000}
                    step={50}
                    value={settings.hover_close_delay_ms}
                    onChange={(e) => patch({ hover_close_delay_ms: Number(e.target.value) })}
                    className="w-full accent-blue-500"
                  />
                </div>
              )}

              <div className="flex items-center justify-between">
                <div>
                  <div className="font-medium text-white">Format Jam 24 Jam</div>
                  <div className="text-[11px] text-zinc-400">Matikan untuk format 12 jam (AM/PM)</div>
                </div>
                <input
                  type="checkbox"
                  checked={settings.clock_24h}
                  onChange={(e) => patch({ clock_24h: e.target.checked })}
                  className="w-4 h-4 accent-blue-500"
                />
              </div>

              <div className="flex items-center justify-between">
                <div>
                  <div className="font-medium text-white">Tampilkan Detik pada Jam</div>
                  <div className="text-[11px] text-zinc-400">Menampilkan detik real-time di pil compact</div>
                </div>
                <input
                  type="checkbox"
                  checked={settings.show_seconds}
                  onChange={(e) => patch({ show_seconds: e.target.checked })}
                  className="w-4 h-4 accent-blue-500"
                />
              </div>

              <div className="flex items-center justify-between">
                <div>
                  <div className="font-medium text-white">Jalankan Otomatis saat Login (Start on Boot)</div>
                  <div className="text-[11px] text-zinc-400">Otomatis aktif saat komputer dinyalakan (Windows/Linux/macOS)</div>
                </div>
                <input
                  type="checkbox"
                  checked={settings.start_on_boot}
                  onChange={(e) => patch({ start_on_boot: e.target.checked })}
                  className="w-4 h-4 accent-blue-500"
                />
              </div>

              <div className="flex items-center justify-between">
                <div>
                  <div className="font-medium text-white">Sembunyikan Otomatis saat Idle</div>
                  <div className="text-[11px] text-zinc-400">Pil mengecil jadi garis tipis setelah 15 detik tidak dipakai</div>
                </div>
                <input
                  type="checkbox"
                  checked={settings.auto_hide_idle}
                  onChange={(e) => patch({ auto_hide_idle: e.target.checked })}
                  className="w-4 h-4 accent-blue-500"
                />
              </div>

              <div className="space-y-1">
                <div className="flex justify-between">
                  <span className="font-medium text-white">Opasitas Latar Island</span>
                  <span className="font-mono text-zinc-400">{settings.island_opacity}%</span>
                </div>
                <input
                  type="range"
                  min={75}
                  max={100}
                  value={settings.island_opacity}
                  onChange={(e) => patch({ island_opacity: Number(e.target.value) })}
                  className="w-full accent-blue-500"
                />
              </div>
            </div>
          </div>
        )}

        {section === "character" && (
          <div className="space-y-3">
            <h2 className="text-base font-semibold text-white">Karakter Gravi &amp; Suara</h2>
            <div className="macos-card rounded-2xl p-4 space-y-3.5 text-xs">
              <div className="flex items-center justify-between">
                <div>
                  <div className="font-medium text-white">Eye-Tracking Kursor Global</div>
                  <div className="text-[11px] text-zinc-400">Mata LED Gravi mengikuti kursor di seluruh layar desktop</div>
                </div>
                <input
                  type="checkbox"
                  checked={settings.eye_tracking}
                  onChange={(e) => patch({ eye_tracking: e.target.checked })}
                  className="w-4 h-4 accent-blue-500"
                />
              </div>
              <div className="flex items-center justify-between">
                <div>
                  <div className="font-medium text-white">Animasi Telinga Gradien</div>
                  <div className="text-[11px] text-zinc-400">Telinga berkedut saat hover, senang &amp; ikut tempo musik</div>
                </div>
                <input
                  type="checkbox"
                  checked={settings.ear_animations}
                  onChange={(e) => patch({ ear_animations: e.target.checked })}
                  className="w-4 h-4 accent-blue-500"
                />
              </div>
              <div className="flex items-center justify-between">
                <div>
                  <div className="font-medium text-white">Interaksi Klik, Elus &amp; Dizzy</div>
                  <div className="text-[11px] text-zinc-400">Klik 1x: reaksi · 2x: love · 3x: pusing · Elus kursor: malu · Klik kanan: ganti ekspresi</div>
                </div>
                <input
                  type="checkbox"
                  checked={settings.squish_dizzy}
                  onChange={(e) => patch({ squish_dizzy: e.target.checked })}
                  className="w-4 h-4 accent-blue-500"
                />
              </div>
              <div className="flex items-center justify-between">
                <div>
                  <div className="font-medium text-white">Efek Suara Interaksi</div>
                  <div className="text-[11px] text-zinc-400">Suara halus untuk notifikasi izin, pomodoro &amp; interaksi Gravi</div>
                </div>
                <input
                  type="checkbox"
                  checked={settings.sound_enabled}
                  onChange={(e) => patch({ sound_enabled: e.target.checked })}
                  className="w-4 h-4 accent-blue-500"
                />
              </div>
              <div className="space-y-1">
                <div className="flex justify-between">
                  <span className="font-medium text-white">Volume Suara</span>
                  <span className="font-mono text-zinc-400">{settings.sound_volume}%</span>
                </div>
                <input
                  type="range"
                  min={0}
                  max={100}
                  value={settings.sound_volume}
                  onChange={(e) => patch({ sound_volume: Number(e.target.value) })}
                  className="w-full accent-blue-500"
                />
              </div>
            </div>
          </div>
        )}

        {section === "ai" && (
          <div className="space-y-3">
            <h2 className="text-base font-semibold text-white">AI Monitor &amp; CLI Hooks</h2>
            <div className="macos-card rounded-2xl p-4 space-y-3 text-xs">
              <div className="font-medium text-white">Aplikasi &amp; CLI yang Dipantau</div>
              <div className="grid grid-cols-2 gap-2">
                {AI_TARGETS.map((ai) => (
                  <label
                    key={ai.id}
                    className="flex items-center gap-2 rounded-xl bg-white/[0.04] border border-white/[0.06] px-3 py-2 cursor-pointer hover:border-white/15"
                  >
                    <input
                      type="checkbox"
                      checked={settings.monitored_ais.includes(ai.id)}
                      onChange={() => toggleAiTarget(ai.id)}
                      className="accent-blue-500"
                    />
                    <span>{ai.label}</span>
                  </label>
                ))}
              </div>
              <div className="flex items-center justify-between pt-2 border-t border-white/[0.06]">
                <span>Maksimal logo AI di Compact Pill</span>
                <select
                  value={settings.max_compact_logos}
                  onChange={(e) => patch({ max_compact_logos: Number(e.target.value) })}
                  className="bg-zinc-900 border border-white/10 rounded-xl px-2.5 py-1 text-xs"
                >
                  {[2, 3, 4, 5, 6].map((n) => (
                    <option key={n} value={n}>{n} logo</option>
                  ))}
                </select>
              </div>
              <div className="flex items-center justify-between">
                <div>
                  <div className="font-medium text-white">Interval Pemindaian Proses AI</div>
                  <div className="text-[11px] text-zinc-400">Seberapa sering proses AI desktop &amp; CLI dicek di latar belakang</div>
                </div>
                <select
                  value={settings.poll_interval_secs}
                  onChange={(e) => patch({ poll_interval_secs: Number(e.target.value) })}
                  className="bg-zinc-900 border border-white/10 rounded-xl px-2.5 py-1 text-xs"
                >
                  {[1, 2, 3, 5, 10].map((n) => (
                    <option key={n} value={n}>{n} detik</option>
                  ))}
                </select>
              </div>
              <div className="rounded-xl bg-black/40 border border-white/[0.06] p-2.5 space-y-1.5 text-[11px]">
                <div className="flex items-center justify-between gap-2">
                  <span className="font-mono text-sky-300">
                    IPC Hook: {hookStatus?.ipc_addr ?? "127.0.0.1:47891"} ({hookStatus?.installed ? "gravi-hook siap" : "gravi-hook belum terpasang"})
                  </span>
                  {hookStatus?.installed && (
                    <button
                      type="button"
                      onClick={() => {
                        void navigator.clipboard.writeText(hookStatus.sample_command);
                        setStatusBanner("Perintah hook disalin ke clipboard");
                        setTimeout(() => setStatusBanner(null), 2000);
                      }}
                      className="px-2 py-0.5 rounded-lg bg-white/[0.08] hover:bg-white/15 text-zinc-200 flex items-center gap-1 text-[10px]"
                    >
                      <Copy className="w-3 h-3" /> Salin
                    </button>
                  )}
                </div>
                {hookStatus && (
                  <div className="font-mono text-[10px] text-zinc-400 break-all">
                    {hookStatus.hook_path}
                  </div>
                )}
              </div>
            </div>
          </div>
        )}

        {section === "github" && (
          <div className="space-y-3">
            <h2 className="text-base font-semibold text-white">Integrasi GitHub</h2>
            <div className="macos-card rounded-2xl p-4 space-y-3.5 text-xs">
              <div className="space-y-1.5">
                <div className="flex justify-between">
                  <span className="font-medium text-white">GitHub Personal Access Token (PAT)</span>
                  {secretStatus.github_token && <span className="text-emerald-400 text-[11px]">Tersimpan</span>}
                </div>
                <div className="flex gap-2">
                  <input
                    type="password"
                    value={secretsInput.github_token ?? ""}
                    onChange={(e) => setSecretsInput((p) => ({ ...p, github_token: e.target.value }))}
                    placeholder="ghp_xxxxxxxxxxxxxxxxxxxx (scope: read:user, repo)"
                    className="flex-1 bg-white/[0.05] border border-white/10 rounded-xl px-3 py-1.5 font-mono text-xs outline-none"
                  />
                  <button
                    onClick={() => void saveKey("github_token", "GitHub PAT")}
                    className="px-3 py-1.5 rounded-xl bg-blue-500 hover:bg-blue-400 text-white font-medium flex items-center gap-1"
                  >
                    <Save className="w-3.5 h-3.5" /> Simpan
                  </button>
                  {secretStatus.github_token && (
                    <button
                      onClick={() => void removeKey("github_token", "GitHub PAT")}
                      className="px-2.5 py-1.5 rounded-xl bg-rose-500/15 hover:bg-rose-500/25 text-rose-300 text-xs"
                    >
                      Hapus
                    </button>
                  )}
                </div>
              </div>

              <div className="flex items-center justify-between">
                <span>Rentang Default Contribution Graph</span>
                <select
                  value={settings.github_default_range}
                  onChange={(e) => patch({ github_default_range: Number(e.target.value) as 3 | 6 | 12 })}
                  className="bg-zinc-900 border border-white/10 rounded-xl px-2.5 py-1 text-xs"
                >
                  <option value={3}>3 Bulan (Default)</option>
                  <option value={6}>6 Bulan</option>
                  <option value={12}>1 Tahun</option>
                </select>
              </div>

              <div className="flex items-center justify-between">
                <span>Jumlah Default Repo &amp; Commit di Island</span>
                <select
                  value={settings.github_default_items}
                  onChange={(e) => patch({ github_default_items: Number(e.target.value) })}
                  className="bg-zinc-900 border border-white/10 rounded-xl px-2.5 py-1 text-xs"
                >
                  <option value={3}>3 item teratas</option>
                  <option value={5}>5 item teratas</option>
                  <option value={10}>10 item teratas</option>
                </select>
              </div>
            </div>
          </div>
        )}

        {section === "calendar" && (
          <div className="space-y-3">
            <h2 className="text-base font-semibold text-white">Integrasi Google Calendar &amp; AI</h2>
            <div className="macos-card rounded-2xl p-4 space-y-3.5 text-xs">
              <div className="space-y-1.5">
                <div className="flex justify-between">
                  <span className="font-medium text-white">Secret iCal URL (Paling Praktis)</span>
                  {secretStatus.google_ical_url && <span className="text-emerald-400 text-[11px]">Tersimpan</span>}
                </div>
                <div className="flex gap-2">
                  <input
                    type="password"
                    value={secretsInput.google_ical_url ?? ""}
                    onChange={(e) => setSecretsInput((p) => ({ ...p, google_ical_url: e.target.value }))}
                    placeholder="https://calendar.google.com/calendar/ical/.../basic.ics"
                    className="flex-1 bg-white/[0.05] border border-white/10 rounded-xl px-3 py-1.5 font-mono text-xs outline-none"
                  />
                  <button
                    onClick={() => void saveKey("google_ical_url", "Google iCal URL")}
                    className="px-3 py-1.5 rounded-xl bg-blue-500 hover:bg-blue-400 text-white font-medium flex items-center gap-1"
                  >
                    <Save className="w-3.5 h-3.5" /> Simpan
                  </button>
                  {secretStatus.google_ical_url && (
                    <button
                      onClick={() => void removeKey("google_ical_url", "Google iCal URL")}
                      className="px-2.5 py-1.5 rounded-xl bg-rose-500/15 hover:bg-rose-500/25 text-rose-300 text-xs"
                    >
                      Hapus
                    </button>
                  )}
                </div>
              </div>

              <div className="space-y-1.5">
                <div className="flex justify-between">
                  <span className="font-medium text-white">Google OAuth Access Token (Opsional)</span>
                  {secretStatus.google_access_token && <span className="text-emerald-400 text-[11px]">Tersimpan</span>}
                </div>
                <div className="flex gap-2">
                  <input
                    type="password"
                    value={secretsInput.google_access_token ?? ""}
                    onChange={(e) => setSecretsInput((p) => ({ ...p, google_access_token: e.target.value }))}
                    placeholder="ya29.a0..."
                    className="flex-1 bg-white/[0.05] border border-white/10 rounded-xl px-3 py-1.5 font-mono text-xs outline-none"
                  />
                  <button
                    onClick={() => void saveKey("google_access_token", "Google OAuth Token")}
                    className="px-3 py-1.5 rounded-xl bg-white/10 hover:bg-white/20 text-white font-medium flex items-center gap-1"
                  >
                    <Save className="w-3.5 h-3.5" /> Simpan
                  </button>
                  {secretStatus.google_access_token && (
                    <button
                      onClick={() => void removeKey("google_access_token", "Google OAuth Token")}
                      className="px-2.5 py-1.5 rounded-xl bg-rose-500/15 hover:bg-rose-500/25 text-rose-300 text-xs"
                    >
                      Hapus
                    </button>
                  )}
                </div>
              </div>

              <div className="flex items-center justify-between">
                <div>
                  <div className="font-medium text-white">Pengingat Sebelum Jadwal</div>
                  <div className="text-[11px] text-zinc-400">Munculkan notifikasi &amp; bunyi di Island sebelum agenda dimulai</div>
                </div>
                <select
                  value={settings.calendar_reminder_mins}
                  onChange={(e) => patch({ calendar_reminder_mins: Number(e.target.value) })}
                  className="bg-zinc-900 border border-white/10 rounded-xl px-2.5 py-1 text-xs"
                >
                  <option value={0}>Nonaktif</option>
                  <option value={5}>5 menit sebelumnya</option>
                  <option value={10}>10 menit sebelumnya</option>
                  <option value={15}>15 menit sebelumnya</option>
                  <option value={30}>30 menit sebelumnya</option>
                  <option value={60}>60 menit sebelumnya</option>
                </select>
              </div>

              <div className="flex items-center justify-between">
                <span>Awal Pekan Kalender</span>
                <select
                  value={settings.calendar_week_start}
                  onChange={(e) => patch({ calendar_week_start: e.target.value as "monday" | "sunday" })}
                  className="bg-zinc-900 border border-white/10 rounded-xl px-2.5 py-1 text-xs"
                >
                  <option value="monday">Senin</option>
                  <option value="sunday">Minggu</option>
                </select>
              </div>

              <div className="pt-2 border-t border-white/[0.06] space-y-2.5">
                <div className="flex items-center justify-between">
                  <div>
                    <div className="font-medium text-white">Asisten AI Kalender (Deteksi Otomatis)</div>
                    <div className="text-[11px] text-zinc-400">Pilih aplikasi/provider AI di laptopmu beserta model akunnya</div>
                  </div>
                  <button
                    type="button"
                    onClick={() => void refreshAiProviders()}
                    className="px-2.5 py-1 rounded-xl bg-white/[0.06] hover:bg-white/15 text-zinc-200 flex items-center gap-1 transition"
                  >
                    <ScanSearch className={`w-3.5 h-3.5 ${providersLoading ? "animate-pulse text-sky-400" : ""}`} />
                    Pindai AI
                  </button>
                </div>

                <div className="grid grid-cols-2 gap-3">
                  <div className="space-y-1">
                    <label className="text-zinc-400">Provider AI</label>
                    <select
                      value={settings.calendar_ai_provider}
                      onChange={(e) => {
                        const prov = detectedProviders.find((p) => p.id === e.target.value);
                        patch({
                          calendar_ai_provider: e.target.value,
                          calendar_ai_model: prov?.default_model ?? settings.calendar_ai_model,
                        });
                      }}
                      className="w-full bg-zinc-900 border border-white/10 rounded-xl px-2.5 py-1.5 text-xs"
                    >
                      {detectedProviders.map((p) => (
                        <option key={p.id} value={p.id}>
                          {p.has_account ? "● " : p.installed ? "◐ " : "○ "}
                          {p.name}
                        </option>
                      ))}
                    </select>
                  </div>

                  <div className="space-y-1">
                    <label className="text-zinc-400">Model</label>
                    <select
                      value={settings.calendar_ai_model}
                      onChange={(e) => patch({ calendar_ai_model: e.target.value })}
                      className="w-full bg-zinc-900 border border-white/10 rounded-xl px-2.5 py-1.5 font-mono text-xs"
                    >
                      {(calProviderObj?.models ?? [settings.calendar_ai_model]).map((m) => (
                        <option key={m} value={m}>
                          {m}
                        </option>
                      ))}
                    </select>
                  </div>
                </div>
              </div>
            </div>
          </div>
        )}

        {section === "chat" && (
          <div className="space-y-3">
            <h2 className="text-base font-semibold text-white">Mini Chat AI &amp; File Drop</h2>
            <div className="macos-card rounded-2xl p-4 space-y-3.5 text-xs">
              <div className="grid grid-cols-2 gap-3">
                <div className="space-y-1">
                  <label className="text-zinc-400">Provider Default</label>
                  <select
                    value={settings.chat_default_provider}
                    onChange={(e) => {
                      const prov = detectedProviders.find((p) => p.id === e.target.value);
                      patch({
                        chat_default_provider: e.target.value,
                        chat_default_model: prov?.default_model ?? settings.chat_default_model,
                      });
                    }}
                    className="w-full bg-zinc-900 border border-white/10 rounded-xl px-2.5 py-1.5 text-xs"
                  >
                    {detectedProviders.map((p) => (
                      <option key={p.id} value={p.id}>
                        {p.has_account ? "● " : p.installed ? "◐ " : "○ "}
                        {p.name}
                      </option>
                    ))}
                  </select>
                </div>
                <div className="space-y-1">
                  <label className="text-zinc-400">Model Default</label>
                  <input
                    value={settings.chat_default_model}
                    onChange={(e) => patch({ chat_default_model: e.target.value })}
                    className="w-full bg-white/[0.05] border border-white/10 rounded-xl px-2.5 py-1.5 font-mono text-xs outline-none"
                  />
                </div>
              </div>

              <div className="space-y-1">
                <label className="text-zinc-400">System Prompt Mini Chat</label>
                <textarea
                  rows={2}
                  value={settings.chat_system_prompt}
                  onChange={(e) => patch({ chat_system_prompt: e.target.value })}
                  placeholder="Instruksi kepribadian atau konteks default untuk Gravi..."
                  className="w-full bg-white/[0.05] border border-white/10 rounded-xl px-3 py-1.5 text-xs outline-none resize-none"
                />
              </div>

              <div className="flex items-center justify-between">
                <div>
                  <div className="font-medium text-white">Batas Ukuran File Drop (KB)</div>
                  <div className="text-[11px] text-zinc-400">Batas maksimal ukuran file lampiran yang dibaca penuh oleh AI</div>
                </div>
                <input
                  type="number"
                  min={16}
                  max={2048}
                  step={32}
                  value={settings.max_drop_file_kb}
                  onChange={(e) =>
                    patch({ max_drop_file_kb: Math.max(16, Math.min(2048, Number(e.target.value) || 256)) })
                  }
                  className="w-24 bg-white/[0.05] border border-white/10 rounded-xl px-2.5 py-1 font-mono text-xs text-right outline-none"
                />
              </div>

              <div className="space-y-1">
                <label className="text-zinc-400">Endpoint Ollama Lokal</label>
                <input
                  value={settings.ollama_endpoint}
                  onChange={(e) => patch({ ollama_endpoint: e.target.value })}
                  placeholder="http://127.0.0.1:11434"
                  className="w-full bg-white/[0.05] border border-white/10 rounded-xl px-3 py-1.5 font-mono text-xs outline-none"
                />
              </div>

              {[
                { key: "anthropic_api_key", label: "Anthropic API Key", ph: "sk-ant-..." },
                { key: "gemini_api_key", label: "Gemini API Key", ph: "AIza..." },
                { key: "openai_api_key", label: "OpenAI API Key", ph: "sk-..." },
              ].map((item) => (
                <div key={item.key} className="space-y-1">
                  <div className="flex justify-between">
                    <span>{item.label}</span>
                    {secretStatus[item.key as keyof typeof secretStatus] && (
                      <span className="text-emerald-400 text-[11px]">Tersimpan</span>
                    )}
                  </div>
                  <div className="flex gap-2">
                    <input
                      type="password"
                      value={secretsInput[item.key] ?? ""}
                      onChange={(e) => setSecretsInput((p) => ({ ...p, [item.key]: e.target.value }))}
                      placeholder={item.ph}
                      className="flex-1 bg-white/[0.05] border border-white/10 rounded-xl px-3 py-1.5 font-mono text-xs outline-none"
                    />
                    <button
                      onClick={() => void saveKey(item.key, item.label)}
                      className="px-3 py-1.5 rounded-xl bg-white/10 hover:bg-white/20 text-white"
                    >
                      Simpan
                    </button>
                    {secretStatus[item.key as keyof typeof secretStatus] && (
                      <button
                        onClick={() => void removeKey(item.key, item.label)}
                        className="px-2.5 py-1.5 rounded-xl bg-rose-500/15 hover:bg-rose-500/25 text-rose-300 text-xs"
                      >
                        Hapus
                      </button>
                    )}
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {section === "hud" && (
          <div className="space-y-3">
            <h2 className="text-base font-semibold text-white">HUD Sistem &amp; Notifikasi Windows</h2>
            <div className="macos-card rounded-2xl p-4 space-y-3.5 text-xs">
              <div className="flex items-center justify-between pb-2 border-b border-white/[0.06]">
                <div>
                  <div className="font-semibold text-white">Aktifkan Popup HUD &amp; Notifikasi</div>
                  <div className="text-[11px] text-zinc-400">Pil akan sedikit melebar otomatis saat ada status penting</div>
                </div>
                <input
                  type="checkbox"
                  checked={settings.hud_enabled}
                  onChange={(e) => patch({ hud_enabled: e.target.checked })}
                  className="w-4 h-4 accent-blue-500"
                />
              </div>

              {settings.hud_enabled && settings.hud_windows_notif && (notifAccess === "denied" || notifAccess === "unspecified") && (
                <div className="rounded-xl bg-amber-500/10 border border-amber-400/25 px-3 py-2 flex items-center justify-between gap-2 text-[11px] text-amber-200">
                  <span>
                    Izin baca notifikasi Windows belum aktif ({notifAccess}). Aktifkan akses notifikasi agar Gravi bisa menangkap pesan masuk.
                  </span>
                  <button
                    type="button"
                    onClick={() => void tauriApi.openNotificationSettings()}
                    className="px-2.5 py-1 rounded-lg bg-amber-400/20 hover:bg-amber-400/30 text-amber-100 font-medium inline-flex items-center gap-1 shrink-0 transition"
                  >
                    <ExternalLink className="w-3 h-3" /> Buka Pengaturan
                  </button>
                </div>
              )}

              {[
                { key: "hud_capslock" as const, title: "HUD Caps Lock", desc: "Tampilkan indikator saat Caps Lock aktif/nonaktif" },
                { key: "hud_battery_charging" as const, title: "HUD Baterai Mengisi & Penuh", desc: "Tampilkan ekspresi petir saat charger dicolok / penuh 100%" },
                { key: "hud_battery_low" as const, title: "HUD Baterai Laptop Lemah (≤ 20%)", desc: "Peringatan baterai merah saat daya laptop kritis" },
                { key: "hud_bt_low" as const, title: "HUD Baterai Bluetooth Lemah (≤ 20%)", desc: "Notifikasi otomatis saat TWS/mouse/perangkat Bluetooth sisa ≤ 20%" },
                { key: "hud_wifi" as const, title: "HUD Wi-Fi (Nama SSID)", desc: "Tampilkan nama jaringan Wi-Fi saat terhubung atau terputus" },
                { key: "hud_ai_done" as const, title: "Popup Sesi AI Selesai", desc: "Ekspresi perayaan & tombol fokus saat AI desktop/CLI selesai bekerja" },
                { key: "hud_windows_notif" as const, title: "Notifikasi Windows + Balas Cepat", desc: "Tangkap notifikasi sosmed/chat Windows langsung di Island" },
                { key: "notif_messaging_only" as const, title: "Prioritaskan Sosmed & Pesan (Anti-Spam)", desc: "Saring agar hanya aplikasi pesan/sosmed yang muncul & pesan beruntun digabung" },
              ].map((item) => (
                <div key={item.key} className="flex items-center justify-between">
                  <div>
                    <div className="font-medium text-white">{item.title}</div>
                    <div className="text-[11px] text-zinc-400">{item.desc}</div>
                  </div>
                  <input
                    type="checkbox"
                    disabled={!settings.hud_enabled}
                    checked={settings[item.key]}
                    onChange={(e) => patch({ [item.key]: e.target.checked })}
                    className="w-4 h-4 accent-blue-500 disabled:opacity-40"
                  />
                </div>
              ))}

              <div className="space-y-1 pt-2 border-t border-white/[0.06]">
                <div className="flex justify-between">
                  <span className="font-medium text-white">Durasi Tampil HUD</span>
                  <span className="font-mono text-zinc-400">{(settings.hud_duration_ms / 1000).toFixed(1)} detik</span>
                </div>
                <input
                  type="range"
                  min={2000}
                  max={7000}
                  step={500}
                  value={settings.hud_duration_ms}
                  onChange={(e) => patch({ hud_duration_ms: Number(e.target.value) })}
                  className="w-full accent-blue-500"
                />
              </div>

              <div className="pt-2 border-t border-white/[0.06] space-y-2">
                <div className="text-[11px] font-medium text-zinc-300">Uji Coba Tampilan HUD &amp; Notifikasi</div>
                <div className="flex flex-wrap gap-1.5">
                  {[
                    { id: "capslock", label: "Tes CapsLock" },
                    { id: "battery_charging", label: "Tes Charging" },
                    { id: "battery_low", label: "Tes Lowbat" },
                    { id: "wifi_connected", label: "Tes Wi-Fi" },
                    { id: "ai_done", label: "Tes AI Selesai" },
                  ].map((btn) => (
                    <button
                      key={btn.id}
                      type="button"
                      onClick={() => void tauriApi.triggerTestHud(btn.id)}
                      className="px-2.5 py-1 rounded-xl bg-white/[0.06] hover:bg-white/15 border border-white/10 text-[11px] text-zinc-200 transition"
                    >
                      {btn.label}
                    </button>
                  ))}
                  <button
                    type="button"
                    onClick={() => void tauriApi.triggerTestNotification()}
                    className="px-2.5 py-1 rounded-xl bg-sky-500/20 hover:bg-sky-500/30 border border-sky-400/30 text-[11px] text-sky-200 transition"
                  >
                    Tes Notif WA
                  </button>
                </div>
              </div>
            </div>
          </div>
        )}

        {section === "utilities" && (
          <div className="space-y-3">
            <h2 className="text-base font-semibold text-white">Media, Fokus &amp; Pintasan</h2>
            <div className="macos-card rounded-2xl p-4 space-y-3.5 text-xs">
              <div className="grid grid-cols-2 gap-3">
                <div className="space-y-1">
                  <label className="text-zinc-400">Durasi Fokus Pomodoro (menit)</label>
                  <input
                    type="number"
                    min={5}
                    max={90}
                    value={settings.pomo_focus_mins}
                    onChange={(e) => patch({ pomo_focus_mins: Math.max(5, Number(e.target.value)) })}
                    className="w-full bg-white/[0.05] border border-white/10 rounded-xl px-3 py-1.5 font-mono text-xs outline-none"
                  />
                </div>
                <div className="space-y-1">
                  <label className="text-zinc-400">Durasi Istirahat (menit)</label>
                  <input
                    type="number"
                    min={1}
                    max={30}
                    value={settings.pomo_break_mins}
                    onChange={(e) => patch({ pomo_break_mins: Math.max(1, Number(e.target.value)) })}
                    className="w-full bg-white/[0.05] border border-white/10 rounded-xl px-3 py-1.5 font-mono text-xs outline-none"
                  />
                </div>
              </div>

              <div className="grid grid-cols-2 gap-3">
                <div className="space-y-1">
                  <label className="text-zinc-400">Batas Waktu Focus Habit</label>
                  <input
                    type="time"
                    value={settings.focus_habit_deadline || "21:00"}
                    onChange={(e) => patch({ focus_habit_deadline: e.target.value })}
                    className="w-full bg-white/[0.05] border border-white/10 rounded-xl px-3 py-1.5 font-mono text-xs outline-none"
                  />
                </div>
                <div className="space-y-1">
                  <label className="text-zinc-400">Alarm Sebelum Batas (menit)</label>
                  <input
                    type="number"
                    min={5}
                    max={180}
                    value={settings.focus_habit_warn_mins}
                    onChange={(e) => patch({ focus_habit_warn_mins: Math.max(5, Number(e.target.value) || 30) })}
                    className="w-full bg-white/[0.05] border border-white/10 rounded-xl px-3 py-1.5 font-mono text-xs outline-none"
                  />
                </div>
              </div>

              <div className="flex items-center justify-between">
                <div>
                  <div className="font-medium text-white">Sinkronisasi Media OS (GSMTC / MPRIS)</div>
                  <div className="text-[11px] text-zinc-400">Tampilkan lagu aktif, artwork, durasi &amp; kontrol pemutaran</div>
                </div>
                <input
                  type="checkbox"
                  checked={settings.media_sync_enabled}
                  onChange={(e) => patch({ media_sync_enabled: e.target.checked })}
                  className="w-4 h-4 accent-blue-500"
                />
              </div>

              <div className="flex items-center justify-between">
                <div>
                  <div className="font-medium text-white">Gravi Menari saat Musik Diputar</div>
                  <div className="text-[11px] text-zinc-400">Animasi menari + not musik otomatis saat ada lagu berjalan</div>
                </div>
                <input
                  type="checkbox"
                  checked={settings.media_dance_enabled}
                  onChange={(e) => patch({ media_dance_enabled: e.target.checked })}
                  className="w-4 h-4 accent-blue-500"
                />
              </div>

              <div className="grid grid-cols-2 gap-3 pt-2 border-t border-white/[0.06]">
                <div className="space-y-1">
                  <label className="text-zinc-400">Pintasan Buka/Tutup Island</label>
                  <input
                    value={settings.shortcut_toggle_island}
                    onChange={(e) => patch({ shortcut_toggle_island: e.target.value })}
                    className="w-full bg-white/[0.05] border border-white/10 rounded-xl px-3 py-1.5 font-mono text-xs outline-none"
                  />
                </div>
                <div className="space-y-1">
                  <label className="text-zinc-400">Pintasan Buka Local Drop</label>
                  <input
                    value={settings.shortcut_open_chat}
                    onChange={(e) => patch({ shortcut_open_chat: e.target.value })}
                    className="w-full bg-white/[0.05] border border-white/10 rounded-xl px-3 py-1.5 font-mono text-xs outline-none"
                  />
                </div>
              </div>
            </div>
          </div>
        )}
      </main>
    </div>
  );
};
