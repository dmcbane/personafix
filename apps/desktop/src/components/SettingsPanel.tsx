import {
  useSettingsStore,
  type Theme,
  type AppFont,
  type FontSize,
} from "../store/settingsStore";

const THEMES: { id: Theme; label: string; desc: string; swatch: string }[] = [
  { id: "cyber",        label: "Cyber Dark",     desc: "Neon cyberpunk default",          swatch: "#39ff6e" },
  { id: "high-contrast",label: "High Contrast",  desc: "Maximum contrast for low vision", swatch: "#00ff46" },
  { id: "deuteranopia", label: "No Red/Green",   desc: "Safe for deuteranopia/protanopia",swatch: "#00b4ff" },
  { id: "tritanopia",   label: "No Blue/Yellow", desc: "Safe for tritanopia",             swatch: "#e040fb" },
  { id: "light",        label: "Light",          desc: "Light background",                swatch: "#16a34a" },
];

const FONTS: { id: AppFont; label: string; desc: string }[] = [
  { id: "mono",         label: "Fira Code",      desc: "Monospace (default)" },
  { id: "sans",         label: "System Sans",    desc: "Clean proportional text" },
  { id: "opendyslexic", label: "OpenDyslexic",   desc: "Dyslexia-friendly" },
];

const FONT_SIZES: { id: FontSize; label: string }[] = [
  { id: "sm", label: "S" },
  { id: "md", label: "M" },
  { id: "lg", label: "L" },
  { id: "xl", label: "XL" },
];

export default function SettingsPanel({ onClose }: { onClose: () => void }) {
  const { theme, font, fontSize, setTheme, setFont, setFontSize } =
    useSettingsStore();

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center"
      onClick={onClose}
    >
      {/* Backdrop */}
      <div className="absolute inset-0 bg-cyber-bg/80 backdrop-blur-sm" />

      {/* Panel */}
      <div
        className="relative z-10 w-full max-w-sm bg-cyber-card border border-cyber-border rounded-lg shadow-glow p-5 space-y-5"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between">
          <h2 className="text-cyber-heading font-semibold text-base font-mono">
            // Settings
          </h2>
          <button
            onClick={onClose}
            className="text-cyber-text-dim hover:text-cyber-text transition-colors text-lg leading-none"
            aria-label="Close settings"
          >
            ✕
          </button>
        </div>

        {/* Color Theme */}
        <section>
          <p className="text-xs font-mono text-cyber-text-dim uppercase tracking-wider mb-2">
            Color Theme
          </p>
          <div className="space-y-1">
            {THEMES.map((t) => (
              <button
                key={t.id}
                onClick={() => setTheme(t.id)}
                className={`w-full flex items-center gap-3 px-3 py-2 rounded text-sm transition-colors ${
                  theme === t.id
                    ? "bg-cyber-green-dim/30 border border-cyber-green text-cyber-text"
                    : "bg-cyber-surface border border-cyber-border text-cyber-text-dim hover:text-cyber-text hover:border-cyber-border-bright"
                }`}
              >
                <span
                  className="w-3 h-3 rounded-full shrink-0 border border-cyber-border-bright"
                  style={{ backgroundColor: t.swatch }}
                />
                <span className="font-mono flex-1 text-left">{t.label}</span>
                <span className="text-xs text-cyber-text-dim hidden sm:block">
                  {t.desc}
                </span>
                {theme === t.id && (
                  <span className="text-cyber-green text-xs shrink-0">✓</span>
                )}
              </button>
            ))}
          </div>
        </section>

        {/* Font */}
        <section>
          <p className="text-xs font-mono text-cyber-text-dim uppercase tracking-wider mb-2">
            Font
          </p>
          <div className="space-y-1">
            {FONTS.map((f) => (
              <button
                key={f.id}
                onClick={() => setFont(f.id)}
                className={`w-full flex items-center gap-3 px-3 py-2 rounded text-sm transition-colors ${
                  font === f.id
                    ? "bg-cyber-green-dim/30 border border-cyber-green text-cyber-text"
                    : "bg-cyber-surface border border-cyber-border text-cyber-text-dim hover:text-cyber-text hover:border-cyber-border-bright"
                }`}
              >
                <span className="flex-1 text-left font-mono">{f.label}</span>
                <span className="text-xs text-cyber-text-dim">{f.desc}</span>
                {font === f.id && (
                  <span className="text-cyber-green text-xs shrink-0">✓</span>
                )}
              </button>
            ))}
          </div>
        </section>

        {/* Font size */}
        <section>
          <p className="text-xs font-mono text-cyber-text-dim uppercase tracking-wider mb-2">
            Font Size
          </p>
          <div className="flex gap-2">
            {FONT_SIZES.map((s) => (
              <button
                key={s.id}
                onClick={() => setFontSize(s.id)}
                className={`flex-1 py-1.5 rounded text-sm font-mono transition-colors border ${
                  fontSize === s.id
                    ? "bg-cyber-green-dim/30 border-cyber-green text-cyber-green"
                    : "border-cyber-border text-cyber-text-dim hover:border-cyber-border-bright hover:text-cyber-text"
                }`}
              >
                {s.label}
              </button>
            ))}
          </div>
        </section>
      </div>
    </div>
  );
}
