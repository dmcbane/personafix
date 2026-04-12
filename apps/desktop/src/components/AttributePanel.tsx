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

  // Calculate BP cost for SR4 (10 per point above racial min)
  const bpCost = ATTRIBUTE_NAMES.reduce((total, attr) => {
    const min = limits[attr][0];
    const current = draft.attributes[attr];
    return total + (current - min) * 10;
  }, 0);

  return (
    <div>
      <h2 className="text-xl font-semibold mb-4">Attributes</h2>
      {draft.edition === "SR4" && (
        <p className="text-sm text-gray-400 mb-4">
          BP spent on attributes: <span className="text-white font-mono">{bpCost}</span>
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
              className="flex items-center gap-3 bg-gray-800 rounded px-3 py-2"
            >
              <span className="text-gray-400 font-mono w-10 text-sm">
                {ATTR_LABELS[attr]}
              </span>
              <button
                onClick={() => handleChange(attr, Math.max(min, value - 1))}
                disabled={value <= min}
                className="w-7 h-7 rounded bg-gray-700 hover:bg-gray-600 disabled:opacity-30 text-sm"
              >
                -
              </button>
              <span className="font-mono text-lg w-6 text-center">{value}</span>
              <button
                onClick={() => handleChange(attr, Math.min(max, value + 1))}
                disabled={value >= max}
                className="w-7 h-7 rounded bg-gray-700 hover:bg-gray-600 disabled:opacity-30 text-sm"
              >
                +
              </button>
              <span className="text-gray-500 text-xs ml-1">
                ({min}-{max})
              </span>
            </div>
          );
        })}
      </div>
    </div>
  );
}
