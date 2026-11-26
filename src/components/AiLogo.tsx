import React from "react";

interface AiLogoProps {
  provider: string;
  size?: number;
  loading?: boolean;
  waiting?: boolean;
}

export const AiLogo: React.FC<AiLogoProps> = ({
  provider,
  size = 20,
  loading = false,
  waiting = false,
}) => {
  const renderIcon = () => {
    switch (provider) {
      case "claude":
        return (
          <svg viewBox="0 0 24 24" className="w-full h-full">
            <rect width="24" height="24" rx="6" fill="#D97757" />
            <g stroke="#FFF" strokeWidth="1.7" strokeLinecap="round">
              <line x1="12" y1="4.5" x2="12" y2="19.5" />
              <line x1="4.5" y1="12" x2="19.5" y2="12" />
              <line x1="6.7" y1="6.7" x2="17.3" y2="17.3" />
              <line x1="17.3" y1="6.7" x2="6.7" y2="17.3" />
            </g>
          </svg>
        );
      case "gemini":
        return (
          <svg viewBox="0 0 24 24" className="w-full h-full">
            <defs>
              <linearGradient id="gemG" x1="0" y1="0" x2="1" y2="1">
                <stop offset="0%" stopColor="#EF4444" />
                <stop offset="35%" stopColor="#FACC15" />
                <stop offset="68%" stopColor="#22C55E" />
                <stop offset="100%" stopColor="#3B82F6" />
              </linearGradient>
            </defs>
            <rect width="24" height="24" rx="6" fill="#12131A" />
            <path
              d="M12 3.2C12.8 8.3 15.7 11.2 20.8 12C15.7 12.8 12.8 15.7 12 20.8C11.2 15.7 8.3 12.8 3.2 12C8.3 11.2 11.2 8.3 12 3.2Z"
              fill="url(#gemG)"
            />
          </svg>
        );
      case "chatgpt":
        return (
          <svg viewBox="0 0 24 24" className="w-full h-full">
            <rect width="24" height="24" rx="6" fill="#141419" />
            <g fill="none" stroke="#FFFFFF" strokeWidth="1.5" strokeLinejoin="round">
              <circle cx="12" cy="12" r="2.6" />
              <path d="M12 4.8a3.2 3.2 0 0 1 5.2 2.5v3.2" />
              <path d="M18.2 8.4a3.2 3.2 0 0 1-.4 5.7l-2.8 1.6" />
              <path d="M18.2 15.6a3.2 3.2 0 0 1-5.2 2.5l-2.8-1.6" />
              <path d="M12 19.2a3.2 3.2 0 0 1-5.2-2.5v-3.2" />
              <path d="M5.8 15.6a3.2 3.2 0 0 1 .4-5.7l2.8-1.6" />
              <path d="M5.8 8.4a3.2 3.2 0 0 1 5.2-2.5l2.8 1.6" />
            </g>
          </svg>
        );
      case "antigravity":
        return (
          <svg viewBox="0 0 24 24" className="w-full h-full">
            <defs>
              <linearGradient id="agyG" x1="0" y1="1" x2="1" y2="0">
                <stop offset="0%" stopColor="#3B82F6" />
                <stop offset="45%" stopColor="#22C55E" />
                <stop offset="75%" stopColor="#FACC15" />
                <stop offset="100%" stopColor="#EF4444" />
              </linearGradient>
            </defs>
            <rect width="24" height="24" rx="6" fill="#FFFFFF" />
            <path
              d="M6.5 17.5C8.5 8.5 10.5 6 12 6C13.5 6 15.5 8.5 17.5 17.5"
              fill="none"
              stroke="url(#agyG)"
              strokeWidth="3.2"
              strokeLinecap="round"
            />
          </svg>
        );
      case "copilot":
        return (
          <svg viewBox="0 0 24 24" className="w-full h-full">
            <rect width="24" height="24" rx="6" fill="#161B22" />
            <path
              d="M6.5 10.5C6.5 8 8.5 6.5 12 6.5C15.5 6.5 17.5 8 17.5 10.5V14.5C17.5 16 16 17 14 17H10C8 17 6.5 16 6.5 14.5V10.5Z"
              fill="none"
              stroke="#58A6FF"
              strokeWidth="1.6"
            />
            <circle cx="9.8" cy="12" r="1.4" fill="#FFFFFF" />
            <circle cx="14.2" cy="12" r="1.4" fill="#FFFFFF" />
          </svg>
        );
      case "openbot":
        return (
          <svg viewBox="0 0 24 24" className="w-full h-full">
            <rect width="24" height="24" rx="6" fill="#C4A7E7" />
            <circle cx="9" cy="12" r="2.3" fill="#111118" />
            <circle cx="15" cy="12" r="2.3" fill="#111118" />
          </svg>
        );
      case "hermes":
      default:
        return (
          <svg viewBox="0 0 24 24" className="w-full h-full">
            <rect width="24" height="24" rx="6" fill="#F5F5F7" />
            <path
              d="M7 18V7h2.5v4.2h5V7H17v11h-2.5v-4.5h-5V18H7z"
              fill="#111118"
            />
          </svg>
        );
    }
  };

  if (!loading && !waiting) {
    return (
      <div
        style={{ width: size, height: size }}
        className="rounded-full overflow-hidden shrink-0 flex items-center justify-center"
      >
        {renderIcon()}
      </div>
    );
  }

  return (
    <div
      style={{ width: size + 6, height: size + 6 }}
      className="relative rounded-full p-[2px] shrink-0 flex items-center justify-center overflow-hidden"
    >
      <div
        className={`absolute inset-[-40%] ai-border-spin ${
          waiting ? "ai-ring-waiting" : "ai-ring-loading"
        }`}
      />
      <div
        style={{ width: size, height: size }}
        className="relative z-10 rounded-full overflow-hidden bg-[#060608] p-[1.5px]"
      >
        {renderIcon()}
      </div>
    </div>
  );
};
