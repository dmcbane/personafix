import { useState } from "react";
import {
  useCharacterStore,
  type DraftWeapon,
  type DraftArmor,
} from "../store/characterStore";
import { useGameDataStore } from "../store/gameDataStore";

type Section = "Weapons" | "Armor";

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

  if (!draft) return null;

  const weaponCategories = Array.from(
    new Set(gameWeapons.map((w) => w.category)),
  ).sort();
  const armorCategories: string[] = [];

  const filteredWeapons = gameWeapons.filter((w) => {
    if (catFilter !== "All" && w.category !== catFilter) return false;
    return !search || w.name.toLowerCase().includes(search.toLowerCase());
  });

  const filteredArmor = gameArmor.filter((a) =>
    !search || a.name.toLowerCase().includes(search.toLowerCase()),
  );

  const existingWeaponIds = new Set(draft.weapons.map((w) => w.id));
  const existingArmorIds = new Set(draft.armor.map((a) => a.id));

  const handleAddWeapon = (gw: (typeof gameWeapons)[0]) => {
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
  };

  const handleAddArmor = (ga: (typeof gameArmor)[0]) => {
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
  };

  const categories =
    section === "Weapons" ? weaponCategories : armorCategories;

  return (
    <div>
      <h2 className="text-xl font-semibold mb-2 text-cyber-heading">
        // Gear
      </h2>

      {/* Equipped summary */}
      {(draft.weapons.length > 0 || draft.armor.length > 0) && (
        <div className="space-y-1 mb-4">
          {draft.weapons.map((w) => (
            <div
              key={w.id}
              className="flex items-center gap-2 bg-cyber-card border border-cyber-border rounded px-3 py-2 text-sm"
            >
              <div className="flex-1 min-w-0">
                <span className="text-cyber-text font-medium">{w.name}</span>
                <span className="text-cyber-text-dim font-mono text-xs ml-2">
                  {w.category}
                </span>
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
                <span className="text-cyber-text-dim font-mono text-xs ml-2">
                  Armor
                </span>
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

      {/* Section tabs + search */}
      <div className="bg-cyber-card border border-cyber-border rounded-lg p-3 space-y-3">
        <div className="flex gap-2 items-center">
          {(["Weapons", "Armor"] as Section[]).map((s) => (
            <button
              key={s}
              onClick={() => {
                setSection(s);
                setCatFilter("All");
                setSearch("");
              }}
              className={`px-3 py-1 text-xs font-mono rounded border transition-colors ${
                section === s
                  ? "border-cyber-blue text-cyber-blue bg-cyber-blue/10"
                  : "border-cyber-border text-cyber-text-dim hover:border-cyber-border-bright"
              }`}
            >
              {s}{" "}
              {s === "Weapons"
                ? `(${gameWeapons.length})`
                : `(${gameArmor.length})`}
            </button>
          ))}
          <input
            type="text"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder={`Search ${section.toLowerCase()}…`}
            className="flex-1 bg-cyber-surface border border-cyber-border rounded px-3 py-1.5 text-sm"
          />
        </div>

        {section === "Weapons" && categories.length > 0 && (
          <div className="flex gap-1 flex-wrap">
            {["All", ...categories].map((c) => (
              <button
                key={c}
                onClick={() => setCatFilter(c)}
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
        )}

        <div className="max-h-64 overflow-y-auto space-y-0.5">
          {section === "Weapons" &&
            (filteredWeapons.length === 0 ? (
              <p className="text-cyber-text-dim text-xs font-mono py-4 text-center">
                No weapons match filter
              </p>
            ) : (
              filteredWeapons.map((gw) => {
                const alreadyOwned = existingWeaponIds.has(gw.id);
                return (
                  <div
                    key={gw.id}
                    className="flex items-center gap-2 bg-cyber-surface border border-cyber-border rounded px-3 py-1.5 text-sm"
                  >
                    <div className="flex-1 min-w-0">
                      <span className="text-cyber-text truncate block">
                        {gw.name}
                      </span>
                      <span className="text-cyber-text-dim font-mono text-xs">
                        {gw.category} · {gw.damage} · {gw.mode}
                      </span>
                    </div>
                    <button
                      onClick={() => !alreadyOwned && handleAddWeapon(gw)}
                      disabled={alreadyOwned}
                      className={`px-2 py-0.5 text-xs font-mono border rounded transition-colors shrink-0 ${
                        alreadyOwned
                          ? "border-cyber-border text-cyber-text-dim opacity-40 cursor-default"
                          : "border-cyber-border text-cyber-text-dim hover:border-cyber-border-bright"
                      }`}
                    >
                      {alreadyOwned ? "✓" : "+"}
                    </button>
                  </div>
                );
              })
            ))}

          {section === "Armor" &&
            (filteredArmor.length === 0 ? (
              <p className="text-cyber-text-dim text-xs font-mono py-4 text-center">
                No armor match filter
              </p>
            ) : (
              filteredArmor.map((ga) => {
                const alreadyOwned = existingArmorIds.has(ga.id);
                return (
                  <div
                    key={ga.id}
                    className="flex items-center gap-2 bg-cyber-surface border border-cyber-border rounded px-3 py-1.5 text-sm"
                  >
                    <div className="flex-1 min-w-0">
                      <span className="text-cyber-text truncate block">
                        {ga.name}
                      </span>
                      <span className="text-cyber-text-dim font-mono text-xs">
                        Armor {ga.armor_value || "—"} · {ga.availability}
                      </span>
                    </div>
                    <button
                      onClick={() => !alreadyOwned && handleAddArmor(ga)}
                      disabled={alreadyOwned}
                      className={`px-2 py-0.5 text-xs font-mono border rounded transition-colors shrink-0 ${
                        alreadyOwned
                          ? "border-cyber-border text-cyber-text-dim opacity-40 cursor-default"
                          : "border-cyber-border text-cyber-text-dim hover:border-cyber-border-bright"
                      }`}
                    >
                      {alreadyOwned ? "✓" : "+"}
                    </button>
                  </div>
                );
              })
            ))}
        </div>
      </div>
    </div>
  );
}
