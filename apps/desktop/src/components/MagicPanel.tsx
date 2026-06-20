import { useState } from "react";
import {
  useCharacterStore,
  type DraftSpell,
  type DraftAdeptPower,
  type DraftComplexForm,
  type MagicTradition,
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

function canCastSpells(tradition: MagicTradition | null): boolean {
  return tradition === "Magician" || tradition === "MysticAdept";
}

function canUseAdeptPowers(tradition: MagicTradition | null): boolean {
  return tradition === "Adept" || tradition === "MysticAdept";
}

function canUseComplexForms(tradition: MagicTradition | null): boolean {
  return tradition === "Technomancer";
}

/** Parse cost string like "0.25" → 0.25 */
function parseCost(cost: string): number {
  return parseFloat(cost) || 0;
}

export default function MagicPanel() {
  const draft = useCharacterStore((s) => s.draft);
  const addSpell = useCharacterStore((s) => s.addSpell);
  const removeSpell = useCharacterStore((s) => s.removeSpell);
  const addAdeptPower = useCharacterStore((s) => s.addAdeptPower);
  const removeAdeptPower = useCharacterStore((s) => s.removeAdeptPower);
  const addComplexForm = useCharacterStore((s) => s.addComplexForm);
  const removeComplexForm = useCharacterStore((s) => s.removeComplexForm);
  const gameSpells = useGameDataStore((s) => s.spells);
  const gameAdeptPowers = useGameDataStore((s) => s.adeptPowers);
  const gameComplexForms = useGameDataStore((s) => s.complexForms);

  const [catFilter, setCatFilter] = useState<KnownCategory | "All">("All");
  const [typeFilter, setTypeFilter] = useState<"All" | "Physical" | "Mana">("All");
  const [search, setSearch] = useState("");
  const [selected, setSelected] = useState("");
  const [powerSearch, setPowerSearch] = useState("");
  const [selectedPower, setSelectedPower] = useState("");
  const [formSearch, setFormSearch] = useState("");
  const [selectedForm, setSelectedForm] = useState("");

  if (!draft) return null;

  const tradition = draft.magic_tradition;
  const isMagic = draft.attributes.magic !== null && draft.attributes.magic > 0;
  const magicRating = draft.attributes.magic ?? 0;
  const existingIds = new Set(draft.spells.map((s) => s.id));

  const showSpells = canCastSpells(tradition) || draft.edition === "SR4";
  const showAdeptPowers = canUseAdeptPowers(tradition);
  const showComplexForms = canUseComplexForms(tradition);

  // Power point pool = magic rating; each power costs its decimal value
  const totalPowerPoints = magicRating;
  const spentPowerPoints = draft.adept_powers.reduce(
    (acc, p) => acc + parseCost(p.cost),
    0
  );
  const remainingPP = totalPowerPoints - spentPowerPoints;

  const availableSpells = gameSpells
    .filter((s) => (KNOWN_CATEGORIES as readonly string[]).includes(s.category))
    .filter((s) => {
      if (catFilter !== "All" && s.category !== catFilter) return false;
      if (typeFilter !== "All" && s.spell_type !== typeFilter) return false;
      if (search && !s.name.toLowerCase().includes(search.toLowerCase())) return false;
      return !existingIds.has(s.id);
    });

  const equippedPowerIds = new Set(draft.adept_powers.map((p) => p.id));
  const availablePowers = gameAdeptPowers.filter((p) => {
    if (equippedPowerIds.has(p.id)) return false;
    if (powerSearch && !p.name.toLowerCase().includes(powerSearch.toLowerCase())) return false;
    return true;
  });

  const resonanceRating = draft.attributes.resonance ?? 0;
  const equippedFormIds = new Set(draft.complex_forms.map((f) => f.id));
  const availableForms = gameComplexForms.filter((f) => {
    if (equippedFormIds.has(f.id)) return false;
    if (formSearch && !f.name.toLowerCase().includes(formSearch.toLowerCase())) return false;
    return true;
  });

  const handleAddForm = () => {
    const gf = availableForms.find((f) => f.name === selectedForm);
    if (!gf) return;
    const form: DraftComplexForm = {
      id: gf.id,
      name: gf.name,
      target: gf.target,
      duration: gf.duration,
      fading: gf.fading,
      source: gf.source,
      page: gf.page,
    };
    addComplexForm(form);
    setSelectedForm("");
  };

  const handleAddSpell = () => {
    const gs = availableSpells.find((s) => s.name === selected);
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

  const handleAddPower = () => {
    const gp = availablePowers.find((p) => p.name === selectedPower);
    if (!gp) return;
    const power: DraftAdeptPower = {
      id: gp.id,
      name: gp.name,
      cost: gp.cost,
      levels: gp.levels,
      source: gp.source,
      page: gp.page,
    };
    addAdeptPower(power);
    setSelectedPower("");
  };

  // SR5 with no tradition: show guidance
  if (draft.edition === "SR5" && !tradition) {
    return (
      <div>
        <h2 className="text-xl font-semibold mb-4 text-cyber-heading">// Magic</h2>
        <div className="text-cyber-yellow text-sm font-mono bg-cyber-yellow-dim/20 border border-cyber-yellow/30 rounded px-3 py-2">
          Choose an awakened tradition in the Priority tab to unlock Magic options.
        </div>
      </div>
    );
  }

  return (
    <div>
      <h2 className="text-xl font-semibold mb-4 text-cyber-heading">
        // Magic
        {tradition && (
          <span className="ml-2 text-sm text-cyber-blue font-mono">
            [{tradition}]
          </span>
        )}
      </h2>

      {/* Warning if not awakened (SR4 only) */}
      {draft.edition === "SR4" && !isMagic && (
        <div className="text-cyber-yellow text-sm font-mono mb-4 bg-cyber-yellow-dim/20 border border-cyber-yellow/30 rounded px-3 py-2">
          Magic attribute is 0 — set Magic &gt; 0 in Attributes to be Awakened
        </div>
      )}

      {/* Adept powers section */}
      {showAdeptPowers && (
        <div className="mb-6">
          <div className="flex items-center gap-4 mb-3">
            <p className="text-cyber-blue font-mono text-sm font-semibold">Adept Powers</p>
            <span className={`text-xs font-mono ${remainingPP < 0 ? "text-cyber-red" : "text-cyber-text-dim"}`}>
              PP: {spentPowerPoints.toFixed(2)}/{totalPowerPoints}
              {remainingPP < 0 && (
                <span className="text-cyber-red ml-1">({Math.abs(remainingPP).toFixed(2)} over)</span>
              )}
            </span>
          </div>

          {/* Power search + select + Add */}
          <div className="flex gap-2 mb-4">
            <input
              type="text"
              value={powerSearch}
              onChange={(e) => { setPowerSearch(e.target.value); setSelectedPower(""); }}
              placeholder="Search powers…"
              className="bg-cyber-card border border-cyber-border rounded px-3 py-1.5 text-sm w-40"
            />
            <select
              value={selectedPower}
              onChange={(e) => setSelectedPower(e.target.value)}
              className="bg-cyber-card border border-cyber-border rounded px-3 py-1.5 text-sm flex-1 text-cyber-text"
            >
              <option value="">Select a power…</option>
              {availablePowers.map((p) => (
                <option key={p.id} value={p.name}>
                  {p.name} ({p.cost} PP{p.levels ? ", levels" : ""})
                </option>
              ))}
            </select>
            <button
              onClick={handleAddPower}
              disabled={!selectedPower}
              className="px-4 py-1.5 bg-cyber-green-dim hover:bg-cyber-green/20 border border-cyber-green-dim hover:border-cyber-green rounded text-sm disabled:opacity-50 text-cyber-green font-mono transition-all"
            >
              Add
            </button>
          </div>

          {/* Equipped powers list */}
          {draft.adept_powers.length === 0 ? (
            <p className="text-cyber-text-dim text-sm font-mono">No adept powers chosen.</p>
          ) : (
            <div className="space-y-1">
              {draft.adept_powers.map((p) => (
                <div
                  key={p.id}
                  className="flex items-center gap-2 bg-cyber-card border border-cyber-border rounded px-3 py-2 text-sm"
                >
                  <div className="flex-1 min-w-0">
                    <span className="text-cyber-text font-medium">{p.name}</span>
                    {p.levels && (
                      <span className="text-cyber-text-dim font-mono text-xs ml-2">leveled</span>
                    )}
                  </div>
                  <span className="font-mono text-xs text-cyber-blue shrink-0">
                    {p.cost} PP
                  </span>
                  <button
                    onClick={() => removeAdeptPower(p.id)}
                    className="text-cyber-red hover:text-cyber-red/80 transition-colors ml-1 shrink-0"
                  >
                    X
                  </button>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {/* Complex forms section */}
      {showComplexForms && (
        <div className="mb-6">
          <div className="flex items-center gap-4 mb-3">
            <p className="text-cyber-blue font-mono text-sm font-semibold">Complex Forms</p>
            <span className={`text-xs font-mono ${draft.complex_forms.length > resonanceRating && resonanceRating > 0 ? "text-cyber-red" : "text-cyber-text-dim"}`}>
              {draft.complex_forms.length}
              {resonanceRating > 0 && `/${resonanceRating} (Resonance)`}
              {draft.complex_forms.length > resonanceRating && resonanceRating > 0 && (
                <span className="text-cyber-red ml-1">over cap</span>
              )}
            </span>
          </div>

          {/* Search + select + Add */}
          <div className="flex gap-2 mb-4">
            <input
              type="text"
              value={formSearch}
              onChange={(e) => { setFormSearch(e.target.value); setSelectedForm(""); }}
              placeholder="Search forms…"
              className="bg-cyber-card border border-cyber-border rounded px-3 py-1.5 text-sm w-40"
            />
            <select
              value={selectedForm}
              onChange={(e) => setSelectedForm(e.target.value)}
              className="bg-cyber-card border border-cyber-border rounded px-3 py-1.5 text-sm flex-1 text-cyber-text"
            >
              <option value="">Select a complex form…</option>
              {availableForms.map((f) => (
                <option key={f.id} value={f.name}>
                  {f.name} ({f.target}, {f.duration}, {f.fading})
                </option>
              ))}
            </select>
            <button
              onClick={handleAddForm}
              disabled={!selectedForm}
              className="px-4 py-1.5 bg-cyber-green-dim hover:bg-cyber-green/20 border border-cyber-green-dim hover:border-cyber-green rounded text-sm disabled:opacity-50 text-cyber-green font-mono transition-all"
            >
              Add
            </button>
          </div>

          {/* Equipped complex forms list */}
          {draft.complex_forms.length === 0 ? (
            <p className="text-cyber-text-dim text-sm font-mono">No complex forms learned.</p>
          ) : (
            <div className="space-y-1">
              {draft.complex_forms.map((f) => (
                <div
                  key={f.id}
                  className="flex items-center gap-2 bg-cyber-card border border-cyber-border rounded px-3 py-2 text-sm"
                >
                  <div className="flex-1 min-w-0">
                    <span className="text-cyber-text font-medium">{f.name}</span>
                    <span className="text-cyber-text-dim font-mono text-xs ml-2">
                      {f.target} · {f.duration}
                    </span>
                  </div>
                  <span className="font-mono text-xs text-cyber-blue shrink-0">
                    {f.fading}
                  </span>
                  <button
                    onClick={() => removeComplexForm(f.id)}
                    className="text-cyber-red hover:text-cyber-red/80 transition-colors ml-1 shrink-0"
                  >
                    X
                  </button>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {/* Spells section */}
      {showSpells && (
        <>
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
              {availableSpells.map((s) => (
                <option key={s.id} value={s.name}>
                  {s.name} ({s.category}, {s.spell_type}, {s.drain})
                </option>
              ))}
            </select>
            <button
              onClick={handleAddSpell}
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
        </>
      )}
    </div>
  );
}
