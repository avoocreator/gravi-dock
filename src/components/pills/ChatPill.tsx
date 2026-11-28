import React, { useState } from "react";
import { Paperclip, ScanSearch, Send, Trash2, X } from "lucide-react";
import { useIslandStore } from "../../store/useIslandStore";

export const ChatPill: React.FC = () => {
  const {
    chatProvider,
    chatModel,
    chatMessages,
    chatLoading,
    droppedFiles,
    detectedProviders,
    providersLoading,
    setChatProvider,
    setChatModel,
    refreshAiProviders,
    refreshProviderModels,
    sendChat,
    clearChat,
    removeDroppedFile,
  } = useIslandStore();
  const [input, setInput] = useState("");
  const [modelsLoading, setModelsLoading] = useState(false);

  const currentProviderObj =
    detectedProviders.find((p) => p.id === chatProvider) ?? detectedProviders[0];
  const models = currentProviderObj?.models ?? [];

  const handleProviderChange = async (next: string) => {
    setChatProvider(next);
    setModelsLoading(true);
    const fetched = await refreshProviderModels(next);
    setModelsLoading(false);
    if (fetched.length > 0 && !fetched.includes(useIslandStore.getState().chatModel)) {
      setChatModel(fetched[0]);
    }
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!input.trim() || chatLoading) return;
    const text = input;
    setInput("");
    void sendChat(text);
  };

  return (
    <div className="flex flex-col h-[275px] gap-2">
      {/* Provider bar */}
      <div className="flex items-center gap-1.5">
        <select
          value={chatProvider}
          onChange={(e) => void handleProviderChange(e.target.value)}
          className="max-w-[165px] bg-white/[0.06] border border-white/10 rounded-lg px-2 py-1 text-[11px] text-zinc-100 outline-none truncate"
          title="Pilih provider AI yang terdeteksi"
        >
          {detectedProviders.map((p) => (
            <option key={p.id} value={p.id} className="bg-zinc-900">
              {p.has_account ? "● " : p.installed ? "◐ " : "○ "}
              {p.name}
            </option>
          ))}
        </select>

        {models.length > 0 ? (
          <select
            value={models.includes(chatModel) ? chatModel : models[0]}
            onChange={(e) => setChatModel(e.target.value)}
            disabled={modelsLoading}
            className="flex-1 min-w-0 bg-white/[0.05] border border-white/10 rounded-lg px-2 py-1 text-[11px] font-mono text-zinc-200 outline-none focus:border-blue-400/50 truncate"
          >
            {models.map((m) => (
              <option key={m} value={m} className="bg-zinc-900">
                {m}
              </option>
            ))}
          </select>
        ) : (
          <input
            value={chatModel}
            onChange={(e) => setChatModel(e.target.value)}
            placeholder="Nama model..."
            className="flex-1 min-w-0 bg-white/[0.05] border border-white/10 rounded-lg px-2 py-1 text-[11px] font-mono text-zinc-200 outline-none focus:border-blue-400/50"
          />
        )}

        <button
          type="button"
          onClick={() => void refreshAiProviders()}
          title="Pindai ulang provider & model AI"
          className="p-1.5 rounded-lg bg-white/[0.05] hover:bg-white/10 border border-white/10 text-zinc-400 hover:text-zinc-100 transition"
        >
          <ScanSearch className={`w-3 h-3 ${providersLoading ? "animate-pulse text-sky-400" : ""}`} />
        </button>

        <button
          type="button"
          onClick={clearChat}
          title="Bersihkan riwayat chat"
          className="p-1.5 rounded-lg bg-white/[0.05] hover:bg-white/10 border border-white/10 text-zinc-400 hover:text-zinc-100 transition"
        >
          <Trash2 className="w-3 h-3" />
        </button>
      </div>

      {/* Messages */}
      <div className="flex-1 overflow-y-auto gravi-scroll space-y-2 pr-1">
        {chatMessages.map((m) => (
          <div
            key={m.id}
            className={`rounded-xl px-2.5 py-2 text-xs leading-relaxed ${
              m.role === "user"
                ? "bg-blue-500/20 border border-blue-400/25 text-blue-50 ml-8"
                : "bg-white/[0.05] border border-white/[0.07] text-zinc-200 mr-6"
            }`}
          >
            {m.files && m.files.length > 0 && (
              <div className="flex flex-wrap gap-1 mb-1">
                {m.files.map((f) => (
                  <span
                    key={f}
                    className="text-[10px] px-1.5 py-0.5 rounded bg-black/40 text-zinc-300 font-mono truncate max-w-[200px]"
                  >
                    📎 {f.split(/[\\/]/).pop()}
                  </span>
                ))}
              </div>
            )}
            <div className="whitespace-pre-wrap">
              {m.content || (chatLoading && m.role === "assistant" ? "Mengetik..." : "")}
            </div>
          </div>
        ))}
      </div>

      {droppedFiles.length > 0 && (
        <div className="flex flex-wrap gap-1">
          {droppedFiles.map((path) => (
            <span
              key={path}
              className="inline-flex items-center gap-1 text-[10px] px-2 py-0.5 rounded-full bg-violet-500/20 border border-violet-400/30 text-violet-200"
            >
              <Paperclip className="w-2.5 h-2.5" />
              <span className="truncate max-w-[140px]">{path.split(/[\\/]/).pop()}</span>
              <button onClick={() => removeDroppedFile(path)} className="hover:text-white">
                <X className="w-2.5 h-2.5" />
              </button>
            </span>
          ))}
        </div>
      )}

      {/* Input */}
      <form onSubmit={handleSubmit} className="flex items-center gap-1.5">
        <input
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder="Tanya Gravi atau drop file ke island..."
          className="flex-1 bg-white/[0.06] border border-white/10 rounded-xl px-3 py-1.5 text-xs text-zinc-100 placeholder:text-zinc-500 outline-none focus:border-blue-400/50"
        />
        <button
          type="submit"
          disabled={chatLoading || !input.trim()}
          className="p-2 rounded-xl bg-blue-500 hover:bg-blue-400 disabled:opacity-40 text-white transition"
        >
          <Send className="w-3.5 h-3.5" />
        </button>
      </form>
    </div>
  );
};
