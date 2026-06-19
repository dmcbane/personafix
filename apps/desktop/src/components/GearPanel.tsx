import { useState } from "react";
import {
  useCharacterStore,
  type DraftWeapon,
  type DraftArmor,
} from "../store/characterStore";
import { useGameDataStore } from "../store/gameDataStore";

type Section = "Weapons" | "Armor";

const SECTION_BTN = (active: boolean) =>
  `px-3 py-1 text-xs font-mono rounded border transition-colors ${
    active
      ? "border-cyber-blue text-cyber-blue bg-cyber-blue/10"
      : "border-cyber-border text-cyber-text-dim hover:border-cyber-border-bright"
  }`;

const CAT_BTN = (active: boolean) =>
  `px-2.5 py-1 rounded text-xs font-mono transition-all ${
    active
      ? "bg-cyber-green-dim border border-cyber-green text-cyber-green shadow-glow"
      : "bg-cyber-card border border-cyber-border text-cyber-text-dim hover:border-cyber-border-bright"
  }`;

export default function GearPanel() {
  const draft = useCharacterStore((s) => s.draft);
  const addWeapon = useCharacterStore((s) => s.addWeapon);
  const removeWeapon = useCharacterStore((s) => s.removeWeapon);
  const addArmor = useCharacterStore((s) => s.addArmor);
  const removeArmor = useCharacterStore((s) => s.removeArmor);
  const gameWeapons = useGameDataStore((s) => s.weapons);
  const gameArmor = useGameDataStore((s) => s.armor);

  const [section, setSection] = useState<Section>("Weapons");
  const [search, setSearch] = useState("");
  const [catFilter, setCatFilter] = useState("All");
  const [selected, setSelected] = useState("");

  if (!draft) return null;

  const nuyenSpent = draft.nuyen_spent;
  const weaponCategories = Array.from(new Set(gameWeapons.map((w) => w.category))).sort();
  const existingWeaponIds = new Set(draft.weapons.map((w) => w.id));
  const existingArmorIds = new Set(draft.armor.map((a) => a.id));

  const availableWeapons = gameWeapons.filter((w) => {
    if (catFilter !== "All" && w.category !== catFilter) return false;
    if (search && !w.name.toLowerCase().includes(search.toLowerCase())) return false;
    return !existingWeaponIds.has(w.id);
  });

  const availableArmor = gameArmor.filter((a) => {
    if (search && !a.name.toLowerCase().includes(search.toLowerCase())) return false;
    return !existingArmorIds.has(a.id);
  });

  const handleAddWeapon = () => {
    const gw = availableWeapons.find((w) => w.name === selected);
    if (!gw) return;
    const weapon: DraftWeapon = {
      id: gw.id,
      name: gw.name,
      category: gw.category,
      damage: gw.damage,
      ap: gw.ap,
      mode: gw.mode,
      recoil_comp: parseInt(gw.recoil_comp) || 0,
      ammo: gw.ammo,
      availability: gw.availability,
      cost: parseFloat(gw.cost) || 0,
      source: gw.source,
      page: gw.page,
    };
    addWeapon(weapon);
    setSelected("");
  };

  const handleAddArmor = () => {
    const ga = availableArmor.find((a) => a.name === selected);
    if (!ga) return;
    const armor: DraftArmor = {
      id: ga.id,
      name: ga.name,
      armor_value: parseInt(ga.armor_value) || 0,
      availability: ga.availability,
      cost: parseFloat(ga.cost) || 0,
      source: ga.source,
      page: ga.page,
    };
    addArmor(armor);
    setSelected("");
  };

  const handleAdd = section === "Weapons" ? handleAddWeapon : handleAddArmor;

  return (
    <div>
      <h2 className="text-xl font-semibold mb-4 text-cyber-heading">
        // Gear
      </h2>

      {/* Stats */}
      <div className="flex gap-4 text-sm text-cyber-text-dim mb-4 font-mono">
        <span>
          Nuyen spent:{" "}
          <span className="text-cyber-text">¥{nuyenSpent.toLocaleString()}</span>
        </span>
        {draft.weapons.length > 0 && (
          <span>Weapons: <span className="text-cyber-text">{draft.weapons.length}</span></span>
        )}
        {draft.armor.length > 0 && (
          <span>Armor: <span className="text-cyber-text">{draft.armor.length}</span></span>
        )}
      </div>

      {/* Section tabs */}
      <div className="flex gap-2 mb-3">
        {(["Weapons", "Armor"] as Section[]).map((s) => (
          <button
            key={s}
            onClick={() => { setSection(s); setCatFilter("All"); setSearch(""); setSelected(""); }}
            className={SECTION_BTN(section === s)}
          >
            {s} ({s === "Weapons" ? gameWeapons.length : gameArmor.length})
          </button>
        ))}
      </div>

      {/* Category filter (weapons only) */}
      {section === "Weapons" && (
        <div className="flex gap-1.5 mb-3 flex-wrap">
          <button onClick={() => { setCatFilter("All"); setSelected(""); }} className={CAT_BTN(catFilter === "All")}>
            All
          </button>
          {weaponCategories.map((c) => (
            <button key={c} onClick={() => { setCatFilter(c); setSelected(""); }}
              className={CAT_BTN(catFilter === c)}>
              {c}
            </button>
          ))}
        </div>
      )}

      {/* Search + select + Add */}
      <div className="flex gap-2 mb-6">
        <input
          type="text"
          value={search}
          onChange={(e) => { setSearch(e.target.value); setSelected(""); }}
          placeholder={`Search ${section.toLowerCase()}…`}
          className="bg-cyber-card border border-cyber-border rounded px-3 py-1.5 text-sm w-44"
        />
        <select
          value={selected}
          onChange={(e) => setSelected(e.target.value)}
          className="bg-cyber-card border border-cyber-border rounded px-3 py-1.5 text-sm flex-1 text-cyber-text"
        >
          <option value="">Select {section === "Weapons" ? "a weapon" : "armor"}…</option>
          {section === "Weapons"
            ? availableWeapons.map((w) => (
                <option key={w.id} value={w.name}>
                  {w.name} ({w.category}, {w.damage}, {w.mode})
                </option>
              ))
            : availableArmor.map((a) => (
                <option key={a.id} value={a.name}>
                  {a.name} (Armor {a.armor_value || "—"})
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

      {/* Equipped list */}
      {draft.weapons.length === 0 && draft.armor.length === 0 ? (
        <p className="text-cyber-text-dim text-sm font-mono">No gear equipped.</p>
      ) : (
        <div className="space-y-1">
          {draft.weapons.map((w) => (
            <div
              key={w.id}
              className="flex items-center gap-2 bg-cyber-card border border-cyber-border rounded px-3 py-2 text-sm"
            >
              <div className="flex-1 min-w-0">
                <span className="text-cyber-text font-medium">{w.name}</span>
                <span className="text-cyber-text-dim font-mono text-xs ml-2">{w.category}</span>
              </div>
              <span className="font-mono text-xs text-cyber-text-dim shrink-0">
                {w.damage} / {w.mode}
              </span>
              <button
                onClick={() => removeWeapon(w.id)}
                className="text-cyber-red hover:text-cyber-red/80 transition-colors ml-1 shrink-0"
              >
                X
              </button>
            </div>
          ))}
          {draft.armor.map((a) => (
            <div
              key={a.id}
              className="flex items-center gap-2 bg-cyber-card border border-cyber-border rounded px-3 py-2 text-sm"
            >
              <div className="flex-1 min-w-0">
                <span className="text-cyber-text font-medium">{a.name}</span>
                <span className="text-cyber-text-dim font-mono text-xs ml-2">Armor</span>
              </div>
              <span className="font-mono text-xs text-cyber-blue shrink-0">
                {a.armor_value}
              </span>
              <button
                onClick={() => removeArmor(a.id)}
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
