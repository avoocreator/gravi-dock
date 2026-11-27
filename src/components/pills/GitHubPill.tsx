import React, { useMemo, useState } from "react";
import {
  GitCommitHorizontal,
  FolderGit2,
  RefreshCw,
  Star,
  ChevronDown,
  ChevronUp,
  Settings,
  Flame,
  Lock,
  GitFork,
} from "lucide-react";
import { useIslandStore } from "../../store/useIslandStore";
import { isTauriRuntime, openExternalUrl, tauriApi } from "../../lib/tauriBridge";
import { ContributionDay } from "../../types/island";

// Authentic GitHub Dark Dimmed/Default contribution palette
const CELL_STYLES: Record<number, string> = {
  0: "bg-[#161b22] border-[#21262d]",
  1: "bg-[#0e4429] border-[#0e4429]",
  2: "bg-[#006d32] border-[#006d32]",
  3: "bg-[#26a641] border-[#26a641]",
  4: "bg-[#39d353] border-[#39d353] shadow-[0_0_6px_rgba(57,211,83,0.4)]",
};

const SHORT_MONTHS = [
  "Jan", "Feb", "Mar", "Apr", "Mei", "Jun",
  "Jul", "Agu", "Sep", "Okt", "Nov", "Des",
];

const SHORT_DAYS = ["Min", "Sen", "Sel", "Rab", "Kam", "Jum", "Sab"];

function formatIndoDate(iso: string): string {
  const parts = iso.split("-").map(Number);
  if (parts.length !== 3 || parts.some(Number.isNaN)) return iso;
  const d = new Date(parts[0], parts[1] - 1, parts[2]);
  return `${SHORT_DAYS[d.getDay()]}, ${parts[2]} ${SHORT_MONTHS[parts[1] - 1]} ${parts[0]}`;
}

function formatRelativeTime(iso: string): string {
  if (!iso) return "";
  const ms = Date.now() - new Date(iso).getTime();
  if (Number.isNaN(ms) || ms < 0) return "baru saja";
  const mins = Math.floor(ms / 60000);
  if (mins < 1) return "baru saja";
  if (mins < 60) return `${mins}m lalu`;
  const hrs = Math.floor(mins / 60);
  if (hrs < 24) return `${hrs}j lalu`;
  const days = Math.floor(hrs / 24);
  if (days < 30) return `${days}h lalu`;
  const months = Math.floor(days / 30);
  return `${months}bln lalu`;
}

