import React, { useEffect, useState } from "react";
import {
  ChevronLeft,
  ChevronRight,
  ExternalLink,
  RefreshCw,
  Settings,
  Sparkles,
  Send,
  X,
  ScanSearch,
} from "lucide-react";
import { useIslandStore } from "../../store/useIslandStore";
import { isTauriRuntime, openExternalUrl, tauriApi } from "../../lib/tauriBridge";

const MONTH_NAMES = [
  "Januari", "Februari", "Maret", "April", "Mei", "Juni",
  "Juli", "Agustus", "September", "Oktober", "November", "Desember",
];

function toLocalIsoDate(d: Date): string {
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(
    d.getDate()
  ).padStart(2, "0")}`;
}

export const CalendarPill: React.FC = () => {
  const {
    calYear,
    calMonth,
    selectedDate,
    calendarEvents,
    calendarLoading,
    calendarError,
    secretStatus,
    settings,
    detectedProviders,
    providersLoading,
    calendarAiProvider,
    calendarAiModel,
    calendarAiReply,
    calendarAiLoading,
    calendarAiError,
    setActiveTab,
    setCalendarMonth,
    setSelectedDate,
    refreshCalendar,
    refreshAiProviders,
    refreshProviderModels,
    setCalendarAiProvider,
    setCalendarAiModel,
    askCalendarAi,
    clearCalendarAiReply,
  } = useIslandStore();

  const [aiQuestion, setAiQuestion] = useState("");
  const [modelsLoading, setModelsLoading] = useState(false);

  useEffect(() => {
    if (isTauriRuntime() && detectedProviders.length <= 4) {
      void refreshAiProviders();
    }
  }, [refreshAiProviders, detectedProviders.length]);

  const currentProviderObj =
    detectedProviders.find((p) => p.id === calendarAiProvider) ?? detectedProviders[0];
  const availableModels = currentProviderObj?.models ?? [];

  const handleProviderChange = async (nextProvider: string) => {
    setCalendarAiProvider(nextProvider);
    setModelsLoading(true);
    const fetched = await refreshProviderModels(nextProvider);
    setModelsLoading(false);
    if (fetched.length > 0 && !fetched.includes(useIslandStore.getState().calendarAiModel)) {
      setCalendarAiModel(fetched[0]);
    }
  };

  const handleAiSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!aiQuestion.trim() || calendarAiLoading) return;
    const q = aiQuestion;
    setAiQuestion("");
    void askCalendarAi(q);
  };

  const mondayStart = settings.calendar_week_start === "monday";
  const DAY_HEADERS = mondayStart
    ? ["Sen", "Sel", "Rab", "Kam", "Jum", "Sab", "Min"]
    : ["Min", "Sen", "Sel", "Rab", "Kam", "Jum", "Sab"];

  const rawFirst = new Date(calYear, calMonth - 1, 1).getDay();
  const firstDayOffset = mondayStart ? (rawFirst + 6) % 7 : rawFirst;
  const daysInMonth = new Date(calYear, calMonth, 0).getDate();
  const todayIso = toLocalIsoDate(new Date());

  const eventDates = new Set(calendarEvents.map((e) => e.date));
  const selectedEvents = calendarEvents.filter((e) => e.date === selectedDate);
  const selDayNum = selectedDate.slice(8, 10);
  const selMonthShort = MONTH_NAMES[Number(selectedDate.slice(5, 7)) - 1]?.slice(0, 3) ?? "";

  const shiftMonth = (delta: number) => {
    let m = calMonth + delta;
    let y = calYear;
    if (m < 1) {
      m = 12;
      y -= 1;
    } else if (m > 12) {
      m = 1;
      y += 1;
    }
    void setCalendarMonth(y, m);
  };

  const cells: Array<{ day: number | null; iso: string }> = [];
  for (let i = 0; i < firstDayOffset; i++) cells.push({ day: null, iso: `empty-${i}` });
  for (let d = 1; d <= daysInMonth; d++) {
    const iso = `${calYear}-${String(calMonth).padStart(2, "0")}-${String(d).padStart(2, "0")}`;
    cells.push({ day: d, iso });
  }

  const calendarConfigured = secretStatus.google_ical_url || secretStatus.google_access_token;

  return (
    <div className="space-y-2">
      {/* Compact Mini Calendar Card */}
      <div className="macos-card rounded-2xl p-2.5 space-y-2">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-1">
            <button
              onClick={() => shiftMonth(-1)}
              className="p-1 rounded-lg hover:bg-white/10 text-zinc-400 hover:text-white transition"
            >
              <ChevronLeft className="w-3.5 h-3.5" />
            </button>
            <span className="text-xs font-semibold text-white min-w-[112px] text-center">
              {MONTH_NAMES[calMonth - 1]} {calYear}
            </span>
            <button
              onClick={() => shiftMonth(1)}
              className="p-1 rounded-lg hover:bg-white/10 text-zinc-400 hover:text-white transition"
            >
              <ChevronRight className="w-3.5 h-3.5" />
            </button>
          </div>
          <button
            onClick={() => void refreshCalendar()}
            className="p-1 rounded-lg bg-white/[0.05] hover:bg-white/10 text-zinc-400 hover:text-white transition"
            title="Refresh Google Calendar"
          >
            <RefreshCw className={`w-3 h-3 ${calendarLoading ? "animate-spin" : ""}`} />
          </button>
        </div>

        {calendarError && (
          <div className="text-[11px] text-amber-300 bg-amber-500/10 border border-amber-400/25 rounded-xl px-2.5 py-1.5 flex items-center justify-between gap-2">
            <span>{calendarError}</span>
            {!calendarConfigured && (
              <button
                onClick={() => {
                  if (isTauriRuntime()) void tauriApi.openSettingsWindow();
                  else setActiveTab("settings");
                }}
                className="px-2 py-1 rounded-lg bg-amber-400/20 hover:bg-amber-400/30 text-amber-200 text-[10px] font-medium inline-flex items-center gap-1 shrink-0 transition"
              >
                <Settings className="w-3 h-3" /> Atur
              </button>
            )}
          </div>
        )}

        <div className="grid grid-cols-7 gap-1">
          {DAY_HEADERS.map((h) => (
            <div key={h} className="text-[10px] text-center font-medium text-zinc-500">
              {h}
            </div>
          ))}
          {cells.map((cell) => {
            if (!cell.day) return <div key={cell.iso} />;
            const isSelected = cell.iso === selectedDate;
            const isToday = cell.iso === todayIso;
            const hasEvent = eventDates.has(cell.iso);

            return (
              <button
                key={cell.iso}
                onClick={() => setSelectedDate(cell.iso)}
                className={`relative h-6 rounded-lg text-[11px] font-medium tabular-nums flex flex-col items-center justify-center transition ${
                  isSelected
                    ? "bg-blue-500 text-white font-semibold shadow-sm"
                    : isToday
                    ? "bg-white/12 text-blue-300 border border-blue-400/40"
                    : "text-zinc-300 hover:bg-white/[0.08]"
                }`}
              >
                <span>{cell.day}</span>
                {hasEvent && (
                  <span
                    className={`w-1 h-1 rounded-full absolute bottom-0.5 ${
                      isSelected ? "bg-white" : "bg-emerald-400"
                    }`}
                  />
                )}
              </button>
            );
          })}
        </div>
      </div>

      {/* Selected-Date Agenda Card */}
      <div className="macos-card rounded-2xl p-2.5 flex items-start gap-3">
        <div className="w-11 shrink-0 text-center pt-0.5">
          <div className="text-lg font-bold leading-none text-white tabular-nums">{selDayNum}</div>
          <div className="text-[10px] font-medium text-zinc-400 mt-0.5">{selMonthShort}</div>
        </div>
        <div className="flex-1 min-w-0 space-y-1.5 border-l border-white/[0.07] pl-3">
          {selectedEvents.length === 0 ? (
            <div className="text-xs text-zinc-500 py-0.5">Tidak ada jadwal pada tanggal ini.</div>
          ) : (
            selectedEvents.map((ev) => (
              <div key={ev.id} className="flex items-start justify-between gap-2">
                <div className="min-w-0">
                  <div className="text-xs font-medium text-zinc-100 truncate">{ev.title}</div>
                  <div className="text-[10px] text-zinc-400 tabular-nums">
                    {ev.start_time}
                    {ev.end_time ? ` – ${ev.end_time}` : ""}
                    {ev.location ? ` · ${ev.location}` : ""}
                  </div>
                </div>
                {ev.meeting_url && (
                  <button
                    onClick={() => void openExternalUrl(ev.meeting_url!)}
                    className="p-1 rounded-lg bg-blue-500/20 hover:bg-blue-500/30 text-blue-300 shrink-0 transition"
                    title="Buka link agenda"
                  >
                    <ExternalLink className="w-3 h-3" />
                  </button>
                )}
              </div>
            ))
          )}
        </div>
      </div>

      {/* Auto-Detected AI Calendar Assistant */}
      <div className="macos-card rounded-2xl p-2.5 space-y-2">
        <div className="flex items-center justify-between gap-2">
          <div className="text-[10px] font-semibold uppercase tracking-wider text-zinc-300 flex items-center gap-1.5">
            <Sparkles className="w-3 h-3 text-sky-400" /> Asisten AI Kalender
          </div>
          <button
            onClick={() => void refreshAiProviders()}
            disabled={providersLoading}
            className="text-[10px] px-2 py-0.5 rounded-lg bg-white/[0.05] hover:bg-white/10 text-zinc-300 flex items-center gap-1 transition"
            title="Pindai ulang aplikasi AI & model di laptopmu"
          >
            <ScanSearch className={`w-3 h-3 ${providersLoading ? "animate-pulse text-sky-400" : ""}`} />
            {providersLoading ? "Memindai..." : "Scan AI"}
          </button>
        </div>

        {/* Detected Provider & Account Model Selectors */}
        <div className="grid grid-cols-2 gap-1.5">
          <select
            value={calendarAiProvider}
            onChange={(e) => void handleProviderChange(e.target.value)}
            className="bg-black/45 border border-white/10 rounded-xl px-2 py-1.5 text-[11px] text-zinc-100 outline-none focus:border-sky-400/50 truncate"
            title="Pilih provider AI yang terdeteksi di laptopmu"
          >
            {detectedProviders.map((p) => {
              const badge = p.has_account ? "● " : p.installed ? "◐ " : "○ ";
              return (
                <option key={p.id} value={p.id} className="bg-zinc-900">
                  {badge}
                  {p.name}
                </option>
              );
            })}
          </select>

          <select
            value={calendarAiModel}
            onChange={(e) => setCalendarAiModel(e.target.value)}
            disabled={modelsLoading}
            className="bg-black/45 border border-white/10 rounded-xl px-2 py-1.5 text-[11px] font-mono text-zinc-100 outline-none focus:border-sky-400/50 truncate"
            title="Pilih model yang tersedia di akun/instalasi ini"
          >
            {availableModels.length === 0 ? (
              <option value={calendarAiModel} className="bg-zinc-900">
                {calendarAiModel || "Default"}
              </option>
            ) : (
              availableModels.map((m) => (
                <option key={m} value={m} className="bg-zinc-900">
                  {m}
                </option>
              ))
            )}
          </select>
        </div>

        {currentProviderObj && (
          <div className="flex items-center justify-between text-[10px] px-0.5">
            <span
              className={
                currentProviderObj.has_account
                  ? "text-emerald-400"
                  : currentProviderObj.installed
                  ? "text-amber-300"
                  : "text-zinc-500"
              }
            >
              {currentProviderObj.status_label}
            </span>
            {!currentProviderObj.has_account && currentProviderObj.source !== "cli" && (
              <button
                onClick={() => {
                  if (isTauriRuntime()) void tauriApi.openSettingsWindow();
                  else setActiveTab("settings");
                }}
                className="text-sky-400 hover:text-sky-300 underline"
              >
                Atur kunci
              </button>
            )}
          </div>
        )}

        {/* Quick AI Calendar Chips */}
        <div className="flex flex-wrap gap-1">
          {[
            { label: "Ringkas hari ini", prompt: "Ringkas agenda tanggal yang dipilih dan prioritas utamanya." },
            { label: "Cek jadwal bulan ini", prompt: "Lihat semua jadwal bulan ini, sebutkan agenda penting terdekat." },
            { label: "Saran blok fokus", prompt: "Sarankan slot waktu terbaik untuk deep work berdasarkan jadwalku hari ini." },
          ].map((chip) => (
            <button
              key={chip.label}
              disabled={calendarAiLoading}
              onClick={() => void askCalendarAi(chip.prompt)}
              className="text-[10px] px-2 py-1 rounded-lg bg-white/[0.05] hover:bg-sky-500/20 border border-white/[0.07] hover:border-sky-400/30 text-zinc-300 hover:text-sky-200 transition disabled:opacity-40"
            >
              {chip.label}
            </button>
          ))}
        </div>

        {/* Custom AI Calendar Prompt */}
        <form onSubmit={handleAiSubmit} className="flex items-center gap-1.5">
          <input
            value={aiQuestion}
            onChange={(e) => setAiQuestion(e.target.value)}
            placeholder="Tanya AI soal jadwalmu..."
            className="flex-1 bg-white/[0.05] border border-white/10 rounded-xl px-2.5 py-1 text-[11px] text-zinc-100 placeholder:text-zinc-500 outline-none focus:border-sky-400/50"
          />
          <button
            type="submit"
            disabled={calendarAiLoading || !aiQuestion.trim()}
            className="p-1.5 rounded-xl bg-sky-500 hover:bg-sky-400 disabled:opacity-40 text-zinc-950 transition"
          >
            <Send className="w-3 h-3" />
          </button>
        </form>

        {calendarAiLoading && (
          <div className="text-[11px] text-sky-300 italic px-1">
            Menganalisis jadwal dengan {currentProviderObj?.name ?? calendarAiProvider}...
          </div>
        )}

        {calendarAiError && (
          <div className="text-[11px] text-rose-300 bg-rose-500/10 border border-rose-500/25 rounded-xl px-2.5 py-1.5">
            {calendarAiError}
          </div>
        )}

        {calendarAiReply && (
          <div className="relative rounded-xl bg-black/40 border border-sky-400/20 px-2.5 py-2 text-[11px] text-zinc-200 whitespace-pre-wrap leading-relaxed max-h-28 overflow-y-auto nobi-scroll">
            <button
              onClick={clearCalendarAiReply}
              className="absolute top-1.5 right-1.5 p-0.5 rounded hover:bg-white/10 text-zinc-400 hover:text-white"
              title="Tutup jawaban"
            >
              <X className="w-3 h-3" />
            </button>
            <div className="pr-4">{calendarAiReply}</div>
          </div>
        )}
      </div>
    </div>
  );
};
