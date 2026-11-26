import React, { useState } from "react";
import { CheckCircle2, KeyRound, Save } from "lucide-react";
import { useIslandStore } from "../../store/useIslandStore";
import { tauriApi } from "../../lib/tauriBridge";

export const SettingsPill: React.FC = () => {
  const {
    demoMode,
    toggleDemoMode,
    secretStatus,
    refreshSecretStatus,
    refreshGitHub,
    refreshCalendar,
  } = useIslandStore();

  const [values, setValues] = useState<Record<string, string>>({});
  const [savedMsg, setSavedMsg] = useState<string | null>(null);

  const fields: Array<{ key: keyof typeof secretStatus; label: string; placeholder: string }> = [
    {
      key: "github_token",
      label: "GitHub PAT (read:user, repo)",
      placeholder: "ghp_xxxxxxxxxxxxxxxxxxxx",
    },
    {
      key: "google_ical_url",
      label: "Google Calendar Secret iCal URL (Paling Mudah)",
      placeholder: "https://calendar.google.com/calendar/ical/.../basic.ics",
    },
    {
      key: "google_access_token",
      label: "Google Calendar OAuth Access Token (Opsional)",
      placeholder: "ya29.a0...",
    },
    {
      key: "anthropic_api_key",
      label: "Anthropic API Key (Mini Chat)",
      placeholder: "sk-ant-...",
    },
    {
      key: "gemini_api_key",
      label: "Gemini API Key (Mini Chat)",
      placeholder: "AIza...",
    },
    {
      key: "openai_api_key",
      label: "OpenAI API Key (Mini Chat)",
      placeholder: "sk-...",
    },
  ];

  const handleSave = async (key: string) => {
    const val = values[key] ?? "";
    try {
      await tauriApi.saveSecret(key, val);
      await refreshSecretStatus();
      setSavedMsg(`Tersimpan di OS Keyring (${key})`);
      if (key === "github_token") void refreshGitHub();
      if (key.startsWith("google_")) void refreshCalendar();
      setTimeout(() => setSavedMsg(null), 2500);
    } catch (err) {
      setSavedMsg(`Gagal simpan: ${String(err)}`);
    }
  };

  return (
    <div className="space-y-3">
      {/* Demo Mode Switch */}
      <div className="flex items-center justify-between rounded-xl bg-white/[0.04] border border-white/10 px-3 py-2">
        <div>
          <div className="text-xs font-semibold text-zinc-100">Mode Demo</div>
          <div className="text-[10px] text-zinc-400">
            Tampilkan data simulasi tanpa token API
          </div>
        </div>
        <button
          onClick={toggleDemoMode}
          className={`px-3 py-1 rounded-full text-[11px] font-semibold transition ${
            demoMode
              ? "bg-emerald-500 text-zinc-950"
              : "bg-white/10 text-zinc-300 hover:bg-white/15"
          }`}
        >
          {demoMode ? "Aktif" : "Mati (Live)"}
        </button>
      </div>

      {savedMsg && (
        <div className="text-[11px] text-emerald-300 bg-emerald-500/10 border border-emerald-500/25 rounded-lg px-2.5 py-1.5">
          {savedMsg}
        </div>
      )}

      {/* OS Keyring Credentials */}
      <div className="space-y-2">
        <div className="text-[11px] font-semibold uppercase tracking-wider text-zinc-400 flex items-center gap-1.5">
          <KeyRound className="w-3.5 h-3.5" /> Kredensial (OS Keyring)
        </div>
        {fields.map((f) => (
          <div key={f.key} className="space-y-1">
            <div className="flex items-center justify-between text-[11px]">
              <span className="text-zinc-300">{f.label}</span>
              {secretStatus[f.key] && (
                <span className="text-[10px] text-emerald-400 flex items-center gap-0.5">
                  <CheckCircle2 className="w-3 h-3" /> Tersimpan
                </span>
              )}
            </div>
            <div className="flex gap-1.5">
              <input
                type="password"
                value={values[f.key] ?? ""}
                onChange={(e) => setValues((v) => ({ ...v, [f.key]: e.target.value }))}
                placeholder={f.placeholder}
                className="flex-1 bg-white/[0.05] border border-white/10 rounded-lg px-2.5 py-1 text-[11px] font-mono text-zinc-100 placeholder:text-zinc-600 outline-none focus:border-blue-400/50"
              />
              <button
                onClick={() => void handleSave(f.key)}
                className="px-2.5 py-1 rounded-lg bg-white/10 hover:bg-white/20 text-zinc-200 text-[11px] flex items-center gap-1 transition"
              >
                <Save className="w-3 h-3" /> Simpan
              </button>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};
