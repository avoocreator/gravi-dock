import React, { useState } from "react";
import { GitCommitHorizontal, FolderGit2, RefreshCw, Star, ChevronDown, ChevronUp } from "lucide-react";
import { useIslandStore } from "../../store/useIslandStore";
import { openExternalUrl } from "../../lib/tauriBridge";

const DOT_COLORS: Record<number, string> = {
  0: "bg-white/[0.08]",
  1: "bg-emerald-800/90",
  2: "bg-emerald-600",
  3: "bg-emerald-400",
  4: "bg-emerald-300 shadow-[0_0_6px_rgba(110,231,183,0.45)]",
};

export const GitHubPill: React.FC = () => {
  const {
    githubRange,
    githubData,
    githubLoading,
    githubError,
    setGitHubRange,
    refreshGitHub,
  } = useIslandStore();

  const [showAllRepos, setShowAllRepos] = useState(false);
  const [showAllCommits, setShowAllCommits] = useState(false);

  const ranges: Array<{ val: 3 | 6 | 12; label: string }> = [
    { val: 3, label: "3B" },
    { val: 6, label: "6B" },
    { val: 12, label: "1T" },
  ];

  const visibleRepos = githubData
    ? showAllRepos
      ? githubData.repos
      : githubData.repos.slice(0, 3)
    : [];
  const visibleCommits = githubData
    ? showAllCommits
      ? githubData.commits
      : githubData.commits.slice(0, 3)
    : [];

  return (
    <div className="space-y-2.5">
      {/* Compact Dot-Matrix Header Card */}
      <div className="macos-card rounded-2xl p-2.5 space-y-2">
        <div className="flex items-center justify-between gap-2">
          <div className="flex items-center gap-2 min-w-0">
            <span className="text-xs font-semibold text-white truncate">
              @{githubData?.username ?? "github"}
            </span>
            <span className="text-[11px] text-zinc-400 tabular-nums">
              {githubData ? `${githubData.total_contributions} kontribusi` : ""}
            </span>
          </div>
          <div className="flex items-center gap-1 shrink-0">
            <div className="flex bg-black/40 rounded-lg p-0.5 border border-white/[0.06]">
              {ranges.map((r) => (
                <button
                  key={r.val}
                  onClick={() => void setGitHubRange(r.val)}
                  className={`px-2 py-0.5 rounded-md text-[10px] font-medium transition ${
                    githubRange === r.val
                      ? "bg-white/15 text-white font-semibold"
                      : "text-zinc-400 hover:text-zinc-200"
                  }`}
                >
                  {r.label}
                </button>
              ))}
            </div>
            <button
              onClick={() => void refreshGitHub()}
              className="p-1 rounded-lg bg-white/[0.05] hover:bg-white/10 text-zinc-400 hover:text-white transition"
              title="Refresh GitHub"
            >
              <RefreshCw className={`w-3 h-3 ${githubLoading ? "animate-spin" : ""}`} />
            </button>
          </div>
        </div>

        {githubError && (
          <div className="text-[11px] text-amber-300 bg-amber-500/10 border border-amber-400/25 rounded-xl px-2.5 py-1.5">
            {githubError}
          </div>
        )}

        {githubData && (
          <div className="overflow-x-auto nobi-scroll pb-0.5">
            <div className="grid grid-rows-7 grid-flow-col gap-[3px] w-max">
              {githubData.days.map((d) => (
                <div
                  key={d.date}
                  title={`${d.date}: ${d.count} kontribusi`}
                  className={`rounded-full transition-colors ${
                    githubRange === 12
                      ? "w-[6px] h-[6px]"
                      : githubRange === 6
                      ? "w-[8px] h-[8px]"
                      : "w-[10px] h-[10px]"
                  } ${DOT_COLORS[d.level] ?? DOT_COLORS[0]}`}
                />
              ))}
            </div>
          </div>
        )}
      </div>

      {/* Top 3 Recent Repositories + Expand Button */}
      {githubData && githubData.repos.length > 0 && (
        <div className="macos-card rounded-2xl p-2.5 space-y-1.5">
          <div className="flex items-center justify-between">
            <div className="text-[10px] font-semibold uppercase tracking-wider text-zinc-400 flex items-center gap-1.5">
              <FolderGit2 className="w-3 h-3" /> Repo Terbaru
            </div>
            {githubData.repos.length > 3 && (
              <button
                onClick={() => setShowAllRepos((v) => !v)}
                className="text-[10px] text-sky-400 hover:text-sky-300 flex items-center gap-0.5 transition"
              >
                {showAllRepos ? (
                  <>Ringkas <ChevronUp className="w-3 h-3" /></>
                ) : (
                  <>Lihat lebih lengkap ({githubData.repos.length}) <ChevronDown className="w-3 h-3" /></>
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
                <div className="flex items-center gap-2 min-w-0">
                  <span
                    className="w-2 h-2 rounded-full shrink-0"
                    style={{ backgroundColor: repo.language_color ?? "#94A3B8" }}
                  />
                  <span className="text-xs font-medium text-zinc-100 truncate">{repo.name}</span>
                  {repo.language && (
                    <span className="text-[10px] text-zinc-500 shrink-0">{repo.language}</span>
                  )}
                </div>
                <span className="text-[10px] text-zinc-400 tabular-nums flex items-center gap-0.5 shrink-0">
                  <Star className="w-2.5 h-2.5 text-amber-400" /> {repo.stars}
                </span>
              </button>
            ))}
          </div>
        </div>
      )}

      {/* Top 3 Recent Commits + Expand Button */}
      {githubData && githubData.commits.length > 0 && (
        <div className="macos-card rounded-2xl p-2.5 space-y-1.5">
          <div className="flex items-center justify-between">
            <div className="text-[10px] font-semibold uppercase tracking-wider text-zinc-400 flex items-center gap-1.5">
              <GitCommitHorizontal className="w-3 h-3" /> Commit Terbaru
            </div>
            {githubData.commits.length > 3 && (
              <button
                onClick={() => setShowAllCommits((v) => !v)}
                className="text-[10px] text-sky-400 hover:text-sky-300 flex items-center gap-0.5 transition"
              >
                {showAllCommits ? (
                  <>Ringkas <ChevronUp className="w-3 h-3" /></>
                ) : (
                  <>Lihat lebih lengkap ({githubData.commits.length}) <ChevronDown className="w-3 h-3" /></>
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
                <span className="text-[10px] text-zinc-500 shrink-0">{c.repo}</span>
              </button>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};
