import React, { useEffect, useMemo, useState } from "react";
import {
  Check,
  Copy,
  FolderDown,
  Power,
  QrCode,
  Share2,
  Trash2,
  Upload,
  FileText,
} from "lucide-react";
import { useIslandStore } from "../../store/useIslandStore";
import { isTauriRuntime, openExternalUrl } from "../../lib/tauriBridge";
import { generateQrMatrix } from "../../lib/qrGenerator";

const QrSvg: React.FC<{ value: string; size?: number }> = ({ value, size = 112 }) => {
  const mat = useMemo(() => generateQrMatrix(value), [value]);
  const n = mat.length;
  const pad = 2;
  const total = n + pad * 2;
  return (
    <svg
      width={size}
      height={size}
      viewBox={`0 0 ${total} ${total}`}
      className="rounded-xl bg-white p-1 shadow shrink-0"
      shapeRendering="crispEdges"
    >
      <rect width={total} height={total} fill="#ffffff" />
      {mat.map((row, r) =>
        row.map((dark, c) =>
          dark ? (
            <rect key={`${r}-${c}`} x={c + pad} y={r + pad} width={1} height={1} fill="#09090b" />
          ) : null
        )
      )}
    </svg>
  );
};

function formatBytes(b: number): string {
  if (b < 1024) return `${b} B`;
  if (b < 1024 * 1024) return `${Math.round(b / 1024)} KB`;
  return `${(b / (1024 * 1024)).toFixed(1)} MB`;
}

