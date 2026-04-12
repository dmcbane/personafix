import {
  useCharacterStore,
  ATTRIBUTE_NAMES,
  type AttributeName,
} from "../store/characterStore";

const ATTR_LABELS: Record<AttributeName, string> = {
  body: "BOD",
  agility: "AGI",
  reaction: "REA",
  strength: "STR",
  willpower: "WIL",
  logic: "LOG",
  intuition: "INT",
  charisma: "CHA",
  edge: "EDG",
};

export default function AttributePanel() {
  const draft = useCharacterStore((s) => s.draft);
  const limits = useCharacterStore((s) => s.racialLimits);
  const setAttribute = useCharacterStore((s) => s.setAttribute);
  const validate = useCharacterStore((s) => s.validate);

  if (!draft || !limits) return null;

  const handleChange = (attr: AttributeName, value: number) => {
    setAttribute(attr, value);
    validate();
  };

  const bpCost = ATTRIBUTE_NAMES.reduce((total, attr) => {
    const min = limits[attr][0];
    const current = draft.attributes[attr];
    return total + (current - min) * 10;
  }, 0);

  return (
    <div>
      <h2 className="text-xl font-semibold mb-4 text-cyber-heading">
        // Attributes
      </h2>
      {draft.edition === "SR4" && (
        <p className="text-sm text-cyber-text-dim mb-4 font-mono">
          BP spent: <span className="text-cyber-green">{bpCost}</span>
        </p>
      )}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
        {ATTRIBUTE_NAMES.map((attr) => {
          const min = limits[attr][0];
          const max = limits[attr][1];
          const value = draft.attributes[attr];

          return (
            <div
              key={attr}
              className="flex items-center gap-3 bg-cyber-card border border-cyber-border rounded px-3 py-2"
            >
              <span className="text-cyber-blue font-mono w-10 text-sm font-semibold">
                {ATTR_LABELS[attr]}
              </span>
              <button
                onClick={() => handleChange(attr, Math.max(min, value - 1))}
                disabled={value <= min}
                className="w-7 h-7 rounded bg-cyber-surface border border-cyber-border hover:border-cyber-green-dim disabled:opacity-30 text-sm text-cyber-text transition-colors"
              >
                -
              </button>
              <span className="font-mono text-lg w-6 text-center text-cyber-heading">
                {value}
              </span>
              <button
                onClick={() => handleChange(attr, Math.min(max, value + 1))}
                disabled={value >= max}
                className="w-7 h-7 rounded bg-cyber-surface border border-cyber-border hover:border-cyber-green-dim disabled:opacity-30 text-sm text-cyber-text transition-colors"
              >
                +
              </button>
              <span className="text-cyber-text-dim text-xs ml-1 font-mono">
                {min}-{max}
              </span>
            </div>
          );
        })}
      </div>
    </div>
  );
}
