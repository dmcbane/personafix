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

const CAT_BTN = (active: boolean) =>
  `px-2.5 py-1 rounded text-xs font-mono transition-all ${
    active
      ? "bg-cyber-green-dim border border-cyber-green text-cyber-green shadow-glow"
      : "bg-cyber-card border border-cyber-border text-cyber-text-dim hover:border-cyber-border-bright"
  }`;

const TYPE_BTN = (active: boolean) =>
  `px-2.5 py-1 rounded text-xs font-mono transition-all ${
    active
      ? "bg-cyber-blue/20 border border-cyber-blue text-cyber-blue"
      : "bg-cyber-card border border-cyber-border text-cyber-text-dim hover:border-cyber-border-bright"
  }`;

export default function MagicPanel() {
  const draft = useCharacterStore((s) => s.draft);
  const addSpell = useCharacterStore((s) => s.addSpell);
  const removeSpell = useCharacterStore((s) => s.removeSpell);
  const gameSpells = useGameDataStore((s) => s.spells);

  const [catFilter, setCatFilter] = useState<KnownCategory | "All">("All");
  const [typeFilter, setTypeFilter] = useState<"All" | "Physical" | "Mana">("All");
  const [search, setSearch] = useState("");
  const [selected, setSelected] = useState("");

  if (!draft) return null;

  const isMagic = draft.attributes.magic !== null && draft.attributes.magic > 0;
  const magicRating = draft.attributes.magic ?? 0;
  const existingIds = new Set(draft.spells.map((s) => s.id));

  const available = gameSpells
    .filter((s) => (KNOWN_CATEGORIES as readonly string[]).includes(s.category))
    .filter((s) => {
      if (catFilter !== "All" && s.category !== catFilter) return false;
      if (typeFilter !== "All" && s.spell_type !== typeFilter) return false;
      if (search && !s.name.toLowerCase().includes(search.toLowerCase())) return false;
      return !existingIds.has(s.id);
    });

  const handleAdd = () => {
    const gs = available.find((s) => s.name === selected);
    if (!gs) return;
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
    setSelected("");
  };

  return (
    <div>
      <h2 className="text-xl font-semibold mb-4 text-cyber-heading">
        // Magic
      </h2>

      {/* Warning if not awakened */}
      {!isMagic && (
        <div className="text-cyber-yellow text-sm font-mono mb-4 bg-cyber-yellow-dim/20 border border-cyber-yellow/30 rounded px-3 py-2">
          Magic attribute is 0 — set Magic &gt; 0 in Attributes to be Awakened
        </div>
      )}

      {/* Stats */}
      <div className="flex gap-4 text-sm text-cyber-text-dim mb-4 font-mono">
        <span>
          Spells known:{" "}
          <span className={isMagic && draft.spells.length > magicRating ? "text-cyber-red" : "text-cyber-blue"}>
            {draft.spells.length}
          </span>
          {isMagic && <span>/{magicRating} (Magic rating)</span>}
        </span>
      </div>

      {/* Category filter */}
      <div className="flex gap-1.5 mb-3 flex-wrap">
        {(["All", ...KNOWN_CATEGORIES] as const).map((c) => (
          <button key={c} onClick={() => { setCatFilter(c as KnownCategory | "All"); setSelected(""); }}
            className={CAT_BTN(catFilter === c)}>
            {c}
          </button>
        ))}
      </div>

      {/* Type filter */}
      <div className="flex gap-1.5 mb-3">
        {(["All", "Physical", "Mana"] as const).map((t) => (
          <button key={t} onClick={() => { setTypeFilter(t); setSelected(""); }}
            className={TYPE_BTN(typeFilter === t)}>
            {t}
          </button>
        ))}
      </div>

      {/* Search + select + Add */}
      <div className="flex gap-2 mb-6">
        <input
          type="text"
          value={search}
          onChange={(e) => { setSearch(e.target.value); setSelected(""); }}
          placeholder="Search spells…"
          className="bg-cyber-card border border-cyber-border rounded px-3 py-1.5 text-sm w-40"
        />
        <select
          value={selected}
          onChange={(e) => setSelected(e.target.value)}
          className="bg-cyber-card border border-cyber-border rounded px-3 py-1.5 text-sm flex-1 text-cyber-text"
        >
          <option value="">Select a spell…</option>
          {available.map((s) => (
            <option key={s.id} value={s.name}>
              {s.name} ({s.category}, {s.spell_type}, {s.drain})
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

      {/* Known spells list */}
      {draft.spells.length === 0 ? (
        <p className="text-cyber-text-dim text-sm font-mono">No spells learned.</p>
      ) : (
        <div className="space-y-1">
          {draft.spells.map((s) => (
            <div
              key={s.id}
              className="flex items-center gap-2 bg-cyber-card border border-cyber-border rounded px-3 py-2 text-sm"
            >
              <div className="flex-1 min-w-0">
                <span className="text-cyber-text font-medium">{s.name}</span>
                <span className="text-cyber-text-dim font-mono text-xs ml-2">
                  {s.category} · {s.spell_type}
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
    </div>
  );
}
