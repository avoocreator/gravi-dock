import React from "react";
import { Disc3, Pause, Play, SkipBack, SkipForward } from "lucide-react";
import { useIslandStore } from "../../store/useIslandStore";

export const MusicPill: React.FC = () => {
  const { media, toggleMediaPlay, skipMedia } = useIslandStore();
  const pct =
    media.duration_secs > 0
      ? Math.min(100, Math.round((media.position_secs / media.duration_secs) * 100))
      : 30;

  return (
    <div className="macos-card rounded-2xl p-3 space-y-2.5">
      <div className="flex items-center justify-between gap-3">
        <div className="flex items-center gap-2.5 min-w-0">
          <div className="w-10 h-10 rounded-xl bg-gradient-to-tr from-blue-500 via-emerald-400 to-amber-400 flex items-center justify-center shadow shrink-0">
            <Disc3
              className={`w-5 h-5 text-zinc-950 ${media.is_playing ? "animate-spin" : ""}`}
              style={{ animationDuration: "4s" }}
            />
          </div>
          <div className="min-w-0">
            <div className="text-xs font-semibold text-white truncate">{media.title}</div>
            <div className="text-[11px] text-zinc-400 truncate">{media.artist}</div>
          </div>
        </div>

        <div className="flex items-center gap-1 shrink-0">
          <button
            onClick={() => void skipMedia("previous")}
            className="p-1.5 rounded-full hover:bg-white/10 text-zinc-300 hover:text-white transition"
          >
            <SkipBack className="w-3.5 h-3.5" />
          </button>
          <button
            onClick={() => void toggleMediaPlay()}
            className="w-8 h-8 rounded-full bg-white text-zinc-950 hover:bg-zinc-200 flex items-center justify-center transition shadow"
          >
            {media.is_playing ? (
              <Pause className="w-3.5 h-3.5 fill-current" />
            ) : (
              <Play className="w-3.5 h-3.5 fill-current ml-0.5" />
            )}
          </button>
          <button
            onClick={() => void skipMedia("next")}
            className="p-1.5 rounded-full hover:bg-white/10 text-zinc-300 hover:text-white transition"
          >
            <SkipForward className="w-3.5 h-3.5" />
          </button>
        </div>
      </div>

      <div className="h-1 w-full rounded-full bg-white/10 overflow-hidden">
        <div
          className="h-full rounded-full bg-white/80 transition-all duration-500"
          style={{ width: `${pct}%` }}
        />
      </div>
    </div>
  );
};