export const LocalDropPill: React.FC = () => {
  const {
    localDrop,
    localDropLoading,
    refreshLocalDrop,
    toggleLocalDropServer,
    removeLocalDropFile,
  } = useIslandStore();
  const [copiedId, setCopiedId] = useState<string | null>(null);

  useEffect(() => {
    void refreshLocalDrop();
  }, [refreshLocalDrop]);

  const copyText = (id: string, text: string) => {
    void navigator.clipboard?.writeText(text);
    setCopiedId(id);
    setTimeout(() => setCopiedId(null), 1600);
  };

  return (
    <div className="space-y-2.5">
      {/* Server & QR */}
      <div className="macos-card rounded-2xl p-3 space-y-2.5">
        <div className="flex items-center justify-between gap-2">
          <div className="flex items-center gap-2 min-w-0">
            <div
              className={`w-7 h-7 rounded-xl flex items-center justify-center shrink-0 ${
                localDrop.running
                  ? "bg-emerald-500/20 text-emerald-300 border border-emerald-400/30"
                  : "bg-white/[0.06] text-zinc-400 border border-white/[0.08]"
              }`}
            >
              <QrCode className="w-3.5 h-3.5" />
            </div>
            <div className="min-w-0">
              <div className="text-xs font-semibold text-white truncate">
                Local Drop (HP ↔ Laptop)
              </div>
              <div className="text-[10px] text-zinc-400 truncate">
                {localDrop.running
                  ? (localDrop.connected_clients ?? 0) > 0
                    ? `● ${localDrop.connected_clients} perangkat terhubung (iDrop auto-simpan aktif)`
                    : "Scan QR sekali dari HP → drop file langsung masuk storage HP"
                  : "Kirim file instan ala iDrop tanpa internet"}
              </div>
            </div>
          </div>

          <button
            onClick={() => void toggleLocalDropServer()}
            disabled={localDropLoading}
            className={`px-2.5 py-1.5 rounded-xl text-[11px] font-semibold flex items-center gap-1 transition shrink-0 ${
              localDrop.running
                ? "bg-rose-500/20 hover:bg-rose-500/30 text-rose-300 border border-rose-500/30"
                : "bg-sky-500 hover:bg-sky-400 text-zinc-950"
            }`}
          >
            <Power className="w-3 h-3" />
            {localDropLoading ? "..." : localDrop.running ? "Matikan" : "Nyalakan"}
          </button>
        </div>

        {localDrop.running && localDrop.url && (
          <div className="flex items-center gap-3 pt-1">
            <QrSvg value={localDrop.url} size={104} />
            <div className="flex-1 min-w-0 space-y-2">
              <div className="space-y-0.5">
                <div className="text-[10px] uppercase tracking-wider text-zinc-500 font-semibold">
                  Alamat LAN Lokal
                </div>
                <div className="flex items-center gap-1.5">
                  <code className="text-[11px] font-mono text-sky-300 bg-black/40 border border-white/10 rounded-lg px-2 py-1 truncate flex-1">
                    {localDrop.url}
                  </code>
                  <button
                    onClick={() => copyText("url", localDrop.url)}
                    className="p-1.5 rounded-lg bg-white/[0.06] hover:bg-white/15 text-zinc-300 transition shrink-0"
                    title="Salin URL"
                  >
                    {copiedId === "url" ? (
                      <Check className="w-3 h-3 text-emerald-400" />
                    ) : (
                      <Copy className="w-3 h-3" />
                    )}
                  </button>
                </div>
              </div>
              <div className="text-[10px] text-zinc-400 leading-relaxed">
                Tarik &amp; lepaskan file ke Island untuk langsung dibagikan ke HP. File masuk tersimpan di{" "}
                <span className="font-mono text-zinc-300">Downloads/NobiDrop</span>.
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Shared files */}
      <div className="macos-card rounded-2xl p-2.5 space-y-1.5">
        <div className="flex items-center justify-between text-[10px] font-semibold uppercase tracking-wider text-zinc-400">
          <span className="flex items-center gap-1.5">
            <Upload className="w-3 h-3 text-sky-400" />
            Dibagikan ke HP ({localDrop.shared_files.length})
          </span>
          <span className="text-[9px] text-zinc-500 normal-case font-normal">
            Drag file ke Island
          </span>
        </div>

        {localDrop.shared_files.length === 0 ? (
          <div className="text-[11px] text-zinc-500 px-1 py-1">
            Belum ada file. Tarik file apa saja ke Nobi Island untuk dibagikan ke HP.
          </div>
        ) : (
          <div className="space-y-1 max-h-24 overflow-y-auto nobi-scroll">
            {localDrop.shared_files.map((f) => (
              <div
                key={f.id}
                className="flex items-center justify-between gap-2 rounded-xl bg-white/[0.03] px-2 py-1 text-[11px]"
              >
                <span className="text-zinc-200 truncate">{f.name}</span>
                <div className="flex items-center gap-2 shrink-0">
                  <span className="text-[10px] font-mono text-zinc-500 tabular-nums">
                    {formatBytes(f.size_bytes)}
                  </span>
                  <button
                    onClick={() => void removeLocalDropFile(f.id)}
                    className="p-0.5 rounded hover:bg-white/10 text-zinc-500 hover:text-rose-300 transition"
                    title="Hapus dari daftar berbagi"
                  >
                    <Trash2 className="w-3 h-3" />
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Received items */}
      {localDrop.received_items.length > 0 && (
        <div className="macos-card rounded-2xl p-2.5 space-y-1.5">
          <div className="flex items-center justify-between text-[10px] font-semibold uppercase tracking-wider text-zinc-400">
            <span className="flex items-center gap-1.5">
              <FolderDown className="w-3 h-3 text-emerald-400" />
              Diterima dari HP ({localDrop.received_items.length})
            </span>
            {isTauriRuntime() && localDrop.save_dir && (
              <button
                onClick={() => void openExternalUrl(localDrop.save_dir)}
                className="text-[10px] text-sky-400 hover:text-sky-300 normal-case font-medium flex items-center gap-1"
              >
                <Share2 className="w-2.5 h-2.5" /> Buka Folder
              </button>
            )}
          </div>
          <div className="space-y-1 max-h-24 overflow-y-auto nobi-scroll">
            {localDrop.received_items.slice(0, 6).map((item) => (
              <div
                key={item.id}
                className="flex items-center justify-between gap-2 rounded-xl bg-emerald-500/10 border border-emerald-400/20 px-2 py-1 text-[11px]"
              >
                <div className="flex items-center gap-1.5 min-w-0">
                  {item.kind === "text" ? (
                    <FileText className="w-3 h-3 text-emerald-300 shrink-0" />
                  ) : (
                    <FolderDown className="w-3 h-3 text-emerald-300 shrink-0" />
                  )}
                  <span className="text-zinc-100 truncate">
                    {item.kind === "text" ? item.content_or_path : item.name}
                  </span>
                </div>
                <button
                  onClick={() => copyText(item.id, item.content_or_path)}
                  className="p-1 rounded-lg hover:bg-white/10 text-zinc-300 shrink-0"
                  title={item.kind === "text" ? "Salin teks" : "Salin lokasi file"}
                >
                  {copiedId === item.id ? (
                    <Check className="w-3 h-3 text-emerald-400" />
                  ) : (
                    <Copy className="w-3 h-3" />
                  )}
                </button>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};
