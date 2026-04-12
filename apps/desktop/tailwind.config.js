/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        cyber: {
          bg: "#0a0e17",
          surface: "#111827",
          card: "#151d2e",
          "card-hover": "#1a2438",
          border: "#1e3a5f",
          "border-bright": "#2a5a8f",
          text: "#c8d6e5",
          "text-dim": "#6b7d95",
          heading: "#e2e8f0",
          green: "#39ff6e",
          "green-dim": "#1a8c3c",
          blue: "#00b4d8",
          purple: "#7c3aed",
          red: "#ff4757",
          "red-dim": "#8b1a25",
        },
      },
      fontFamily: {
        mono: [
          "Fira Code",
          "Cascadia Code",
          "JetBrains Mono",
          "monospace",
        ],
      },
      boxShadow: {
        glow: "0 0 20px rgba(57, 255, 110, 0.15)",
        "glow-blue": "0 0 20px rgba(0, 180, 216, 0.1)",
        "glow-red": "0 0 15px rgba(255, 71, 87, 0.15)",
      },
    },
  },
  plugins: [],
};
