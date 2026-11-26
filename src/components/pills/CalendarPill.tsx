import React from "react";
import { ChevronLeft, ChevronRight, ExternalLink, RefreshCw } from "lucide-react";
import { useIslandStore } from "../../store/useIslandStore";
import { openExternalUrl } from "../../lib/tauriBridge";

const MONTH_NAMES = [
  "Januari", "Februari", "Maret", "April", "Mei", "Juni",
  "Juli", "Agustus", "September", "Oktober", "November", "Desember",
];

const DAY_HEADERS = ["Min", "Sen", "Sel", "Rab", "Kam", "Jum", "Sab"];

export const CalendarPill: React.FC = () => {
  const {
    calYear,
    calMonth,
    selectedDate,
    calendarEvents,
    calendarLoading,
    calendarError,
    setCalendarMonth,
    setSelectedDate,
    refreshCalendar,
  } = useIslandStore();

  const firstDayOfWeek = new Date(calYear, calMonth - 1, 1).getDay();
  const daysInMonth = new Date(calYear, calMonth, 0).getDate();
  const todayIso = new Date().toISOString().slice(0, 10);

  const eventDates = new Set(calendarEvents.map((e) => e.date));
  const selectedEvents = calendarEvents.filter((e) => e.date === selectedDate);
  const selDayNum = selectedDate.slice(8, 10);
  const selMonthShort = MONTH_NAMES[Number(selectedDate.slice(5, 7)) - 1]?.slice(0, 3) ?? "";

  const shiftMonth = (delta: number) => {
    let m = calMonth + delta;
    let y = calYear;
    if (m < 1) { m = 12; y -= 1; }
    else if (m > 12) { m = 1; y += 1; }
    void setCalendarMonth(y, m);
  };

  const cells: Array<{ day: number | null; iso: string }> = [];
  for (let i = 0; i < firstDayOfWeek; i++) cells.push({ day: null, iso: `empty-${i}` });
  for (let d = 1; d <= daysInMonth; d++) {
    const iso = `${calYear}-${String(calMonth).padStart(2, "0")}-${String(d).padStart(2, "0")}`;
    cells.push({ day: d, iso });
  }

  return (
    <div className="space-y-2.5">
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
          <div className="text-[11px] text-amber-300 bg-amber-500/10 border border-amber-400/25 rounded-xl px-2.5 py-1.5">
            {calendarError}
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

      {/* macOS Style Selected-Date Agenda Card */}
      <div className="macos-card rounded-2xl p-2.5 flex items-start gap-3">
        <div className="w-11 shrink-0 text-center pt-0.5">
          <div className="text-lg font-bold leading-none text-white tabular-nums">{selDayNum}</div>
          <div className="text-[10px] font-medium text-zinc-400 mt-0.5">{selMonthShort}</div>
        </div>
        <div className="flex-1 min-w-0 space-y-1.5 border-l border-white/[0.07] pl-3">
          {selectedEvents.length === 0 ? (
            <div className="text-xs text-zinc-500 py-1">Tidak ada jadwal pada tanggal ini.</div>
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
    </div>
  );
};
