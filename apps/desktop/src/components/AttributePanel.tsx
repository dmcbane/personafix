import {
  useCharacterStore,
  ATTRIBUTE_NAMES,
  SR5_ATTR_POINTS,
  SR5_MAGIC_STARTING,
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
  const setMagic = useCharacterStore((s) => s.setMagic);
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

  const isSR5 = draft.edition === "SR5";
  const sr5AttrSpent = isSR5
    ? ATTRIBUTE_NAMES.filter((a) => a !== "edge").reduce((sum, attr) => {
        return sum + (draft.attributes[attr] - limits[attr][0]);
      }, 0)
    : 0;
  const sr5AttrAlloc = isSR5 && draft.priority_selection
    ? SR5_ATTR_POINTS[draft.priority_selection.attributes]
    : 0;
  const magicPriorityMax = isSR5 && draft.priority_selection
    ? SR5_MAGIC_STARTING[draft.priority_selection.magic_or_resonance]
    : 0;

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
      {isSR5 && (
        <p className="text-sm text-cyber-text-dim mb-4 font-mono">
          Attribute points:{" "}
          <span
            className={
              sr5AttrSpent > sr5AttrAlloc ? "text-cyber-red" : "text-cyber-green"
            }
          >
            {sr5AttrSpent}
          </span>{" "}
          / {sr5AttrAlloc}
          <span className="text-xs ml-3 text-cyber-text-dim">
            (Edge from metatype pool — not counted here)
          </span>
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

      {/* SR5 Magic / Resonance row */}
      {isSR5 && magicPriorityMax > 0 && (
        <div className="mt-4">
          <h3 className="text-sm font-mono text-cyber-text-dim mb-2">
            Magic (from priority — max {magicPriorityMax})
          </h3>
          <div className="flex items-center gap-3 bg-cyber-card border border-cyber-border rounded px-3 py-2 w-48">
            <span className="text-cyber-purple font-mono w-10 text-sm font-semibold">
              MAG
            </span>
            <button
              onClick={() => {
                const cur = draft.attributes.magic ?? 0;
                setMagic(Math.max(1, cur - 1));
                validate();
              }}
              disabled={(draft.attributes.magic ?? 0) <= 1}
              className="w-7 h-7 rounded bg-cyber-surface border border-cyber-border hover:border-cyber-green-dim disabled:opacity-30 text-sm text-cyber-text transition-colors"
            >
              -
            </button>
            <span className="font-mono text-lg w-6 text-center text-cyber-heading">
              {draft.attributes.magic ?? 0}
            </span>
            <button
              onClick={() => {
                const cur = draft.attributes.magic ?? 0;
                setMagic(Math.min(magicPriorityMax, cur + 1));
                validate();
              }}
              disabled={(draft.attributes.magic ?? 0) >= magicPriorityMax}
              className="w-7 h-7 rounded bg-cyber-surface border border-cyber-border hover:border-cyber-green-dim disabled:opacity-30 text-sm text-cyber-text transition-colors"
            >
              +
            </button>
            <span className="text-cyber-text-dim text-xs ml-1 font-mono">
              1-{magicPriorityMax}
            </span>
          </div>
        </div>
      )}
      {isSR5 && magicPriorityMax === 0 && (
        <p className="mt-3 text-xs text-cyber-text-dim font-mono">
          Mundane priority — no magic or resonance attribute
        </p>
      )}
    </div>
  );
}
