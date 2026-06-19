import { useState } from "react";
import {
  useCharacterStore,
  type DraftSpell,
} from "../store/characterStore";
import { useGameDataStore } from "../store/gameDataStore";

// Must match the Rust SpellCategory enum variants (case-sensitive)
const KNOWN_CATEGORIES = [
  "Combat",
  "Detection",
  "Health",
  "Illusion",
  "Manipulation",
] as const;

type KnownCategory = (typeof KNOWN_CATEGORIES)[number];

export default function MagicPanel() {
  const draft = useCharacterStore((s) => s.draft);
  const addSpell = useCharacterStore((s) => s.addSpell);
  const removeSpell = useCharacterStore((s) => s.removeSpell);
  const gameSpells = useGameDataStore((s) => s.spells);

  const [search, setSearch] = useState("");
  const [catFilter, setCatFilter] = useState<KnownCategory | "All">("All");
  const [typeFilter, setTypeFilter] = useState<"All" | "Physical" | "Mana">(
    "All",
  );

  if (!draft) return null;

  const isMagic =
    draft.attributes.magic !== null && draft.attributes.magic > 0;

  // Only include spells with known categories (so Rust deserialization succeeds on save)
  const availableSpells = gameSpells.filter((s) =>
    (KNOWN_CATEGORIES as readonly string[]).includes(s.category),
  );

  const existingIds = new Set(draft.spells.map((s) => s.id));

  const filtered = availableSpells.filter((s) => {
    if (catFilter !== "All" && s.category !== catFilter) return false;
    if (typeFilter !== "All" && s.spell_type !== typeFilter) return false;
    if (search && !s.name.toLowerCase().includes(search.toLowerCase()))
      return false;
    return true;
  });

  const handleAdd = (gs: (typeof availableSpells)[0]) => {
    const spell: DraftSpell = {
      id: gs.id,
      name: gs.name,
      category: gs.category,
      spell_type: gs.spell_type,
      range: gs.range,
      damage: gs.damage,
      duration: gs.duration,
      drain: gs.drain,
      source: gs.source,
      page: gs.page,
    };
    addSpell(spell);
  };

  return (
    <div>
      <h2 className="text-xl font-semibold mb-2 text-cyber-heading">
        // Magic
      </h2>

      {!isMagic && (
        <div className="text-cyber-yellow text-sm font-mono mb-4 bg-cyber-yellow-dim/20 border border-cyber-yellow/30 rounded px-3 py-2">
          Magic attribute is 0 — set Magic &gt; 0 in Attributes to be Awakened
        </div>
      )}

      <div className="text-sm text-cyber-text-dim font-mono mb-4">
        Spells:{" "}
        <span className="text-cyber-blue">{draft.spells.length}</span>
        {isMagic && (
          <span className="text-cyber-text-dim">
            {" "}(max: Magic rating in SR4)
          </span>
        )}
      </div>

      {/* Known spells */}
      {draft.spells.length > 0 && (
        <div className="space-y-1 mb-4">
          {draft.spells.map((s) => (
            <div
              key={s.id}
              className="flex items-center gap-2 bg-cyber-card border border-cyber-border rounded px-3 py-2 text-sm"
            >
              <div className="flex-1 min-w-0">
                <span className="text-cyber-text font-medium">{s.name}</span>
                <span className="text-cyber-text-dim font-mono text-xs ml-2">
                  {s.spell_type}
                </span>
              </div>
              <span className="font-mono text-xs text-cyber-text-dim shrink-0">
                {s.drain}
              </span>
              <button
                onClick={() => removeSpell(s.id)}
                className="text-cyber-red hover:text-cyber-red/80 transition-colors ml-1 shrink-0"
              >
                X
              </button>
            </div>
          ))}
        </div>
      )}

      {/* Add spell section */}
      <div className="bg-cyber-card border border-cyber-border rounded-lg p-3 space-y-3">
        <div className="flex gap-2 flex-wrap">
          <input
            type="text"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Search spells…"
            className="flex-1 min-w-32 bg-cyber-surface border border-cyber-border rounded px-3 py-1.5 text-sm"
          />
          {(["All", "Physical", "Mana"] as const).map((t) => (
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

        <div className="flex gap-1 flex-wrap">
          {(["All", ...KNOWN_CATEGORIES] as const).map((c) => (
            <button
              key={c}
              onClick={() => setCatFilter(c as KnownCategory | "All")}
              className={`px-2 py-0.5 text-xs font-mono rounded border transition-colors ${
                catFilter === c
                  ? "border-cyber-green text-cyber-green bg-cyber-green/10"
                  : "border-cyber-border text-cyber-text-dim hover:border-cyber-border-bright"
              }`}
            >
              {c}
            </button>
          ))}
        </div>

        <div className="max-h-64 overflow-y-auto space-y-0.5">
          {filtered.length === 0 ? (
            <p className="text-cyber-text-dim text-xs font-mono py-4 text-center">
              No spells match filter
            </p>
          ) : (
            filtered.map((gs) => {
              const alreadyKnown = existingIds.has(gs.id);
              return (
                <div
                  key={gs.id}
                  className="flex items-center gap-2 bg-cyber-surface border border-cyber-border rounded px-3 py-1.5 text-sm"
                >
                  <div className="flex-1 min-w-0">
                    <span className="text-cyber-text truncate block">
                      {gs.name}
                    </span>
                    <span className="text-cyber-text-dim font-mono text-xs">
                      {gs.category} · {gs.spell_type} · Drain: {gs.drain}
                    </span>
                  </div>
                  <button
                    onClick={() => !alreadyKnown && handleAdd(gs)}
                    disabled={alreadyKnown}
                    className={`px-2 py-0.5 text-xs font-mono border rounded transition-colors shrink-0 ${
                      alreadyKnown
                        ? "border-cyber-border text-cyber-text-dim opacity-40 cursor-default"
                        : "border-cyber-border text-cyber-text-dim hover:border-cyber-border-bright"
                    }`}
                  >
                    {alreadyKnown ? "✓" : "+"}
                  </button>
                </div>
              );
            })
          )}
        </div>
      </div>
    </div>
  );
}
