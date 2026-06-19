import {
  useCharacterStore,
  PRIORITY_LEVELS,
  PRIORITY_CATEGORIES,
  PRIORITY_TABLE,
  type PriorityCategory,
  type PriorityLevel,
  type MagicTradition,
} from "../store/characterStore";

const TRADITIONS: { value: MagicTradition; label: string; desc: string }[] = [
  { value: "Magician", label: "Magician", desc: "Casts spells and summons spirits" },
  { value: "Adept", label: "Adept", desc: "Channels magic into physical power (Adept Powers)" },
  { value: "MysticAdept", label: "Mystic Adept", desc: "Splits magic between spells and Adept Powers" },
  { value: "Technomancer", label: "Technomancer", desc: "Uses Resonance and Complex Forms instead of magic" },
];

export default function PriorityPanel() {
  const draft = useCharacterStore((s) => s.draft);
  const setPriority = useCharacterStore((s) => s.setPriority);
  const setMagicTradition = useCharacterStore((s) => s.setMagicTradition);
  const validate = useCharacterStore((s) => s.validate);

  if (!draft || !draft.priority_selection) return null;

  const selection = draft.priority_selection;

  const usedLevels = new Map<PriorityLevel, PriorityCategory>();
  for (const cat of PRIORITY_CATEGORIES) {
    const level = selection[cat.key];
    usedLevels.set(level, cat.key);
  }

  const handleChange = (category: PriorityCategory, level: PriorityLevel) => {
    const currentLevel = selection[category];
    const otherCategory = usedLevels.get(level);

    if (otherCategory && otherCategory !== category) {
      setPriority(otherCategory, currentLevel);
    }
    setPriority(category, level);
    validate();
  };

  return (
    <div>
      <h2 className="text-xl font-semibold mb-4 text-cyber-heading">
        // Priority Selection
      </h2>
      <p className="text-sm text-cyber-text-dim mb-4 font-mono">
        Assign each priority level (A-E) to exactly one category. Selecting a
        level already in use will swap the two categories.
      </p>

      <div className="overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b border-cyber-border">
              <th className="text-left py-2 pr-4 text-cyber-text-dim font-mono font-medium text-xs uppercase tracking-wider">
                Category
              </th>
              {PRIORITY_LEVELS.map((level) => (
                <th
                  key={level}
                  className="py-2 px-3 text-center text-cyber-green font-mono font-medium"
                >
                  {level}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {PRIORITY_CATEGORIES.map((cat) => (
              <tr key={cat.key} className="border-b border-cyber-border/50">
                <td className="py-2 pr-4 font-medium text-cyber-heading">
                  {cat.label}
                </td>
                {PRIORITY_LEVELS.map((level) => {
                  const isSelected = selection[cat.key] === level;
                  const description = PRIORITY_TABLE[cat.key][level];

                  return (
                    <td key={level} className="py-2 px-1 text-center">
                      <button
                        onClick={() => handleChange(cat.key, level)}
                        className={`w-full px-2 py-1.5 rounded text-xs font-mono transition-all ${
                          isSelected
                            ? "bg-cyber-green-dim border border-cyber-green text-cyber-green shadow-glow font-semibold"
                            : "bg-cyber-card border border-cyber-border text-cyber-text-dim hover:border-cyber-border-bright"
                        }`}
                        title={description}
                      >
                        {description}
                      </button>
                    </td>
                  );
                })}
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {/* Summary */}
      <div className="mt-4 grid grid-cols-2 sm:grid-cols-3 gap-2 text-sm">
        {PRIORITY_CATEGORIES.map((cat) => {
          const level = selection[cat.key];
          const desc = PRIORITY_TABLE[cat.key][level];
          return (
            <div
              key={cat.key}
              className="bg-cyber-card border border-cyber-border rounded px-3 py-2"
            >
              <span className="text-cyber-text-dim font-mono">
                {cat.label}:{" "}
              </span>
              <span className="text-cyber-green font-mono font-bold">
                {level}
              </span>
              <span className="text-cyber-text-dim text-xs ml-1">
                ({desc})
              </span>
            </div>
          );
        })}
      </div>

      {/* Awakened tradition selector — shown when magic priority is not E */}
      {selection.magic_or_resonance !== "E" && (
        <div className="mt-6">
          <h3 className="text-base font-semibold text-cyber-heading font-mono mb-2">
            // Awakened Tradition
          </h3>
          <p className="text-xs text-cyber-text-dim font-mono mb-3">
            Choose your tradition. This determines which abilities you can use.
          </p>
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
            {TRADITIONS.map((t) => {
              const isSelected = draft.magic_tradition === t.value;
              return (
                <button
                  key={t.value}
                  onClick={() => { setMagicTradition(t.value); validate(); }}
                  className={`text-left px-3 py-2 rounded border text-sm font-mono transition-all ${
                    isSelected
                      ? "bg-cyber-green-dim border-cyber-green text-cyber-green shadow-glow"
                      : "bg-cyber-card border-cyber-border text-cyber-text-dim hover:border-cyber-border-bright"
                  }`}
                >
                  <div className="font-semibold">{t.label}</div>
                  <div className="text-xs mt-0.5 opacity-70">{t.desc}</div>
                </button>
              );
            })}
          </div>
          {!draft.magic_tradition && (
            <p className="mt-2 text-cyber-red text-xs font-mono">
              Tradition required — choose one above.
            </p>
          )}
        </div>
      )}
    </div>
  );
}
