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
} from "lucide-react";
import { useIslandStore } from "../store/useIslandStore";
import { AppSettings } from "../types/island";
import { tauriApi } from "../lib/tauriBridge";

type SettingsSection =
  | "general"
  | "character"
  | "ai"
  | "github"
  | "calendar"
  | "chat"
  | "utilities";

const SECTIONS: Array<{ id: SettingsSection; label: string; keywords: string; icon: React.ReactNode }> = [
  { id: "general", label: "Umum & Island", keywords: "posisi jam format 24 12 transparansi demo startup", icon: <Sliders className="w-4 h-4" /> },
  { id: "character", label: "Karakter & Suara", keywords: "nobi eye tracking telinga squish dizzy volume suara", icon: <Sparkles className="w-4 h-4" /> },
  { id: "ai", label: "AI Monitor & Hooks", keywords: "claude gemini chatgpt antigravity copilot hermes openbot ipc hook", icon: <Bot className="w-4 h-4" /> },
  { id: "github", label: "GitHub", keywords: "pat token kontribusi 3b 6b 1t repo commit refresh", icon: <Github className="w-4 h-4" /> },
  { id: "calendar", label: "Google Calendar", keywords: "ical oauth agenda pengingat pekan senin minggu", icon: <CalendarDays className="w-4 h-4" /> },
  { id: "chat", label: "Mini Chat & File", keywords: "ollama anthropic openai gemini api key prompt drop file", icon: <MessageSquare className="w-4 h-4" /> },
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
  const { settings, secretStatus, updateSettings, resetAllSettings, refreshSecretStatus } =
    useIslandStore();
  const [section, setSection] = useState<SettingsSection>("general");
  const [search, setSearch] = useState("");
  const [secretsInput, setSecretsInput] = useState<Record<string, string>>({});
  const [statusBanner, setStatusBanner] = useState<string | null>(null);

  useEffect(() => {
    void refreshSecretStatus();
  }, [refreshSecretStatus]);

  const filteredSections = SECTIONS.filter((s) => {
    const q = search.trim().toLowerCase();
    if (!q) return true;
    return s.label.toLowerCase().includes(q) || s.keywords.includes(q);
  });

  const patch = (partial: Partial<AppSettings>) => {
    void updateSettings(partial);
    setStatusBanner("Pengaturan tersinkronisasi real-time ke Nobi Island");
    setTimeout(() => setStatusBanner(null), 1800);
  };

  const saveKey = async (key: string, label: string) => {
    try {
      await tauriApi.saveSecret(key, secretsInput[key] ?? "");
      await refreshSecretStatus();
      setSecretsInput((prev) => ({ ...prev, [key]: "" }));
      setStatusBanner(`${label} tersimpan aman di OS Keyring`);
      setTimeout(() => setStatusBanner(null), 2400);
    } catch (err) {
      setStatusBanner(`Gagal menyimpan ${label}: ${String(err)}`);
    }
  };

  const toggleAiTarget = (id: string) => {
    const has = settings.monitored_ais.includes(id);
    const next = has
      ? settings.monitored_ais.filter((x) => x !== id)
      : [...settings.monitored_ais, id];
    patch({ monitored_ais: next });
  };

  return (
    <div className="w-screen h-screen bg-[#0B0B0F] text-zinc-100 flex overflow-hidden select-none">
      {/* macOS System Settings Sidebar */}
      <aside className="w-60 bg-[#111117] border-r border-white/[0.08] p-3 flex flex-col gap-3 shrink-0">
        <div className="px-1">
          <div className="text-sm font-bold tracking-tight text-white">Pengaturan Nobi</div>
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

        <nav className="flex-1 space-y-1 overflow-y-auto nobi-scroll">
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

      {/* Content Pane */}
      <main className="flex-1 p-5 overflow-y-auto nobi-scroll space-y-4">
        {statusBanner && (
          <div className="rounded-xl bg-emerald-500/15 border border-emerald-400/30 px-3 py-2 text-xs text-emerald-200 flex items-center gap-2">
            <CheckCircle2 className="w-4 h-4 shrink-0" />
            <span>{statusBanner}</span>
          </div>
        )}

        {section === "general" && (
          <div className="space-y-3">
            <h2 className="text-base font-semibold text-white">Umum &amp; Tampilan Island</h2>
            <div className="macos-card rounded-2xl p-4 space-y-3.5 text-xs">
              <div className="flex items-center justify-between">
                <div>
                  <div className="font-medium text-white">Posisi Dynamic Island</div>
                  <div className="text-[11px] text-zinc-400">Atur letak pil di tepi atas layar</div>
                </div>
                <select
                  value={settings.island_position}
                  onChange={(e) => {
                    const pos = e.target.value as AppSettings["island_position"];
                    patch({ island_position: pos });
                    void tauriApi.recenterIsland(pos);
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
                  <div className="font-medium text-white">Sembunyikan Otomatis saat Idle</div>
                  <div className="text-[11px] text-zinc-400">Pil mengecil jadi garis tipis saat tidak dipakai</div>
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

              <div className="flex items-center justify-between pt-2 border-t border-white/[0.06]">
                <div>
                  <div className="font-medium text-white">Mode Demo</div>
                  <div className="text-[11px] text-zinc-400">Gunakan data simulasi tanpa token API</div>
                </div>
                <input
                  type="checkbox"
                  checked={settings.demo_mode}
                  onChange={(e) => patch({ demo_mode: e.target.checked })}
                  className="w-4 h-4 accent-emerald-500"
                />
              </div>
            </div>
          </div>
        )}

        {section === "character" && (
          <div className="space-y-3">
            <h2 className="text-base font-semibold text-white">Karakter Nobi &amp; Suara</h2>
            <div className="macos-card rounded-2xl p-4 space-y-3.5 text-xs">
              <div className="flex items-center justify-between">
                <div>
                  <div className="font-medium text-white">Eye-Tracking Kursor</div>
                  <div className="text-[11px] text-zinc-400">Mata LED Nobi mengikuti arah kursor</div>
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
                  <div className="text-[11px] text-zinc-400">Telinga berkedut saat hover &amp; ikut tempo musik</div>
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
                  <div className="font-medium text-white">Interaksi Squish &amp; Dizzy</div>
                  <div className="text-[11px] text-zinc-400">Klik 1x untuk squish, klik 3x cepat untuk pusing</div>
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
                  <div className="text-[11px] text-zinc-400">Suara halus untuk notifikasi izin &amp; klik Nobi</div>
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
              <div className="rounded-xl bg-black/40 border border-white/[0.06] p-2.5 font-mono text-[11px] text-sky-300">
                IPC Hook Server: 127.0.0.1:47891 (nobi-hook fail-open aktif)
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
            <h2 className="text-base font-semibold text-white">Integrasi Google Calendar</h2>
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
                </div>
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
                    onChange={(e) => patch({ chat_default_provider: e.target.value as AppSettings["chat_default_provider"] })}
                    className="w-full bg-zinc-900 border border-white/10 rounded-xl px-2.5 py-1.5 text-xs"
                  >
                    <option value="ollama">Ollama (Lokal)</option>
                    <option value="anthropic">Anthropic</option>
                    <option value="gemini">Gemini</option>
                    <option value="openai">OpenAI</option>
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
                  </div>
                </div>
              ))}
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

              <div className="flex items-center justify-between">
                <div>
                  <div className="font-medium text-white">Sinkronisasi Media OS (GSMTC / MPRIS)</div>
                  <div className="text-[11px] text-zinc-400">Tampilkan lagu aktif &amp; buat Nobi menari</div>
                </div>
                <input
                  type="checkbox"
                  checked={settings.media_sync_enabled}
                  onChange={(e) => patch({ media_sync_enabled: e.target.checked })}
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
                  <label className="text-zinc-400">Pintasan Buka Mini Chat</label>
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
