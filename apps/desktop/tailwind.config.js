/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        cyber: {
          bg:             "rgb(var(--c-bg) / <alpha-value>)",
          surface:        "rgb(var(--c-surface) / <alpha-value>)",
          card:           "rgb(var(--c-card) / <alpha-value>)",
          "card-hover":   "rgb(var(--c-card-hover) / <alpha-value>)",
          border:         "rgb(var(--c-border) / <alpha-value>)",
          "border-bright":"rgb(var(--c-border-bright) / <alpha-value>)",
          text:           "rgb(var(--c-text) / <alpha-value>)",
          "text-dim":     "rgb(var(--c-text-dim) / <alpha-value>)",
          heading:        "rgb(var(--c-heading) / <alpha-value>)",
          green:          "rgb(var(--c-green) / <alpha-value>)",
          "green-dim":    "rgb(var(--c-green-dim) / <alpha-value>)",
          blue:           "rgb(var(--c-blue) / <alpha-value>)",
          purple:         "rgb(var(--c-purple) / <alpha-value>)",
          red:            "rgb(var(--c-red) / <alpha-value>)",
          "red-dim":      "rgb(var(--c-red-dim) / <alpha-value>)",
          yellow:         "rgb(var(--c-yellow) / <alpha-value>)",
          "yellow-dim":   "rgb(var(--c-yellow-dim) / <alpha-value>)",
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
        glow:      "0 0 20px var(--shadow-glow)",
        "glow-blue": "0 0 20px var(--shadow-glow-blue)",
        "glow-red":  "0 0 15px var(--shadow-glow-red)",
      },
    },
  },
  plugins: [],
};
