import { useState } from "react";
import { useCharacterStore, type Quality } from "../store/characterStore";
import { useGameDataStore } from "../store/gameDataStore";

const FALLBACK_QUALITIES: Omit<Quality, "id">[] = [
  { name: "Ambidextrous", quality_type: "Positive", cost: 5, source: "SR4", page: "90", improvements: [], incompatible_with: [] },
  { name: "Analytical Mind", quality_type: "Positive", cost: 5, source: "SR4", page: "90", improvements: [], incompatible_with: [] },
  { name: "Aptitude", quality_type: "Positive", cost: 10, source: "SR4", page: "90", improvements: [], incompatible_with: [] },
  { name: "Lucky", quality_type: "Positive", cost: 20, source: "SR4", page: "90", improvements: [], incompatible_with: ["bad_luck"] },
  { name: "Toughness", quality_type: "Positive", cost: 10, source: "SR4", page: "90", improvements: [], incompatible_with: [] },
  { name: "High Pain Tolerance", quality_type: "Positive", cost: 5, source: "SR4", page: "90", improvements: [], incompatible_with: [] },
  { name: "Magic Resistance", quality_type: "Positive", cost: 5, source: "SR4", page: "90", improvements: [], incompatible_with: [] },
  { name: "Addiction (Mild)", quality_type: "Negative", cost: 5, source: "SR4", page: "91", improvements: [], incompatible_with: [] },
  { name: "Addiction (Moderate)", quality_type: "Negative", cost: 10, source: "SR4", page: "91", improvements: [], incompatible_with: [] },
  { name: "Bad Luck", quality_type: "Negative", cost: 20, source: "SR4", page: "91", improvements: [], incompatible_with: ["lucky"] },
  { name: "SINner", quality_type: "Negative", cost: 5, source: "SR4", page: "91", improvements: [], incompatible_with: [] },
  { name: "Gremlins", quality_type: "Negative", cost: 5, source: "SR4", page: "91", improvements: [], incompatible_with: [] },
  { name: "Combat Paralysis", quality_type: "Negative", cost: 20, source: "SR4", page: "91", improvements: [], incompatible_with: [] },
];

const FILTER_BTN = (active: boolean) =>
  `px-2.5 py-1 rounded text-xs font-mono transition-all ${
    active
      ? "bg-cyber-green-dim border border-cyber-green text-cyber-green shadow-glow"
      : "bg-cyber-card border border-cyber-border text-cyber-text-dim hover:border-cyber-border-bright"
  }`;

export default function QualityPanel() {
  const draft = useCharacterStore((s) => s.draft);
  const addQuality = useCharacterStore((s) => s.addQuality);
  const removeQuality = useCharacterStore((s) => s.removeQuality);
  const validate = useCharacterStore((s) => s.validate);
  const gameQualities = useGameDataStore((s) => s.qualities);
  const gameDataLoaded = useGameDataStore((s) => s.loaded);

  const [filter, setFilter] = useState<"All" | "Positive" | "Negative">("All");
  const [search, setSearch] = useState("");
  const [selected, setSelected] = useState("");

  if (!draft) return null;

  const costUnit = draft.edition === "SR4" ? "BP" : "karma";
  const limit = draft.edition === "SR4" ? 35 : 25;

  const posCost = draft.qualities
    .filter((q) => q.quality_type === "Positive")
    .reduce((sum, q) => sum + q.cost, 0);
  const negCost = draft.qualities
    .filter((q) => q.quality_type === "Negative")
    .reduce((sum, q) => sum + q.cost, 0);

  const qualitySource: Omit<Quality, "id">[] = gameDataLoaded
    ? gameQualities.map((gq) => ({
        name: gq.name,
        quality_type: gq.quality_type,
        cost: gq.cost,
        source: gq.source,
        page: gq.page,
        improvements: [],
        incompatible_with: gq.incompatible_with,
      }))
    : FALLBACK_QUALITIES;

  const available = qualitySource.filter(
    (q) =>
      !draft.qualities.some((dq) => dq.name === q.name) &&
      (filter === "All" || q.quality_type === filter) &&
      (search === "" || q.name.toLowerCase().includes(search.toLowerCase())),
  );

  const handleAdd = () => {
    const q = available.find((q) => q.name === selected);
    if (!q) return;
    addQuality({
      ...q,
      id: q.name.toLowerCase().replace(/ /g, "_").replace(/[()]/g, ""),
    });
    setSelected("");
    validate();
  };

  return (
    <div>
      <h2 className="text-xl font-semibold mb-4 text-cyber-heading">
        // Qualities
      </h2>

      {/* Stats */}
      <div className="flex gap-4 text-sm text-cyber-text-dim mb-4 font-mono">
        <span>
          Positive:{" "}
          <span className={posCost > limit ? "text-cyber-red" : "text-cyber-green"}>
            {posCost}
          </span>
          /{limit} {costUnit}
        </span>
        <span>
          Negative:{" "}
          <span className={negCost > limit ? "text-cyber-red" : "text-cyber-red"}>
            {negCost}
          </span>
          /{limit} {costUnit}
        </span>
        {gameDataLoaded && (
          <span className="text-cyber-green-dim">
            ({gameQualities.length} from game data)
          </span>
        )}
      </div>

      {/* Type filter */}
      <div className="flex gap-1.5 mb-3 flex-wrap">
        {(["All", "Positive", "Negative"] as const).map((f) => (
          <button key={f} onClick={() => { setFilter(f); setSelected(""); }}
            className={FILTER_BTN(filter === f)}>
            {f}
          </button>
        ))}
      </div>

      {/* Search + select + Add */}
      <div className="flex gap-2 mb-6">
        <input
          type="text"
          value={search}
          onChange={(e) => { setSearch(e.target.value); setSelected(""); }}
          placeholder="Search qualities…"
          className="bg-cyber-card border border-cyber-border rounded px-3 py-1.5 text-sm w-40"
        />
        <select
          value={selected}
          onChange={(e) => setSelected(e.target.value)}
          className="bg-cyber-card border border-cyber-border rounded px-3 py-1.5 text-sm flex-1 text-cyber-text"
        >
          <option value="">Select a quality…</option>
          {available.map((q) => (
            <option key={q.name} value={q.name}>
              {q.quality_type === "Positive" ? "+" : "-"} {q.name} ({q.cost} {costUnit})
            </option>
          ))}
        </select>
        <button
          onClick={handleAdd}
          disabled={!selected}
          className="px-4 py-1.5 bg-cyber-green-dim hover:bg-cyber-green/20 border border-cyber-green-dim hover:border-cyber-green rounded text-sm disabled:opacity-50 text-cyber-green font-mono transition-all"
        >
          Add
        </button>
      </div>

      {/* Selected qualities */}
      {draft.qualities.length === 0 ? (
        <p className="text-cyber-text-dim text-sm font-mono">No qualities selected.</p>
      ) : (
        <div className="space-y-1">
          {draft.qualities.map((q) => (
            <div
              key={q.id}
              className="flex items-center gap-2 bg-cyber-card border border-cyber-border rounded px-3 py-1.5 text-sm"
            >
              <span className={q.quality_type === "Positive" ? "text-cyber-green" : "text-cyber-red"}>
                {q.quality_type === "Positive" ? "+" : "-"}
              </span>
              <span className="flex-1">{q.name}</span>
              <span className="text-cyber-text-dim font-mono text-xs">
                {q.cost} {costUnit}
              </span>
              <button
                onClick={() => { removeQuality(q.id); validate(); }}
                className="text-cyber-red hover:text-cyber-red/80 ml-2 transition-colors"
              >
                X
              </button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
