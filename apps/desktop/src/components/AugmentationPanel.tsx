import { useState } from "react";
import {
  useCharacterStore,
  type DraftAugmentation,
  type AugmentationGrade,
  type AugmentationType,
  GRADE_MULTIPLIER,
} from "../store/characterStore";
import { useGameDataStore } from "../store/gameDataStore";

const GRADES: AugmentationGrade[] = [
  "Standard",
  "Alpha",
  "Beta",
  "Delta",
  "Used",
];

const GRADE_LABEL: Record<AugmentationGrade, string> = {
  Standard: "Std",
  Alpha: "α",
  Beta: "β",
  Delta: "δ",
  Used: "Used",
};

function parseEssenceCost(raw: string, rating: number): number {
  // FixedValues(a,b,...) — Standard grade = first value
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

export default function AugmentationPanel() {
  const draft = useCharacterStore((s) => s.draft);
  const addAugmentation = useCharacterStore((s) => s.addAugmentation);
  const removeAugmentation = useCharacterStore((s) => s.removeAugmentation);
  const gameAugs = useGameDataStore((s) => s.augmentations);

  const [search, setSearch] = useState("");
  const [typeFilter, setTypeFilter] = useState<AugmentationType | "All">("All");
  const [expandedId, setExpandedId] = useState<string | null>(null);
  const [grade, setGrade] = useState<AugmentationGrade>("Standard");
  const [rating, setRating] = useState(1);

  if (!draft) return null;

  const essenceUsed = draft.augmentations.reduce(
    (sum, a) => sum + computeAugEssence(a),
    0,
  );
  const essenceRemaining = 600 - essenceUsed;

  const existingIds = new Set(draft.augmentations.map((a) => a.id));

  const filtered = gameAugs.filter((a) => {
    if (typeFilter !== "All" && a.augmentation_type !== typeFilter) return false;
    if (
      search &&
      !a.name.toLowerCase().includes(search.toLowerCase())
    )
      return false;
    return true;
  });

  const handleAdd = (ga: (typeof gameAugs)[0]) => {
    const hasRating = /Rating/i.test(ga.essence_cost);
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
    void hasRating;
    addAugmentation(aug);
    setExpandedId(null);
  };

  return (
    <div>
      <h2 className="text-xl font-semibold mb-2 text-cyber-heading">
        // Augmentations
      </h2>
      <div className="text-sm text-cyber-text-dim font-mono mb-4 flex gap-4">
        <span>
          Essence:{" "}
          <span
            className={
              essenceRemaining <= 0 ? "text-cyber-red" : "text-cyber-blue"
            }
          >
            {(essenceRemaining / 100).toFixed(2)}
          </span>{" "}
          / 6.00
        </span>
        {draft.augmentations.length > 0 && (
          <span className="text-cyber-text-dim">
            ({draft.augmentations.length} installed)
          </span>
        )}
      </div>

      {/* Installed augmentations */}
      {draft.augmentations.length > 0 && (
        <div className="space-y-1 mb-4">
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

      {/* Add augmentation section */}
      <div className="bg-cyber-card border border-cyber-border rounded-lg p-3 space-y-3">
        <div className="flex gap-2 items-center">
          <input
            type="text"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Search augmentations…"
            className="flex-1 bg-cyber-surface border border-cyber-border rounded px-3 py-1.5 text-sm"
          />
          {(["All", "Cyberware", "Bioware"] as const).map((t) => (
            <button
              key={t}
              onClick={() => setTypeFilter(t)}
              className={`px-2 py-1 text-xs font-mono rounded border transition-colors ${
                typeFilter === t
                  ? "border-cyber-blue text-cyber-blue bg-cyber-blue/10"
                  : "border-cyber-border text-cyber-text-dim hover:border-cyber-border-bright"
              }`}
            >
              {t}
            </button>
          ))}
        </div>

        {/* Grade + rating controls */}
        <div className="flex gap-2 items-center text-xs font-mono">
          <span className="text-cyber-text-dim">Grade:</span>
          {GRADES.map((g) => (
            <button
              key={g}
              onClick={() => setGrade(g)}
              className={`px-2 py-0.5 rounded border transition-colors ${
                grade === g
                  ? "border-cyber-green text-cyber-green bg-cyber-green/10"
                  : "border-cyber-border text-cyber-text-dim hover:border-cyber-border-bright"
              }`}
            >
              {GRADE_LABEL[g]}
            </button>
          ))}
          <span className="text-cyber-text-dim ml-2">Rating:</span>
          <input
            type="number"
            min={1}
            max={6}
            value={rating}
            onChange={(e) =>
              setRating(Math.min(6, Math.max(1, Number(e.target.value))))
            }
            className="w-12 bg-cyber-surface border border-cyber-border rounded px-1 py-0.5 text-center"
          />
        </div>

        <div className="max-h-64 overflow-y-auto space-y-0.5">
          {filtered.length === 0 ? (
            <p className="text-cyber-text-dim text-xs font-mono py-4 text-center">
              No augmentations match filter
            </p>
          ) : (
            filtered.map((ga) => {
              const hasRating = /Rating/i.test(ga.essence_cost);
              const essenceCost = parseEssenceCost(ga.essence_cost, rating);
              const adjCost = Math.floor(
                (essenceCost * (GRADE_MULTIPLIER[grade] ?? 100)) / 100,
              );
              const alreadyInstalled = existingIds.has(
                `${ga.id}_${grade}`,
              );
              const isExpanded = expandedId === ga.id;

              return (
                <div
                  key={ga.id}
                  className="bg-cyber-surface border border-cyber-border rounded"
                >
                  <div className="flex items-center gap-2 px-3 py-1.5 text-sm">
                    <div className="flex-1 min-w-0">
                      <span className="text-cyber-text truncate block">
                        {ga.name}
                      </span>
                      <span className="text-cyber-text-dim font-mono text-xs">
                        {ga.augmentation_type}
                        {hasRating ? " · Rating-based" : ""} ·{" "}
                        {(adjCost / 100).toFixed(2)}E
                      </span>
                    </div>
                    <button
                      onClick={() =>
                        alreadyInstalled
                          ? undefined
                          : isExpanded
                            ? handleAdd(ga)
                            : setExpandedId(ga.id)
                      }
                      disabled={alreadyInstalled}
                      className={`px-2 py-0.5 text-xs font-mono border rounded transition-colors shrink-0 ${
                        alreadyInstalled
                          ? "border-cyber-border text-cyber-text-dim opacity-40 cursor-default"
                          : isExpanded
                            ? "border-cyber-green text-cyber-green bg-cyber-green/10"
                            : "border-cyber-border text-cyber-text-dim hover:border-cyber-border-bright"
                      }`}
                    >
                      {alreadyInstalled ? "✓" : isExpanded ? "Confirm" : "+"}
                    </button>
                    {isExpanded && (
                      <button
                        onClick={() => setExpandedId(null)}
                        className="text-cyber-text-dim hover:text-cyber-text text-xs"
                      >
                        ✕
                      </button>
                    )}
                  </div>
                </div>
              );
            })
          )}
        </div>
      </div>
    </div>
  );
}
