import { useState } from "react";
import { useCharacterStore, type Quality } from "../store/characterStore";
import { useGameDataStore } from "../store/gameDataStore";

const FALLBACK_QUALITIES: Omit<Quality, "id">[] = [
  { name: "Ambidextrous", quality_type: "Positive", cost: 5, source: "SR4", page: "90", improvements: [], incompatible_with: [] },
  { name: "Analytical Mind", quality_type: "Positive", cost: 5, source: "SR4", page: "90", improvements: [], incompatible_with: [] },
  { name: "Aptitude", quality_type: "Positive", cost: 10, source: "SR4", page: "90", improvements: [], incompatible_with: [] },
  { name: "Toughness", quality_type: "Positive", cost: 10, source: "SR4", page: "90", improvements: [], incompatible_with: [] },
  { name: "High Pain Tolerance", quality_type: "Positive", cost: 5, source: "SR4", page: "90", improvements: [], incompatible_with: [] },
  { name: "Magic Resistance", quality_type: "Positive", cost: 5, source: "SR4", page: "90", improvements: [], incompatible_with: [] },
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
  const gameQualities = useGameDataStore((s) => s.qualities);
  const gameDataLoaded = useGameDataStore((s) => s.loaded);
  const [filter, setFilter] = useState<"all" | "Positive" | "Negative">("all");
  const [search, setSearch] = useState("");

  if (!draft) return null;

  const posBP = draft.qualities
    .filter((q) => q.quality_type === "Positive")
    .reduce((sum, q) => sum + q.cost, 0);
  const negBP = draft.qualities
    .filter((q) => q.quality_type === "Negative")
    .reduce((sum, q) => sum + q.cost, 0);

  // Use game data if loaded
  const qualitySource: Omit<Quality, "id">[] = gameDataLoaded
    ? gameQualities.map((gq) => ({
        name: gq.name,
        quality_type: gq.quality_type,
        cost: gq.cost,
        source: gq.source,
        page: gq.page,
        improvements: [],
        incompatible_with: [],
      }))
    : FALLBACK_QUALITIES;

  const available = qualitySource
    .filter(
      (sq) =>
        !draft.qualities.some((q) => q.name === sq.name) &&
        (filter === "all" || sq.quality_type === filter) &&
        (search === "" || sq.name.toLowerCase().includes(search.toLowerCase())),
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
      <h2 className="text-xl font-semibold mb-4 text-cyber-heading">
        // Qualities
      </h2>
      <div className="flex gap-4 text-sm text-cyber-text-dim mb-4 font-mono">
        <span>
          Positive:{" "}
          <span className="text-cyber-green">{posBP}</span>
          {draft.edition === "SR4" ? "/35 BP" : "/25 karma"}
        </span>
        <span>
          Negative:{" "}
          <span className="text-cyber-red">{negBP}</span>
          {draft.edition === "SR4" ? "/35 BP" : "/25 karma"}
        </span>
        {gameDataLoaded && (
          <span className="text-cyber-green-dim">
            ({gameQualities.length} from game data)
          </span>
        )}
      </div>

      {/* Current qualities */}
      {draft.qualities.length > 0 && (
        <div className="space-y-1 mb-4">
          {draft.qualities.map((q) => (
            <div
              key={q.id}
              className="flex items-center gap-2 bg-cyber-card border border-cyber-border rounded px-3 py-1.5 text-sm"
            >
              <span
                className={
                  q.quality_type === "Positive"
                    ? "text-cyber-green"
                    : "text-cyber-red"
                }
              >
                {q.quality_type === "Positive" ? "+" : "-"}
              </span>
              <span className="flex-1">{q.name}</span>
              <span className="text-cyber-text-dim font-mono">
                {q.cost} {draft.edition === "SR4" ? "BP" : "karma"}
              </span>
              <button
                onClick={() => {
                  removeQuality(q.id);
                  validate();
                }}
                className="text-cyber-red hover:text-cyber-red/80 ml-2 transition-colors"
              >
                X
              </button>
            </div>
          ))}
        </div>
      )}

      {/* Filter + search */}
      <div className="flex gap-2 mb-3">
        {(["all", "Positive", "Negative"] as const).map((f) => (
          <button
            key={f}
            onClick={() => setFilter(f)}
            className={`px-3 py-1 rounded text-xs font-mono transition-all ${
              filter === f
                ? "bg-cyber-green-dim border border-cyber-green text-cyber-green shadow-glow"
                : "bg-cyber-card border border-cyber-border text-cyber-text-dim hover:border-cyber-border-bright"
            }`}
          >
            {f === "all" ? "All" : f}
          </button>
        ))}
        <input
          type="text"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          placeholder="Search..."
          className="bg-cyber-card border border-cyber-border rounded px-3 py-1 text-xs flex-1 ml-2"
        />
      </div>

      {/* Available qualities */}
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-1 max-h-64 overflow-y-auto">
        {available.slice(0, 50).map((q) => (
          <button
            key={q.name}
            onClick={() => handleAdd(q)}
            className="text-left bg-cyber-card border border-cyber-border hover:border-cyber-border-bright rounded px-3 py-1.5 text-sm transition-colors"
          >
            <span
              className={
                q.quality_type === "Positive"
                  ? "text-cyber-green"
                  : "text-cyber-red"
              }
            >
              {q.quality_type === "Positive" ? "+" : "-"}
            </span>{" "}
            {q.name}{" "}
            <span className="text-cyber-text-dim font-mono text-xs">
              {q.cost} {draft.edition === "SR4" ? "BP" : "karma"}
            </span>
          </button>
        ))}
        {available.length > 50 && (
          <p className="text-cyber-text-dim text-xs font-mono col-span-2 py-2">
            Showing 50 of {available.length} — use search to filter
          </p>
        )}
      </div>
    </div>
  );
}
