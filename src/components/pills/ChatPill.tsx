import React, { useState } from "react";
import { Paperclip, Send, X } from "lucide-react";
import { useIslandStore } from "../../store/useIslandStore";

export const ChatPill: React.FC = () => {
  const {
    chatProvider,
    chatModel,
    chatMessages,
    chatLoading,
    droppedFiles,
    setChatProvider,
    setChatModel,
    sendChat,
    removeDroppedFile,
  } = useIslandStore();
  const [input, setInput] = useState("");

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!input.trim() || chatLoading) return;
    const text = input;
    setInput("");
    void sendChat(text);
  };

  return (
    <div className="flex flex-col h-[275px] gap-2">
      {/* Provider & Model bar */}
      <div className="flex items-center gap-1.5">
        <select
          value={chatProvider}
          onChange={(e) =>
            setChatProvider(e.target.value as "ollama" | "openai" | "anthropic" | "gemini")
          }
          className="bg-white/[0.06] border border-white/10 rounded-lg px-2 py-1 text-[11px] text-zinc-100 outline-none"
        >
          <option value="ollama" className="bg-zinc-900">Ollama (Lokal)</option>
          <option value="anthropic" className="bg-zinc-900">Anthropic</option>
          <option value="gemini" className="bg-zinc-900">Gemini</option>
          <option value="openai" className="bg-zinc-900">OpenAI</option>
        </select>
        <input
          value={chatModel}
          onChange={(e) => setChatModel(e.target.value)}
          placeholder="Nama model..."
          className="flex-1 bg-white/[0.05] border border-white/10 rounded-lg px-2 py-1 text-[11px] font-mono text-zinc-200 outline-none focus:border-blue-400/50"
        />
      </div>

      {/* Messages */}
      <div className="flex-1 overflow-y-auto nobi-scroll space-y-2 pr-1">
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
            <div className="whitespace-pre-wrap">{m.content}</div>
          </div>
        ))}
        {chatLoading && (
          <div className="text-[11px] text-zinc-400 italic px-2">Nobi sedang berpikir...</div>
        )}
      </div>

      {/* Dropped files attachment bar */}
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
          placeholder="Tanya Nobi atau drop file ke island..."
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
