import {
  useCharacterStore,
  PRIORITY_LEVELS,
  PRIORITY_CATEGORIES,
  PRIORITY_TABLE,
  type PriorityCategory,
  type PriorityLevel,
} from "../store/characterStore";

export default function PriorityPanel() {
  const draft = useCharacterStore((s) => s.draft);
  const setPriority = useCharacterStore((s) => s.setPriority);
  const validate = useCharacterStore((s) => s.validate);

  if (!draft || !draft.priority_selection) return null;

  const selection = draft.priority_selection;

  // Track which levels are already used
  const usedLevels = new Map<PriorityLevel, PriorityCategory>();
  for (const cat of PRIORITY_CATEGORIES) {
    const level = selection[cat.key];
    usedLevels.set(level, cat.key);
  }

  const handleChange = (category: PriorityCategory, level: PriorityLevel) => {
    // Swap: if this level is already used by another category, swap them
    const currentLevel = selection[category];
    const otherCategory = usedLevels.get(level);

    if (otherCategory && otherCategory !== category) {
      // Swap the two
      setPriority(otherCategory, currentLevel);
    }
    setPriority(category, level);
    validate();
  };

  return (
    <div>
      <h2 className="text-xl font-semibold mb-4">Priority Selection</h2>
      <p className="text-sm text-gray-400 mb-4">
        Assign each priority level (A-E) to exactly one category.
        Selecting a level already in use will swap the two categories.
      </p>

      <div className="overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b border-gray-700">
              <th className="text-left py-2 pr-4 text-gray-400 font-medium">
                Category
              </th>
              {PRIORITY_LEVELS.map((level) => (
                <th
                  key={level}
                  className="py-2 px-3 text-center text-gray-400 font-medium"
                >
                  {level}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {PRIORITY_CATEGORIES.map((cat) => (
              <tr key={cat.key} className="border-b border-gray-800">
                <td className="py-2 pr-4 font-medium">{cat.label}</td>
                {PRIORITY_LEVELS.map((level) => {
                  const isSelected = selection[cat.key] === level;
                  const description = PRIORITY_TABLE[cat.key][level];

                  return (
                    <td key={level} className="py-2 px-1 text-center">
                      <button
                        onClick={() => handleChange(cat.key, level)}
                        className={`w-full px-2 py-1.5 rounded text-xs transition-colors ${
                          isSelected
                            ? "bg-blue-600 text-white font-semibold"
                            : "bg-gray-800 text-gray-400 hover:bg-gray-700"
                        }`}
                        title={description}
                      >
                        <div>{description}</div>
                      </button>
                    </td>
                  );
                })}
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {/* Summary of current allocation */}
      <div className="mt-4 grid grid-cols-2 sm:grid-cols-3 gap-2 text-sm">
        {PRIORITY_CATEGORIES.map((cat) => {
          const level = selection[cat.key];
          const desc = PRIORITY_TABLE[cat.key][level];
          return (
            <div key={cat.key} className="bg-gray-800 rounded px-3 py-2">
              <span className="text-gray-400">{cat.label}: </span>
              <span className="text-blue-400 font-mono">{level}</span>
              <span className="text-gray-500 text-xs ml-1">({desc})</span>
            </div>
          );
        })}
      </div>
    </div>
  );
}
