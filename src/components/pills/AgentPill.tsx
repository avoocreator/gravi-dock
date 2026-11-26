import React from "react";
import {
  Check,
  ExternalLink,
  ShieldAlert,
  Terminal,
  Monitor,
  X,
  CheckCheck,
} from "lucide-react";
import { useIslandStore } from "../../store/useIslandStore";
import { tauriApi } from "../../lib/tauriBridge";
import { AiLogo } from "../AiLogo";

export const AgentPill: React.FC = () => {
  const { aiSessions, permissions, resolvePermission, demoMode, pushPermission } =
    useIslandStore();

  const cliSessions = aiSessions.filter((s) => s.kind === "cli");
  const desktopSessions = aiSessions.filter((s) => s.kind === "desktop");

  const triggerDemoApproval = () => {
    pushPermission({
      id: `demo-${Date.now()}`,
      agent: "GitHub Copilot CLI",
      tool: "shell",
      command: "git push origin feat/ai-logo-loading-pill",
      cwd: "~/projects/nobi-island",
      timestamp: Date.now(),
    });
  };

  return (
    <div className="space-y-2.5">
      {permissions.length > 0 && (
        <div className="space-y-1.5">
          <div className="text-[10px] font-semibold uppercase tracking-wider text-amber-300 flex items-center gap-1.5">
            <ShieldAlert className="w-3 h-3" />
            Butuh Persetujuan ({permissions.length})
          </div>
          {permissions.map((perm) => (
            <div
              key={perm.id}
              className="rounded-2xl bg-amber-500/10 border border-amber-400/25 p-2.5 space-y-2"
            >
              <div className="flex items-center justify-between text-xs">
                <span className="font-semibold text-amber-200">{perm.agent}</span>
                <span className="text-[10px] px-1.5 py-0.5 rounded-md bg-black/40 text-zinc-300 font-mono">
                  {perm.tool}
                </span>
              </div>
              <pre className="text-[11px] font-mono bg-black/50 text-zinc-100 rounded-xl px-2.5 py-1.5 overflow-x-auto border border-white/5">
                {perm.command}
              </pre>
              <div className="flex items-center gap-1.5">
                <button
                  onClick={() => void resolvePermission(perm.id, "allow")}
                  className="flex-1 flex items-center justify-center gap-1 py-1.5 rounded-xl bg-emerald-500 hover:bg-emerald-400 text-zinc-950 font-semibold text-xs transition"
                >
                  <Check className="w-3.5 h-3.5" /> Allow
                </button>
                <button
                  onClick={() => void resolvePermission(perm.id, "always")}
                  className="px-2.5 py-1.5 rounded-xl bg-white/10 hover:bg-white/15 text-zinc-200 font-medium text-xs flex items-center gap-1 transition"
                >
                  <CheckCheck className="w-3.5 h-3.5" /> Always
                </button>
                <button
                  onClick={() => void resolvePermission(perm.id, "deny")}
                  className="px-2.5 py-1.5 rounded-xl bg-rose-500/20 hover:bg-rose-500/30 text-rose-300 border border-rose-500/25 font-medium text-xs flex items-center gap-1 transition"
                >
                  <X className="w-3.5 h-3.5" /> Deny
                </button>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* AI CLI Sessions */}
      <div className="macos-card rounded-2xl p-2.5 space-y-1.5">
        <div className="flex items-center justify-between">
          <span className="text-[10px] font-semibold uppercase tracking-wider text-zinc-400 flex items-center gap-1.5">
            <Terminal className="w-3 h-3" />
            AI CLI ({cliSessions.length})
          </span>
          {demoMode && (
            <button
              onClick={triggerDemoApproval}
              className="text-[10px] px-2 py-0.5 rounded-full bg-white/10 hover:bg-white/15 text-zinc-300 transition"
            >
              + Simulasi Izin
            </button>
          )}
        </div>
        {cliSessions.map((s) => (
          <div
            key={s.id}
            className="flex items-center justify-between gap-2 rounded-xl hover:bg-white/[0.04] px-2 py-1.5 transition"
          >
            <div className="flex items-center gap-2.5 min-w-0">
              <AiLogo
                provider={s.provider}
                size={20}
                loading={s.status === "active"}
                waiting={s.status === "waiting"}
              />
              <div className="min-w-0">
                <div className="text-xs font-medium text-zinc-100 truncate">{s.name}</div>
                <div className="text-[10px] text-zinc-400 truncate">{s.detail}</div>
              </div>
            </div>
            <span className="text-[10px] font-mono text-zinc-500 shrink-0 tabular-nums">
              {s.memory_mb} MB
            </span>
          </div>
        ))}
      </div>

      {/* AI Desktop Apps with Official Logos */}
      <div className="space-y-1.5">
        <span className="text-[10px] font-semibold uppercase tracking-wider text-zinc-400 flex items-center gap-1.5 px-1">
          <Monitor className="w-3 h-3" />
          AI Desktop ({desktopSessions.length})
        </span>
        <div className="grid grid-cols-2 gap-1.5">
          {desktopSessions.map((s) => (
            <div
              key={s.id}
              className="macos-card rounded-2xl px-2.5 py-2 flex items-center justify-between gap-1.5 hover:border-white/15 transition"
            >
              <div className="flex items-center gap-2 min-w-0">
                <AiLogo
                  provider={s.provider}
                  size={20}
                  loading={s.status === "active"}
                  waiting={s.status === "waiting"}
                />
                <div className="min-w-0">
                  <div className="text-[11px] font-medium text-zinc-100 truncate">{s.name}</div>
                  <div className="text-[10px] text-zinc-500 tabular-nums">{s.memory_mb} MB</div>
                </div>
              </div>
              <button
                onClick={() => void tauriApi.focusAiWindow(s.name)}
                className="p-1 rounded-lg hover:bg-white/10 text-zinc-400 hover:text-white transition shrink-0"
                title={`Fokus ke ${s.name}`}
              >
                <ExternalLink className="w-3 h-3" />
              </button>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};
