import { useState } from "react";
import {
  useCharacterStore,
  type DraftAugmentation,
  type AugmentationGrade,
  type AugmentationType,
  GRADE_MULTIPLIER,
} from "../store/characterStore";
import { useGameDataStore } from "../store/gameDataStore";

const GRADES: AugmentationGrade[] = ["Standard", "Alpha", "Beta", "Delta", "Used"];

const GRADE_LABEL: Record<AugmentationGrade, string> = {
  Standard: "Std",
  Alpha: "α",
  Beta: "β",
  Delta: "δ",
  Used: "Used",
};

function parseEssenceCost(raw: string, rating: number): number {
  const fv = raw.match(/^FixedValues\(([^)]+)\)$/i);
  if (fv) {
    const vals = fv[1].split(",").map(Number);
    return Math.max(0, Math.round((vals[0] ?? 0) * 100));
  }
  const expr = raw.replace(/Rating/gi, String(rating));
  try {
    // Local game data only — not user input
    // eslint-disable-next-line no-new-func
    const result = new Function(`return (${expr})`)() as number;
    return Math.max(0, Math.round(result * 100));
  } catch {
    return 0;
  }
}

function computeAugEssence(aug: DraftAugmentation): number {
  const mult = GRADE_MULTIPLIER[aug.grade] ?? 100;
  return Math.floor((aug.essence_cost * mult) / 100);
}

const FILTER_BTN = (active: boolean) =>
  `px-2.5 py-1 rounded text-xs font-mono transition-all ${
    active
      ? "bg-cyber-blue/20 border border-cyber-blue text-cyber-blue"
      : "bg-cyber-card border border-cyber-border text-cyber-text-dim hover:border-cyber-border-bright"
  }`;

const GRADE_BTN = (active: boolean) =>
  `px-2 py-0.5 rounded text-xs font-mono border transition-colors ${
    active
      ? "border-cyber-green text-cyber-green bg-cyber-green/10"
      : "border-cyber-border text-cyber-text-dim hover:border-cyber-border-bright"
  }`;

