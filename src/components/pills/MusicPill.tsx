import React, { useEffect, useState } from "react";
import {
  Disc3,
  Pause,
  Play,
  SkipBack,
  SkipForward,
  Volume1,
  Volume2,
  VolumeX,
  RefreshCw,
  SlidersHorizontal,
} from "lucide-react";
import { useIslandStore } from "../../store/useIslandStore";
import { tauriApi } from "../../lib/tauriBridge";
import { AudioAppSession } from "../../types/island";

function formatTime(secs: number): string {
  const s = Math.max(0, Math.floor(secs));
  const m = Math.floor(s / 60);
  const rem = s % 60;
  return `${m}:${String(rem).padStart(2, "0")}`;
}

export const MusicPill: React.FC = () => {
  const {
    media,
    toggleMediaPlay,
    skipMedia,
    seekMedia,
    adjustMediaVolume,
    refreshMedia,
  } = useIslandStore();

  const [subTab, setSubTab] = useState<"player" | "mixer">("player");
  const [sessions, setSessions] = useState<AudioAppSession[]>([]);
  const [mixerLoading, setMixerLoading] = useState(false);

  const loadMixer = async () => {
    setMixerLoading(true);
    try {
      const list = await tauriApi.listAudioSessions();
      if (list) setSessions(list);
    } catch {
      // ignore
    } finally {
      setMixerLoading(false);
    }
  };

  useEffect(() => {
    if (subTab === "mixer") {
      void loadMixer();
    }
  }, [subTab]);

  const handleAppVol = (appKey: string, vol: number) => {
    setSessions((prev) =>
      prev.map((s) =>
        s.app_key === appKey ? { ...s, volume_percent: vol, muted: vol === 0 ? s.muted : false } : s
      )
    );
    void tauriApi.setAppAudioVolume(appKey, vol);
  };

  const handleAppMute = (appKey: string, nextMuted: boolean) => {
    setSessions((prev) =>
      prev.map((s) => (s.app_key === appKey ? { ...s, muted: nextMuted } : s))
    );
    void tauriApi.setAppAudioMute(appKey, nextMuted);
  };

  const hasDuration = media.duration_secs > 0;
  const clampedPos = hasDuration
    ? Math.min(media.duration_secs, media.position_secs)
    : media.position_secs;
  const pct = hasDuration ? Math.min(100, (clampedPos / media.duration_secs) * 100) : 0;

  const handleSeekClick = (e: React.MouseEvent<HTMLDivElement>) => {
    if (!hasDuration) return;
    const rect = e.currentTarget.getBoundingClientRect();
    const ratio = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
    void seekMedia(Math.round(ratio * media.duration_secs));
  };

  return (
    <div className="space-y-2">
      {/* Segmented Sub-Tab: Pemutar Media vs Mixer Aplikasi */}
      <div className="flex items-center gap-1 bg-white/[0.03] p-0.5 rounded-xl border border-white/[0.06]">
        <button
          onClick={() => setSubTab("player")}
          className={`flex-1 py-1 rounded-lg text-[11px] font-medium transition ${
            subTab === "player"
              ? "bg-white/[0.12] text-white"
              : "text-zinc-400 hover:text-zinc-200"
          }`}
        >
          Pemutar Media
        </button>
        <button
          onClick={() => setSubTab("mixer")}
          className={`flex-1 py-1 rounded-lg text-[11px] font-medium flex items-center justify-center gap-1.5 transition ${
            subTab === "mixer"
              ? "bg-white/[0.12] text-white"
              : "text-zinc-400 hover:text-zinc-200"
          }`}
        >
          <SlidersHorizontal className="w-3 h-3" />
          <span>Mixer Aplikasi</span>
        </button>
      </div>

      {subTab === "player" ? (
        <div className="macos-card rounded-2xl p-3 space-y-3">
          {/* Top Row: Album Art + Track Info + Player Badge */}
          <div className="flex items-center justify-between gap-3">
            <div className="flex items-center gap-3 min-w-0">
              {media.artwork_url ? (
                <img
                  src={media.artwork_url}
                  alt={media.title}
                  className="w-12 h-12 rounded-xl object-cover shadow-md border border-white/10 shrink-0"
                />
              ) : (
                <div className="w-12 h-12 rounded-xl bg-gradient-to-tr from-blue-500 via-emerald-400 to-amber-400 flex items-center justify-center shadow shrink-0">
                  <Disc3
                    className={`w-6 h-6 text-zinc-950 ${media.is_playing ? "animate-spin" : ""}`}
                    style={{ animationDuration: "4s" }}
                  />
                </div>
              )}

              <div className="min-w-0 space-y-0.5">
                <div className="flex items-center gap-1.5">
                  <span className="text-xs font-semibold text-white truncate">{media.title}</span>
                  {media.is_playing && (
                    <span className="inline-flex items-end gap-[2px] h-2.5 shrink-0">
                      <span className="w-[2px] h-full bg-emerald-400 rounded-full eq-bar" style={{ animationDelay: "0ms" }} />
                      <span className="w-[2px] h-full bg-sky-400 rounded-full eq-bar" style={{ animationDelay: "160ms" }} />
                      <span className="w-[2px] h-full bg-emerald-300 rounded-full eq-bar" style={{ animationDelay: "320ms" }} />
                    </span>
                  )}
                </div>
                <div className="text-[11px] text-zinc-400 truncate">
                  {media.artist}
                  {media.album ? ` · ${media.album}` : ""}
                </div>
                {media.player && (
                  <div className="text-[10px] text-zinc-500 flex items-center gap-1.5">
                    <span className="px-1.5 py-0.2 rounded-md bg-white/[0.06] border border-white/[0.06] text-zinc-300">
                      {media.player}
                    </span>
                    <span>
                      {media.is_playing ? "Sedang diputar" : media.has_media ? "Dijeda" : "Siaga"}
                    </span>
                  </div>
                )}
              </div>
            </div>

            <button
              onClick={() => void refreshMedia()}
              className="p-1.5 rounded-xl bg-white/[0.05] hover:bg-white/10 text-zinc-400 hover:text-white transition shrink-0 self-start"
              title="Sinkronkan media OS"
            >
              <RefreshCw className="w-3.5 h-3.5" />
            </button>
          </div>

          {/* Interactive Timeline Seek Bar */}
          <div className="space-y-1">
            <div
              onClick={handleSeekClick}
              className={`group relative h-2 w-full flex items-center ${
                hasDuration ? "cursor-pointer" : "cursor-default"
              }`}
              title={hasDuration ? "Klik untuk lompat ke posisi lagu" : undefined}
            >
              <div className="h-1 group-hover:h-1.5 w-full rounded-full bg-white/10 overflow-hidden transition-all">
                <div
                  className="h-full rounded-full bg-gradient-to-r from-sky-400 to-emerald-400 transition-all duration-300"
                  style={{ width: `${hasDuration ? pct : media.is_playing ? 40 : 0}%` }}
                />
              </div>
            </div>
            <div className="flex items-center justify-between text-[10px] font-mono text-zinc-500 tabular-nums">
              <span>{formatTime(clampedPos)}</span>
              <span>{hasDuration ? formatTime(media.duration_secs) : "--:--"}</span>
            </div>
          </div>

          {/* Transport + Master Volume Controls */}
          <div className="flex items-center justify-between gap-2 pt-0.5">
            <div className="flex items-center gap-1">
              <button
                onClick={() => void adjustMediaVolume("volume_down")}
                className="p-1.5 rounded-xl bg-white/[0.04] hover:bg-white/10 text-zinc-400 hover:text-white transition"
                title="Kurangi volume master"
              >
                <Volume1 className="w-3.5 h-3.5" />
              </button>
              <button
                onClick={() => void adjustMediaVolume("mute")}
                className="p-1.5 rounded-xl bg-white/[0.04] hover:bg-white/10 text-zinc-400 hover:text-white transition"
                title="Bisukan / Bunyikan master"
              >
                <VolumeX className="w-3.5 h-3.5" />
              </button>
              <button
                onClick={() => void adjustMediaVolume("volume_up")}
                className="p-1.5 rounded-xl bg-white/[0.04] hover:bg-white/10 text-zinc-400 hover:text-white transition"
                title="Naikkan volume master"
              >
                <Volume2 className="w-3.5 h-3.5" />
              </button>
            </div>

            <div className="flex items-center gap-1.5">
              <button
                onClick={() => void skipMedia("previous")}
                className="p-2 rounded-full hover:bg-white/10 text-zinc-300 hover:text-white transition"
                title="Lagu sebelumnya"
              >
                <SkipBack className="w-4 h-4" />
              </button>
              <button
                onClick={() => void toggleMediaPlay()}
                className="w-9 h-9 rounded-full bg-white text-zinc-950 hover:bg-zinc-200 flex items-center justify-center transition shadow-md"
                title={media.is_playing ? "Jeda" : "Putar"}
              >
                {media.is_playing ? (
                  <Pause className="w-4 h-4 fill-current" />
                ) : (
                  <Play className="w-4 h-4 fill-current ml-0.5" />
                )}
              </button>
              <button
                onClick={() => void skipMedia("next")}
                className="p-2 rounded-full hover:bg-white/10 text-zinc-300 hover:text-white transition"
                title="Lagu berikutnya"
              >
                <SkipForward className="w-4 h-4" />
              </button>
            </div>

            <div className="w-[74px]" />
          </div>
        </div>
      ) : (
        /* Native Windows WASAPI Per-App Volume Mixer */
        <div className="macos-card rounded-2xl p-3 space-y-2.5">
          <div className="flex items-center justify-between">
            <div>
              <div className="text-xs font-semibold text-white">Mixer Volume per Aplikasi</div>
              <div className="text-[10px] text-zinc-400">
                Atur volume masing-masing software secara langsung (WASAPI)
              </div>
            </div>
            <button
              onClick={() => void loadMixer()}
              className="p-1.5 rounded-xl bg-white/[0.05] hover:bg-white/10 text-zinc-400 hover:text-white transition"
              title="Muat ulang daftar aplikasi audio"
            >
              <RefreshCw className={`w-3.5 h-3.5 ${mixerLoading ? "animate-spin" : ""}`} />
            </button>
          </div>

          {sessions.length === 0 ? (
            <div className="text-[11px] text-zinc-500 text-center py-4">
              {mixerLoading
                ? "Memuat aplikasi audio aktif..."
                : "Belum ada aplikasi yang sedang mengeluarkan suara. Putar audio di Chrome, Spotify, atau Discord lalu klik refresh."}
            </div>
          ) : (
            <div className="space-y-1.5 max-h-44 overflow-y-auto nobi-scroll pr-0.5">
              {sessions.map((s) => (
                <div
                  key={s.app_key}
                  className="rounded-xl bg-white/[0.04] border border-white/[0.06] px-2.5 py-2 flex items-center gap-2.5"
                >
                  <button
                    type="button"
                    onClick={() => handleAppMute(s.app_key, !s.muted)}
                    className={`p-1 rounded-lg transition shrink-0 ${
                      s.muted
                        ? "bg-rose-500/20 text-rose-300"
                        : "bg-white/[0.06] hover:bg-white/15 text-zinc-300"
                    }`}
                    title={s.muted ? "Bunyikan" : "Bisukan"}
                  >
                    {s.muted || s.volume_percent === 0 ? (
                      <VolumeX className="w-3.5 h-3.5" />
                    ) : (
                      <Volume2 className="w-3.5 h-3.5 text-sky-400" />
                    )}
                  </button>

                  <div className="flex-1 min-w-0 space-y-1">
                    <div className="flex items-center justify-between gap-2 text-[11px]">
                      <span className="font-medium text-zinc-100 truncate">{s.display_name}</span>
                      <span className="font-mono text-[10px] text-zinc-400 tabular-nums shrink-0">
                        {s.muted ? "Mute" : `${s.volume_percent}%`}
                      </span>
                    </div>
                    <input
                      type="range"
                      min={0}
                      max={100}
                      value={s.muted ? 0 : s.volume_percent}
                      onChange={(e) => handleAppVol(s.app_key, Number(e.target.value))}
                      className="w-full h-1 accent-sky-400 cursor-pointer"
                    />
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
};
