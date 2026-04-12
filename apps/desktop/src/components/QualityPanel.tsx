import { useState } from "react";
import { useCharacterStore, type Quality } from "../store/characterStore";

const SAMPLE_QUALITIES: Omit<Quality, "id">[] = [
  // Positive
  { name: "Ambidextrous", quality_type: "Positive", cost: 5, source: "SR4", page: "90", improvements: [], incompatible_with: [] },
  { name: "Analytical Mind", quality_type: "Positive", cost: 5, source: "SR4", page: "90", improvements: [], incompatible_with: [] },
  { name: "Aptitude", quality_type: "Positive", cost: 10, source: "SR4", page: "90", improvements: [], incompatible_with: [] },
  { name: "Toughness", quality_type: "Positive", cost: 10, source: "SR4", page: "90", improvements: [], incompatible_with: [] },
  { name: "High Pain Tolerance", quality_type: "Positive", cost: 5, source: "SR4", page: "90", improvements: [], incompatible_with: [] },
  { name: "Magic Resistance", quality_type: "Positive", cost: 5, source: "SR4", page: "90", improvements: [], incompatible_with: [] },
  // Negative
  { name: "Addiction (Mild)", quality_type: "Negative", cost: 5, source: "SR4", page: "91", improvements: [], incompatible_with: [] },
  { name: "Addiction (Moderate)", quality_type: "Negative", cost: 10, source: "SR4", page: "91", improvements: [], incompatible_with: [] },
  { name: "Bad Luck", quality_type: "Negative", cost: 20, source: "SR4", page: "91", improvements: [], incompatible_with: [] },
  { name: "SINner", quality_type: "Negative", cost: 5, source: "SR4", page: "91", improvements: [], incompatible_with: [] },
  { name: "Gremlins", quality_type: "Negative", cost: 5, source: "SR4", page: "91", improvements: [], incompatible_with: [] },
  { name: "Combat Paralysis", quality_type: "Negative", cost: 20, source: "SR4", page: "91", improvements: [], incompatible_with: [] },
];

export default function QualityPanel() {
  const draft = useCharacterStore((s) => s.draft);
  const addQuality = useCharacterStore((s) => s.addQuality);
  const removeQuality = useCharacterStore((s) => s.removeQuality);
  const validate = useCharacterStore((s) => s.validate);
  const [filter, setFilter] = useState<"all" | "Positive" | "Negative">("all");

  if (!draft) return null;

  const posBP = draft.qualities
    .filter((q) => q.quality_type === "Positive")
    .reduce((sum, q) => sum + q.cost, 0);
  const negBP = draft.qualities
    .filter((q) => q.quality_type === "Negative")
    .reduce((sum, q) => sum + q.cost, 0);

  const available = SAMPLE_QUALITIES.filter(
    (sq) =>
      !draft.qualities.some((q) => q.name === sq.name) &&
      (filter === "all" || sq.quality_type === filter),
  );

  const handleAdd = (q: Omit<Quality, "id">) => {
    addQuality({
      ...q,
      id: q.name.toLowerCase().replace(/ /g, "_").replace(/[()]/g, ""),
    });
    validate();
  };

  return (
    <div>
      <h2 className="text-xl font-semibold mb-4">Qualities</h2>
      <div className="flex gap-4 text-sm text-gray-400 mb-4">
        <span>
          Positive: <span className="text-green-400 font-mono">{posBP}</span>
          {draft.edition === "SR4" ? "/35 BP" : "/25 karma"}
        </span>
        <span>
          Negative: <span className="text-red-400 font-mono">{negBP}</span>
          {draft.edition === "SR4" ? "/35 BP" : "/25 karma"}
        </span>
      </div>

      {/* Current qualities */}
      {draft.qualities.length > 0 && (
        <div className="space-y-1 mb-4">
          {draft.qualities.map((q) => (
            <div
              key={q.id}
              className="flex items-center gap-2 bg-gray-800 rounded px-3 py-1.5 text-sm"
            >
              <span
                className={
                  q.quality_type === "Positive"
                    ? "text-green-400"
                    : "text-red-400"
                }
              >
                {q.quality_type === "Positive" ? "+" : "-"}
              </span>
              <span className="flex-1">{q.name}</span>
              <span className="text-gray-500 font-mono">{q.cost} BP</span>
              <button
                onClick={() => {
                  removeQuality(q.id);
                  validate();
                }}
                className="text-red-400 hover:text-red-300 ml-2"
              >
                ✕
              </button>
            </div>
          ))}
        </div>
      )}

      {/* Filter + add */}
      <div className="flex gap-2 mb-2">
        {(["all", "Positive", "Negative"] as const).map((f) => (
          <button
            key={f}
            onClick={() => setFilter(f)}
            className={`px-3 py-1 rounded text-xs ${
              filter === f
                ? "bg-blue-600 text-white"
                : "bg-gray-800 text-gray-400 hover:bg-gray-700"
            }`}
          >
            {f === "all" ? "All" : f}
          </button>
        ))}
      </div>
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-1">
        {available.map((q) => (
          <button
            key={q.name}
            onClick={() => handleAdd(q)}
            className="text-left bg-gray-800 hover:bg-gray-700 rounded px-3 py-1.5 text-sm"
          >
            <span
              className={
                q.quality_type === "Positive"
                  ? "text-green-400"
                  : "text-red-400"
              }
            >
              {q.quality_type === "Positive" ? "+" : "-"}
            </span>{" "}
            {q.name}{" "}
            <span className="text-gray-500 font-mono text-xs">{q.cost} BP</span>
          </button>
        ))}
      </div>
    </div>
  );
}