export default function AugmentationPanel() {
  const draft = useCharacterStore((s) => s.draft);
  const addAugmentation = useCharacterStore((s) => s.addAugmentation);
  const removeAugmentation = useCharacterStore((s) => s.removeAugmentation);
  const gameAugs = useGameDataStore((s) => s.augmentations);

  const [search, setSearch] = useState("");
  const [typeFilter, setTypeFilter] = useState<AugmentationType | "All">("All");
  const [grade, setGrade] = useState<AugmentationGrade>("Standard");
  const [rating, setRating] = useState(1);
  const [selected, setSelected] = useState("");

  if (!draft) return null;

  const essenceUsed = draft.augmentations.reduce(
    (sum, a) => sum + computeAugEssence(a),
    0,
  );
  const essenceRemaining = 600 - essenceUsed;

  const existingIds = new Set(
    draft.augmentations.map((a) => `${a.name}_${a.grade}`),
  );

  const available = gameAugs.filter((a) => {
    if (typeFilter !== "All" && a.augmentation_type !== typeFilter) return false;
    if (search && !a.name.toLowerCase().includes(search.toLowerCase())) return false;
    return !existingIds.has(`${a.name}_${grade}`);
  });

  const handleAdd = () => {
    const ga = available.find((a) => a.name === selected);
    if (!ga) return;
    const essenceCost = parseEssenceCost(ga.essence_cost, rating);
    const aug: DraftAugmentation = {
      id: `${ga.id}_${grade}`,
      name: ga.name,
      augmentation_type: ga.augmentation_type as AugmentationType,
      grade,
      essence_cost: essenceCost,
      availability: ga.availability,
      cost: parseFloat(ga.cost) || 0,
      source: ga.source,
      page: ga.page,
      improvements: [],
    };
    addAugmentation(aug);
    setSelected("");
  };

  const mult = GRADE_MULTIPLIER[grade] ?? 100;
  const selectedAug = available.find((a) => a.name === selected);
  const previewEssence = selectedAug
    ? Math.floor((parseEssenceCost(selectedAug.essence_cost, rating) * mult) / 100)
    : null;

  return (
    <div>
      <h2 className="text-xl font-semibold mb-4 text-cyber-heading">
        // Augmentations
      </h2>

      {/* Stats */}
      <div className="flex gap-4 text-sm text-cyber-text-dim mb-4 font-mono">
        <span>
          Essence:{" "}
          <span className={essenceRemaining <= 0 ? "text-cyber-red" : "text-cyber-blue"}>
            {(essenceRemaining / 100).toFixed(2)}
          </span>
          {" / 6.00"}
        </span>
        {draft.augmentations.length > 0 && (
          <span>({draft.augmentations.length} installed)</span>
        )}
      </div>

      {/* Type filter */}
      <div className="flex gap-1.5 mb-3 flex-wrap">
        {(["All", "Cyberware", "Bioware"] as const).map((t) => (
          <button key={t} onClick={() => { setTypeFilter(t); setSelected(""); }}
            className={FILTER_BTN(typeFilter === t)}>
            {t}
          </button>
        ))}
      </div>

      {/* Grade + Rating */}
      <div className="flex gap-2 items-center mb-3 flex-wrap">
        <span className="text-xs font-mono text-cyber-text-dim">Grade:</span>
        {GRADES.map((g) => (
          <button key={g} onClick={() => setGrade(g)} className={GRADE_BTN(grade === g)}>
            {GRADE_LABEL[g]}
          </button>
        ))}
        <span className="text-xs font-mono text-cyber-text-dim ml-2">Rating:</span>
        <input
          type="number"
          min={1}
          max={6}
          value={rating}
          onChange={(e) => setRating(Math.min(6, Math.max(1, Number(e.target.value))))}
          className="w-12 bg-cyber-card border border-cyber-border rounded px-1 py-0.5 text-center text-sm"
        />
        {previewEssence !== null && (
          <span className="text-xs font-mono text-cyber-blue ml-1">
            → {(previewEssence / 100).toFixed(2)}E
          </span>
        )}
      </div>

      {/* Search + select + Add */}
      <div className="flex gap-2 mb-6">
        <input
          type="text"
          value={search}
          onChange={(e) => { setSearch(e.target.value); setSelected(""); }}
          placeholder="Search augmentations…"
          className="bg-cyber-card border border-cyber-border rounded px-3 py-1.5 text-sm w-44"
        />
        <select
          value={selected}
          onChange={(e) => setSelected(e.target.value)}
          className="bg-cyber-card border border-cyber-border rounded px-3 py-1.5 text-sm flex-1 text-cyber-text"
        >
          <option value="">Select an augmentation…</option>
          {available.map((a) => {
            const ec = parseEssenceCost(a.essence_cost, rating);
            const adj = Math.floor((ec * mult) / 100);
            return (
              <option key={a.id} value={a.name}>
                {a.name} ({a.augmentation_type}, {(adj / 100).toFixed(2)}E)
              </option>
            );
          })}
        </select>
        <button
          onClick={handleAdd}
          disabled={!selected}
          className="px-4 py-1.5 bg-cyber-green-dim hover:bg-cyber-green/20 border border-cyber-green-dim hover:border-cyber-green rounded text-sm disabled:opacity-50 text-cyber-green font-mono transition-all"
        >
          Add
        </button>
      </div>

      {/* Installed list */}
      {draft.augmentations.length === 0 ? (
        <p className="text-cyber-text-dim text-sm font-mono">No augmentations installed.</p>
      ) : (
        <div className="space-y-1">
          {draft.augmentations.map((a) => (
            <div
              key={a.id}
              className="flex items-center gap-2 bg-cyber-card border border-cyber-border rounded px-3 py-2 text-sm"
            >
              <div className="flex-1 min-w-0">
                <span className="text-cyber-text font-medium">{a.name}</span>
                <span className="text-cyber-text-dim font-mono text-xs ml-2">
                  {a.augmentation_type}
                </span>
              </div>
              <span className="font-mono text-xs text-cyber-text-dim shrink-0">
                {GRADE_LABEL[a.grade]}{" "}
                <span className="text-cyber-blue">
                  -{(computeAugEssence(a) / 100).toFixed(2)}E
                </span>
              </span>
              <button
                onClick={() => removeAugmentation(a.id)}
                className="text-cyber-red hover:text-cyber-red/80 transition-colors ml-1 shrink-0"
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