function toLocalIsoToday(): string {
  const d = new Date();
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(
    d.getDate()
  ).padStart(2, "0")}`;
}

export const GitHubPill: React.FC = () => {
  const {
    githubRange,
    githubData,
    githubLoading,
    githubError,
    secretStatus,
    settings,
    setActiveTab,
    setGitHubRange,
    refreshGitHub,
  } = useIslandStore();

  const [showAllRepos, setShowAllRepos] = useState(false);
  const [showAllCommits, setShowAllCommits] = useState(false);
  const [hoveredDay, setHoveredDay] = useState<ContributionDay | null>(null);

  const todayIso = toLocalIsoToday();

  const ranges: Array<{ val: 3 | 6 | 12; label: string }> = [
    { val: 3, label: "3B" },
    { val: 6, label: "6B" },
    { val: 12, label: "1T" },
  ];

  const { weeks, monthLabels } = useMemo(() => {
    if (!githubData || githubData.days.length === 0) {
      return {
        weeks: [] as Array<Array<ContributionDay | null>>,
        monthLabels: [] as Array<{ col: number; label: string }>,
      };
    }
    const cols: Array<Array<ContributionDay | null>> = [];
    const firstWeekday = githubData.days[0].weekday ?? 0;
    let currentCol: Array<ContributionDay | null> = Array(firstWeekday).fill(null);

    for (const day of githubData.days) {
      currentCol.push(day);
      if (currentCol.length === 7) {
        cols.push(currentCol);
        currentCol = [];
      }
    }
    if (currentCol.length > 0) {
      while (currentCol.length < 7) currentCol.push(null);
      cols.push(currentCol);
    }

    const labels: Array<{ col: number; label: string }> = [];
    let lastMonth = -1;
    cols.forEach((col, colIdx) => {
      const firstValid = col.find((c) => c !== null);
      if (firstValid) {
        const m = Number(firstValid.date.slice(5, 7)) - 1;
        if (m !== lastMonth && m >= 0 && m < 12) {
          if (labels.length === 0 || colIdx - labels[labels.length - 1].col >= 3) {
            labels.push({ col: colIdx, label: SHORT_MONTHS[m] });
          }
          lastMonth = m;
        }
      }
    });

    return { weeks: cols, monthLabels: labels };
  }, [githubData]);

  const defaultLimit = Math.max(1, settings.github_default_items || 3);
  const visibleRepos = githubData
    ? showAllRepos
      ? githubData.repos
      : githubData.repos.slice(0, defaultLimit)
    : [];
  const visibleCommits = githubData
    ? showAllCommits
      ? githubData.commits
      : githubData.commits.slice(0, defaultLimit)
    : [];

  if (!secretStatus.github_token && !githubData) {
    return (
      <div className="macos-card rounded-2xl p-4 text-center space-y-2.5">
        <div className="text-xs font-semibold text-white">GitHub belum terhubung</div>
        <div className="text-[11px] text-zinc-400">
          Tambahkan GitHub Personal Access Token di Pengaturan untuk menampilkan grafik kontribusi, repo, dan commit terbaru.
        </div>
        <button
          onClick={() => {
            if (isTauriRuntime()) void tauriApi.openSettingsWindow();
            else setActiveTab("settings");
          }}
          className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-blue-500 hover:bg-blue-400 text-white text-xs font-medium transition"
        >
          <Settings className="w-3.5 h-3.5" /> Buka Pengaturan
        </button>
      </div>
    );
  }

  const activeDataRange = githubData?.range_months ?? githubRange;
  const cellSizeClass =
    activeDataRange === 12
      ? "w-[6px] h-[6px] rounded-[1.5px]"
      : activeDataRange === 6
      ? "w-[10px] h-[10px] rounded-[2px]"
      : "w-[12px] h-[12px] rounded-[2.5px]";
  const colGapClass =
    activeDataRange === 12 ? "gap-[1.5px]" : activeDataRange === 6 ? "gap-[2.5px]" : "gap-[3px]";
  const colStepPx = activeDataRange === 12 ? 7.5 : activeDataRange === 6 ? 12.5 : 15;
  const spanLabel =
    githubData && githubData.days.length > 1
      ? `${SHORT_MONTHS[Number(githubData.days[0].date.slice(5, 7)) - 1] ?? ""} ${githubData.days[0].date.slice(0, 4)} – ${
          SHORT_MONTHS[Number(githubData.days[githubData.days.length - 1].date.slice(5, 7)) - 1] ?? ""
        } ${githubData.days[githubData.days.length - 1].date.slice(0, 4)}`
      : `${githubRange === 12 ? "1 Tahun" : `${githubRange} Bulan`}`;
  const todayCount = githubData?.today_contributions ?? 0;

  return (
    <div className="space-y-2.5">
      {/* Authentic GitHub Dark Contribution Graph Card */}
      <div className="rounded-2xl bg-[#0d1117] border border-[#30363d] p-2.5 space-y-2 shadow-inner">
        <div className="flex items-center justify-between gap-2">
          <div className="flex items-center gap-2 min-w-0">
            <span className="text-xs font-semibold text-[#e6edf3] truncate">
              @{githubData?.username ?? "github"}
            </span>
            {githubData && (
              <span className="text-[10px] px-1.5 py-0.5 rounded-md bg-[#238636]/20 border border-[#2ea043]/40 text-[#3fb950] tabular-nums font-medium shrink-0">
                {githubData.total_contributions} kontribusi
              </span>
            )}
            {githubData && githubData.current_streak > 0 && (
              <span
                className="text-[10px] text-amber-300 flex items-center gap-0.5 tabular-nums shrink-0"
                title="Streak kontribusi berturut-turut"
              >
                <Flame className="w-3 h-3 text-amber-400" />
                {githubData.current_streak}h
              </span>
            )}
          </div>
          <div className="flex items-center gap-1 shrink-0">
            <div className="flex bg-[#161b22] rounded-lg p-0.5 border border-[#30363d]">
              {ranges.map((r) => (
                <button
                  key={r.val}
                  onClick={() => void setGitHubRange(r.val)}
                  className={`px-2 py-0.5 rounded-md text-[10px] font-medium transition ${
                    githubRange === r.val
                      ? "bg-[#238636] text-white font-semibold"
                      : "text-[#8b949e] hover:text-[#e6edf3]"
                  }`}
                >
                  {r.label}
                </button>
              ))}
            </div>
            <button
              onClick={() => void refreshGitHub()}
              className="p-1 rounded-lg bg-[#161b22] hover:bg-[#21262d] border border-[#30363d] text-[#8b949e] hover:text-white transition"
              title="Refresh GitHub"
            >
              <RefreshCw className={`w-3 h-3 ${githubLoading ? "animate-spin" : ""}`} />
            </button>
          </div>
        </div>

        {/* Authentic GitHub Hover Tooltip & Today Counter Strip */}
        {githubData && (
          <div
            className={`flex items-center justify-between rounded-lg px-2.5 py-1 text-[10px] border transition-colors ${
              hoveredDay
                ? "bg-[#1f2937] border-[#388bfd]/50 text-[#e6edf3]"
                : "bg-[#161b22] border-[#21262d] text-[#8b949e]"
            }`}
          >
            {hoveredDay ? (
              <>
                <span className="font-semibold text-[#39d353] tabular-nums">
                  {hoveredDay.count === 0
                    ? "0 commit"
                    : `${hoveredDay.count} kontribusi`}
                </span>
                <span className="text-[#c9d1d9]">
                  {hoveredDay.date === todayIso
                    ? `Hari ini · ${formatIndoDate(hoveredDay.date)}`
                    : formatIndoDate(hoveredDay.date)}
                </span>
              </>
            ) : (
              <>
                <span>Arahkan kursor ke kotak untuk detail harian</span>
                <span className="font-semibold text-[#3fb950] tabular-nums">
                  Hari ini: {todayCount} commit
                </span>
              </>
            )}
          </div>
        )}

        {githubError && (
          <div className="text-[11px] text-amber-300 bg-amber-500/10 border border-amber-400/25 rounded-xl px-2.5 py-1.5">
            {githubError}
          </div>
        )}

        {githubLoading && !githubData && (
          <div className="text-[11px] text-[#8b949e] py-2 text-center">
            Memuat data GitHub...
          </div>
        )}

        {githubData && weeks.length > 0 && (
          <div className="space-y-1.5">
            <div className="overflow-x-auto nobi-scroll pb-0.5">
              <div className="inline-flex flex-col gap-1">
                {/* Month Headers */}
                <div className="relative h-3 ml-6">
                  {monthLabels.map((m) => (
                    <span
                      key={`${m.col}-${m.label}`}
                      style={{ left: `${m.col * colStepPx}px` }}
                      className="absolute top-0 text-[9px] text-[#8b949e] font-medium select-none"
                    >
                      {m.label}
                    </span>
                  ))}
                </div>

                {/* Weekday labels + Authentic Square Matrix */}
                <div className="flex items-start gap-1.5">
                  <div className="flex flex-col justify-between text-[8px] text-[#8b949e] pt-0.5 select-none w-4 shrink-0 self-stretch">
                    <span className="opacity-0">Min</span>
                    <span>Sen</span>
                    <span className="opacity-0">Sel</span>
                    <span>Rab</span>
                    <span className="opacity-0">Kam</span>
                    <span>Jum</span>
                    <span className="opacity-0">Sab</span>
                  </div>

                  <div className={`flex ${colGapClass}`}>
                    {weeks.map((col, wIdx) => (
                      <div key={wIdx} className={`flex flex-col ${colGapClass}`}>
                        {col.map((d, dIdx) => {
                          if (!d) {
                            return (
                              <div
                                key={`pad-${wIdx}-${dIdx}`}
                                className={`${cellSizeClass} opacity-0`}
                              />
                            );
                          }
                          const isHovered = hoveredDay?.date === d.date;
                          const isToday = d.date === todayIso;
                          return (
                            <div
                              key={d.date}
                              onMouseEnter={() => setHoveredDay(d)}
                              onMouseLeave={() => setHoveredDay(null)}
                              className={`${cellSizeClass} border transition-transform duration-100 cursor-pointer ${
                                CELL_STYLES[d.level] ?? CELL_STYLES[0]
                              } ${
                                isHovered
                                  ? "scale-125 ring-1 ring-white z-10"
                                  : isToday
                                  ? "ring-1 ring-[#58a6ff]"
                                  : ""
                              }`}
                            />
                          );
                        })}
                      </div>
                    ))}
                  </div>
                </div>
              </div>
            </div>

            {/* Authentic GitHub Less -> More Legend */}
            <div className="flex items-center justify-between text-[9px] text-[#8b949e]">
              <span>{githubLoading ? `Memuat ${githubRange === 12 ? "1 Tahun" : `${githubRange} Bulan`}...` : spanLabel}</span>
              <div className="flex items-center gap-1">
                <span>Less</span>
                {[0, 1, 2, 3, 4].map((lvl) => (
                  <span
                    key={lvl}
                    className={`w-[9px] h-[9px] rounded-[2px] border ${CELL_STYLES[lvl]}`}
                  />
                ))}
                <span>More</span>
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Recent Repositories */}
      {githubData && githubData.repos.length > 0 && (
        <div className="macos-card rounded-2xl p-2.5 space-y-1.5">
          <div className="flex items-center justify-between">
            <div className="text-[10px] font-semibold uppercase tracking-wider text-zinc-400 flex items-center gap-1.5">
              <FolderGit2 className="w-3 h-3" /> Repo Terbaru
            </div>
            {githubData.repos.length > defaultLimit && (
              <button
                onClick={() => setShowAllRepos((v) => !v)}
                className="text-[10px] text-sky-400 hover:text-sky-300 flex items-center gap-0.5 transition"
              >
                {showAllRepos ? (
                  <>
                    Ringkas <ChevronUp className="w-3 h-3" />
                  </>
                ) : (
                  <>
                    Lihat semua ({githubData.repos.length}) <ChevronDown className="w-3 h-3" />
                  </>
                )}
              </button>
            )}
          </div>
          <div className="space-y-1">
            {visibleRepos.map((repo) => (
              <button
                key={repo.full_name}
                onClick={() => void openExternalUrl(repo.url)}
                className="w-full text-left flex items-center justify-between gap-2 rounded-xl hover:bg-white/[0.06] px-2 py-1 transition"
              >
                <div className="flex items-center gap-1.5 min-w-0">
                  <span
                    className="w-2 h-2 rounded-full shrink-0"
                    style={{ backgroundColor: repo.language_color ?? "#94A3B8" }}
                  />
                  <span className="text-xs font-medium text-zinc-100 truncate">{repo.name}</span>
                  {repo.is_private && (
                    <Lock className="w-2.5 h-2.5 text-zinc-500 shrink-0" />
                  )}
                  {repo.is_fork && (
                    <GitFork className="w-2.5 h-2.5 text-zinc-500 shrink-0" />
                  )}
                  {repo.language && (
                    <span className="text-[10px] text-zinc-500 shrink-0">{repo.language}</span>
                  )}
                </div>
                <div className="flex items-center gap-2 shrink-0">
                  <span className="text-[10px] text-zinc-500 tabular-nums">
                    {formatRelativeTime(repo.updated_at)}
                  </span>
                  <span className="text-[10px] text-zinc-400 tabular-nums flex items-center gap-0.5">
                    <Star className="w-2.5 h-2.5 text-amber-400" /> {repo.stars}
                  </span>
                </div>
              </button>
            ))}
          </div>
        </div>
      )}

      {/* Recent Viewer-Authored Commits */}
      {githubData && githubData.commits.length > 0 && (
        <div className="macos-card rounded-2xl p-2.5 space-y-1.5">
          <div className="flex items-center justify-between">
            <div className="text-[10px] font-semibold uppercase tracking-wider text-zinc-400 flex items-center gap-1.5">
              <GitCommitHorizontal className="w-3 h-3" /> Commit Terbaru Milikmu
            </div>
            {githubData.commits.length > defaultLimit && (
              <button
                onClick={() => setShowAllCommits((v) => !v)}
                className="text-[10px] text-sky-400 hover:text-sky-300 flex items-center gap-0.5 transition"
              >
                {showAllCommits ? (
                  <>
                    Ringkas <ChevronUp className="w-3 h-3" />
                  </>
                ) : (
                  <>
                    Lihat semua ({githubData.commits.length}) <ChevronDown className="w-3 h-3" />
                  </>
                )}
              </button>
            )}
          </div>
          <div className="space-y-1">
            {visibleCommits.map((c) => (
              <button
                key={`${c.repo}-${c.sha}`}
                onClick={() => void openExternalUrl(c.url)}
                className="w-full text-left flex items-center gap-2 rounded-xl hover:bg-white/[0.06] px-2 py-1 transition"
              >
                <code className="text-[10px] font-mono px-1.5 py-0.5 rounded-md bg-white/[0.07] text-sky-300 shrink-0 tabular-nums">
                  {c.sha}
                </code>
                <span className="text-[11px] text-zinc-200 truncate flex-1">{c.message}</span>
                <span className="text-[10px] text-zinc-500 shrink-0 max-w-[85px] truncate">
                  {c.repo}
                </span>
                <span className="text-[9px] text-zinc-500 shrink-0 tabular-nums">
                  {formatRelativeTime(c.committed_at)}
                </span>
              </button>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};
