/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        island: {
          bg: "#0B0B10",
          surface: "#15151E",
          border: "rgba(255,255,255,0.09)",
          accent: "#60A5FA",
          mint: "#34D399",
          coral: "#FB923C",
          amber: "#FBBF24",
        },
      },
      fontFamily: {
        sans: ["Inter", "Segoe UI", "system-ui", "sans-serif"],
        mono: ["JetBrains Mono", "Cascadia Code", "Consolas", "monospace"],
      },
    },
  },
  plugins: [],
};
