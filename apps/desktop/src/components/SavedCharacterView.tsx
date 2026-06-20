import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useCharacterStore, type LedgerEvent } from "../store/characterStore";

// ---- Dice roller logic ----
interface RollResult {
  dice: number[];
  hits: number;
  ones: number;
  isGlitch: boolean;
  isCriticalGlitch: boolean;
  label: string;
}

function rollPool(count: number, label: string): RollResult {
  const dice = Array.from({ length: Math.max(1, count) }, () => Math.floor(Math.random() * 6) + 1);
  const hits = dice.filter((d) => d >= 5).length;
  const ones = dice.filter((d) => d === 1).length;
  const isGlitch = ones > dice.length / 2;
  return { dice, hits, ones, isGlitch, isCriticalGlitch: isGlitch && hits === 0, label };
}

function DieIcon({ value }: { value: number }) {
  const isHit = value >= 5;
  const isOne = value === 1;
  const color = isHit ? "text-cyber-green bg-cyber-green/10 border-cyber-green/50" : isOne ? "text-cyber-red bg-cyber-red/10 border-cyber-red/50" : "text-cyber-text-dim bg-cyber-surface border-cyber-border";
  return (
    <span className={`inline-flex items-center justify-center w-7 h-7 rounded text-xs font-bold font-mono border ${color}`}>
      {value}
    </span>
  );
}

export default function SavedCharacterView() {
  const saved = useCharacterStore((s) => s.savedCharacter);
  const reset = useCharacterStore((s) => s.reset);
  const applyEvent = useCharacterStore((s) => s.applyEvent);
  const updateNotes = useCharacterStore((s) => s.updateNotes);

  const [karmaAmount, setKarmaAmount] = useState(5);
  const [karmaReason, setKarmaReason] = useState("Run reward");
  const [nuyenReceiveAmount, setNuyenReceiveAmount] = useState(5000);
  const [nuyenReceiveReason, setNuyenReceiveReason] = useState("Run reward");
  const [nuyenSpendAmount, setNuyenSpendAmount] = useState(1000);
  const [nuyenSpendReason, setNuyenSpendReason] = useState("Gear");
  const [applying, setApplying] = useState(false);
  const [eventError, setEventError] = useState<string | null>(null);
  const [showImprove, setShowImprove] = useState(false);
  const [showLedger, setShowLedger] = useState(false);
  const [showSheet, setShowSheet] = useState(false);
  const [showDice, setShowDice] = useState(false);
  const [dicePool, setDicePool] = useState(6);
  const [rollResults, setRollResults] = useState<RollResult[]>([]);
  const diceScrollRef = useRef<HTMLDivElement>(null);
  const [ledgerEvents, setLedgerEvents] = useState<LedgerEvent[]>([]);
  const [ledgerLoading, setLedgerLoading] = useState(false);
  const [showNotes, setShowNotes] = useState(false);
  const [notesText, setNotesText] = useState<string | null>(null);
  const [notesSaving, setNotesSaving] = useState(false);

  if (!saved) return null;

  const characterId = saved.base.id;
  const base = saved.base;

  const loadLedger = useCallback(async () => {
    setLedgerLoading(true);
    try {
      const events = await invoke<LedgerEvent[]>("get_ledger", { characterId });
      setLedgerEvents(events);
    } catch {
      // non-fatal
    } finally {
      setLedgerLoading(false);
    }
  }, [characterId]);

  useEffect(() => {
    if (showLedger) loadLedger();
  }, [showLedger, loadLedger]);

  const applyWithReload = async (event: LedgerEvent) => {
    setApplying(true);
    setEventError(null);
    try {
      await applyEvent(characterId, event);
      if (showLedger) await loadLedger();
    } catch (err) {
      setEventError(String(err));
    } finally {
      setApplying(false);
    }
  };

  const handleKarmaReceived = () =>
    applyWithReload({ KarmaReceived: { amount: karmaAmount, reason: karmaReason, run_id: null } });

  const handleNuyenReceived = () =>
    applyWithReload({ NuyenReceived: { amount: nuyenReceiveAmount, reason: nuyenReceiveReason, run_id: null } });

  const handleNuyenSpent = () => {
    if (nuyenSpendAmount <= 0 || nuyenSpendAmount > saved.nuyen) {
      setEventError(`Cannot spend ¥${nuyenSpendAmount.toLocaleString()} (available: ¥${saved.nuyen.toLocaleString()})`);
      return;
    }
    applyWithReload({ NuyenSpent: { amount: nuyenSpendAmount, description: nuyenSpendReason } });
  };

  const handleContactImprove = async (
    contactId: string,
    contactName: string,
    currentConnection: number,
    currentLoyalty: number,
    field: "connection" | "loyalty",
  ) => {
    const newConnection = field === "connection" ? currentConnection + 1 : currentConnection;
    const newLoyalty = field === "loyalty" ? currentLoyalty + 1 : currentLoyalty;
    const karmaCost = field === "connection" ? newConnection : newLoyalty;
    const available = saved.total_karma_earned - saved.total_karma_spent;
    if (available < karmaCost) {
      setEventError(`Not enough karma (need ${karmaCost}, have ${available})`);
      return;
    }
    setApplying(true);
    setEventError(null);
    try {
      await applyEvent(characterId, {
        ContactChanged: {
          contact_id: contactId,
          new_connection: newConnection,
          new_loyalty: newLoyalty,
        },
      });
      await applyEvent(characterId, {
        KarmaSpent: {
          amount: karmaCost,
          description: `${contactName} ${field} ${field === "connection" ? currentConnection : currentLoyalty} → ${field === "connection" ? newConnection : newLoyalty}`,
        },
      });
      if (showLedger) await loadLedger();
    } catch (err) {
      setEventError(String(err));
    } finally {
      setApplying(false);
    }
  };

  const handleSkillImprove = async (skillName: string, from: number) => {
    const to = from + 1;
    const karmaCost = to * 2;
    const available = saved.total_karma_earned - saved.total_karma_spent;
    if (available < karmaCost) {
      setEventError(`Not enough karma (need ${karmaCost}, have ${available})`);
      return;
    }
    setApplying(true);
    setEventError(null);
    try {
      await applyEvent(characterId, { SkillImproved: { skill_name: skillName, from, to, karma_cost: karmaCost } });
      await applyEvent(characterId, { KarmaSpent: { amount: karmaCost, description: `${skillName} ${from} → ${to}` } });
      if (showLedger) await loadLedger();
    } catch (err) {
      setEventError(String(err));
    } finally {
      setApplying(false);
    }
  };

  const handleAttrImprove = async (attr: string, from: number) => {
    const to = from + 1;
    const karmaCost = to * 5;
    const available = saved.total_karma_earned - saved.total_karma_spent;
    if (available < karmaCost) {
      setEventError(`Not enough karma (need ${karmaCost}, have ${available})`);
      return;
    }
    setApplying(true);
    setEventError(null);
    try {
      await applyEvent(characterId, { AttributeImproved: { attribute: attr, from, to, karma_cost: karmaCost } });
      await applyEvent(characterId, { KarmaSpent: { amount: karmaCost, description: `${attr} ${from} → ${to}` } });
      if (showLedger) await loadLedger();
    } catch (err) {
      setEventError(String(err));
    } finally {
      setApplying(false);
    }
  };

  const handleRoll = (pool: number, label: string) => {
    const result = rollPool(pool, label);
    setRollResults((prev) => [result, ...prev.slice(0, 9)]);
    setTimeout(() => diceScrollRef.current?.scrollIntoView({ behavior: "smooth" }), 50);
  };

  const attrs = saved.computed_attributes;
  const tradition = base.magic_tradition;

  const skillPools = base.skills.map((skill) => {
    const attrValue = (attrs as unknown as Record<string, unknown>)[skill.linked_attribute.toLowerCase()] as number | undefined;
    const pool = skill.rating + (attrValue ?? 0);
    return { skill, pool };
  }).filter((sp) => sp.pool > 0).sort((a, b) => a.skill.name.localeCompare(b.skill.name));
  const hasSpells = tradition === "Magician" || tradition === "MysticAdept" || base.edition === "SR4";
  const hasPowers = tradition === "Adept" || tradition === "MysticAdept";
  const hasForms = tradition === "Technomancer";

  return (
    <div className="min-h-screen p-8">
      <div className="max-w-2xl mx-auto">
        {/* Header */}
        <div className="flex items-center justify-between mb-6">
          <div>
            <h1 className="text-3xl font-bold text-cyber-heading">{base.name}</h1>
            <p className="text-cyber-text-dim font-mono">
              {base.edition} {base.metatype} // Character Sheet
              {tradition && <span className="text-cyber-blue ml-2">[{tradition}{base.tradition_name ? ` · ${base.tradition_name}` : ""}]</span>}
            </p>
          </div>
          <div className="flex gap-2">
            <button
              onClick={() => window.print()}
              title="Print character sheet"
              className="px-3 py-2 bg-cyber-card border border-cyber-border hover:border-cyber-blue text-cyber-blue rounded text-sm transition-colors font-mono"
            >
              ⎙ Print
            </button>
            <button
              onClick={reset}
              className="px-4 py-2 bg-cyber-card border border-cyber-border hover:border-cyber-border-bright rounded text-sm text-cyber-text transition-colors"
            >
              ← Characters
            </button>
          </div>
        </div>

        {/* Attributes */}
        <div className="bg-cyber-card border border-cyber-border rounded-lg p-4 mb-4">
          <h2 className="text-lg font-semibold mb-3 text-cyber-heading font-mono">// Attributes</h2>
          <div className="grid grid-cols-3 sm:grid-cols-5 gap-2">
            {(
              [
                ["BOD", attrs.body],
                ["AGI", attrs.agility],
                ["REA", attrs.reaction],
                ["STR", attrs.strength],
                ["WIL", attrs.willpower],
                ["LOG", attrs.logic],
                ["INT", attrs.intuition],
                ["CHA", attrs.charisma],
                ["EDG", attrs.edge],
              ] as [string, number][]
            ).map(([label, value]) => (
              <div key={label} className="bg-cyber-surface border border-cyber-border rounded px-3 py-2 text-center">
                <div className="text-cyber-blue text-xs font-mono">{label}</div>
                <div className="text-xl font-bold text-cyber-heading">{value}</div>
              </div>
            ))}
          </div>
        </div>

        {/* Derived Stats */}
        <div className="bg-cyber-card border border-cyber-border rounded-lg p-4 mb-4">
          <h2 className="text-lg font-semibold mb-3 text-cyber-heading font-mono">// Derived Stats</h2>
          <div className="grid grid-cols-2 sm:grid-cols-4 gap-2">
            <StatBox label="Physical CM" value={saved.physical_condition_monitor} />
            <StatBox label="Stun CM" value={saved.stun_condition_monitor} />
            <StatBox label="Initiative" value={`${saved.initiative} + ${saved.initiative_dice}d6`} />
            <StatBox label="Essence" value={(attrs.essence / 100).toFixed(2)} accent="blue" />
            {attrs.magic !== null && <StatBox label="Magic" value={attrs.magic} accent="purple" />}
            {attrs.resonance !== null && <StatBox label="Resonance" value={attrs.resonance} accent="blue" />}
          </div>
        </div>

        {/* Career */}
        <div className="bg-cyber-card border border-cyber-border rounded-lg p-4 mb-4">
          <h2 className="text-lg font-semibold mb-3 text-cyber-heading font-mono">// Career</h2>
          <div className="grid grid-cols-3 gap-2 mb-4">
            <StatBox label="Karma Earned" value={saved.total_karma_earned} accent="green" />
            <StatBox label="Karma Spent" value={saved.total_karma_spent} />
            <StatBox label="Nuyen" value={`¥${saved.nuyen.toLocaleString()}`} accent="green" />
          </div>

          <div className="border-t border-cyber-border pt-3 space-y-3">
            <h3 className="text-xs font-mono text-cyber-text-dim">Apply Event</h3>

            {/* Karma received */}
            <div className="flex gap-2 items-end">
              <div className="flex-1">
                <label className="text-xs text-cyber-text-dim font-mono block mb-1">Karma +</label>
                <div className="flex gap-1">
                  <input type="number" min={1} value={karmaAmount}
                    onChange={(e) => setKarmaAmount(Math.max(1, Number(e.target.value)))}
                    className="w-16 bg-cyber-surface border border-cyber-border rounded px-2 py-1.5 text-sm text-center" />
                  <input type="text" value={karmaReason}
                    onChange={(e) => setKarmaReason(e.target.value)}
                    placeholder="Reason"
                    className="flex-1 bg-cyber-surface border border-cyber-border rounded px-2 py-1.5 text-sm" />
                </div>
              </div>
              <button onClick={handleKarmaReceived} disabled={applying}
                className="px-3 py-1.5 text-xs font-mono border border-cyber-green text-cyber-green hover:bg-cyber-green/10 rounded transition-colors disabled:opacity-40 shrink-0">
                Apply
              </button>
            </div>

            {/* Nuyen received */}
            <div className="flex gap-2 items-end">
              <div className="flex-1">
                <label className="text-xs text-cyber-text-dim font-mono block mb-1">Nuyen +</label>
                <div className="flex gap-1">
                  <input type="number" min={1} value={nuyenReceiveAmount}
                    onChange={(e) => setNuyenReceiveAmount(Math.max(1, Number(e.target.value)))}
                    className="w-24 bg-cyber-surface border border-cyber-border rounded px-2 py-1.5 text-sm text-center" />
                  <input type="text" value={nuyenReceiveReason}
                    onChange={(e) => setNuyenReceiveReason(e.target.value)}
                    placeholder="Reason"
                    className="flex-1 bg-cyber-surface border border-cyber-border rounded px-2 py-1.5 text-sm" />
                </div>
              </div>
              <button onClick={handleNuyenReceived} disabled={applying}
                className="px-3 py-1.5 text-xs font-mono border border-cyber-green text-cyber-green hover:bg-cyber-green/10 rounded transition-colors disabled:opacity-40 shrink-0">
                Apply
              </button>
            </div>

            {/* Nuyen spent */}
            <div className="flex gap-2 items-end">
              <div className="flex-1">
                <label className="text-xs text-cyber-text-dim font-mono block mb-1">Nuyen −</label>
                <div className="flex gap-1">
                  <input type="number" min={1} value={nuyenSpendAmount}
                    onChange={(e) => setNuyenSpendAmount(Math.max(1, Number(e.target.value)))}
                    className="w-24 bg-cyber-surface border border-cyber-border rounded px-2 py-1.5 text-sm text-center" />
                  <input type="text" value={nuyenSpendReason}
                    onChange={(e) => setNuyenSpendReason(e.target.value)}
                    placeholder="Description"
                    className="flex-1 bg-cyber-surface border border-cyber-border rounded px-2 py-1.5 text-sm" />
                </div>
              </div>
              <button onClick={handleNuyenSpent} disabled={applying || nuyenSpendAmount > saved.nuyen}
                className="px-3 py-1.5 text-xs font-mono border border-cyber-red text-cyber-red hover:bg-cyber-red/10 rounded transition-colors disabled:opacity-40 shrink-0">
                Spend
              </button>
            </div>

            {eventError && <p className="text-cyber-red text-xs font-mono">{eventError}</p>}
          </div>
        </div>

        {/* Dice Roller */}
        <div className="bg-cyber-card border border-cyber-border rounded-lg p-4 mb-4">
          <button onClick={() => setShowDice((s) => !s)}
            className="text-lg font-semibold text-cyber-heading font-mono flex items-center gap-2 w-full text-left">
            // Dice Roller
            <span className="text-xs text-cyber-text-dim ml-auto">{showDice ? "▲ collapse" : "▼ expand"}</span>
          </button>

          {showDice && (
            <div className="mt-4">
              {/* Manual pool */}
              <div className="flex gap-2 items-end mb-3">
                <div>
                  <label className="text-xs font-mono text-cyber-text-dim block mb-1">Pool</label>
                  <input type="number" min={1} max={40} value={dicePool}
                    onChange={(e) => setDicePool(Math.min(40, Math.max(1, Number(e.target.value))))}
                    className="w-16 bg-cyber-surface border border-cyber-border rounded px-2 py-1.5 text-sm text-center" />
                </div>
                <button onClick={() => handleRoll(dicePool, `${dicePool}d6`)}
                  className="px-4 py-1.5 text-sm font-mono border border-cyber-green text-cyber-green hover:bg-cyber-green/10 rounded transition-colors">
                  Roll
                </button>
                <button onClick={() => handleRoll(dicePool + (attrs.edge ?? 0), `${dicePool + (attrs.edge ?? 0)}d6 (edge)`)}
                  title={`Roll with +${attrs.edge} Edge`}
                  className="px-4 py-1.5 text-sm font-mono border border-cyber-yellow text-cyber-yellow hover:bg-cyber-yellow/10 rounded transition-colors">
                  Edge ({attrs.edge})
                </button>
              </div>

              {/* Skill pool shortcuts */}
              {skillPools.length > 0 && (
                <div className="mb-3">
                  <p className="text-xs font-mono text-cyber-text-dim mb-2">Quick Roll</p>
                  <div className="flex flex-wrap gap-1.5 max-h-32 overflow-y-auto">
                    {skillPools.map(({ skill, pool }) => (
                      <button key={skill.name}
                        onClick={() => { setDicePool(pool); handleRoll(pool, `${skill.name} (${pool})`); }}
                        className="px-2 py-1 text-xs font-mono border border-cyber-border hover:border-cyber-blue hover:text-cyber-blue rounded transition-colors">
                        {skill.name} ({pool})
                      </button>
                    ))}
                  </div>
                </div>
              )}

              {/* Roll results */}
              <div ref={diceScrollRef} className="space-y-3">
                {rollResults.map((r, i) => (
                  <div key={i} className={`bg-cyber-surface border rounded px-3 py-2 ${r.isCriticalGlitch ? "border-cyber-red" : r.isGlitch ? "border-cyber-yellow" : "border-cyber-border"}`}>
                    <div className="flex items-center justify-between mb-1.5">
                      <span className="text-xs font-mono text-cyber-text-dim">{r.label}</span>
                      <span className="flex items-center gap-3">
                        <span className="text-sm font-bold text-cyber-green font-mono">{r.hits} hit{r.hits !== 1 ? "s" : ""}</span>
                        {r.isCriticalGlitch && <span className="text-xs font-mono text-cyber-red font-bold">CRITICAL GLITCH</span>}
                        {!r.isCriticalGlitch && r.isGlitch && <span className="text-xs font-mono text-cyber-yellow font-bold">GLITCH</span>}
                      </span>
                    </div>
                    <div className="flex flex-wrap gap-1">
                      {r.dice.map((d, j) => <DieIcon key={j} value={d} />)}
                    </div>
                  </div>
                ))}
                {rollResults.length === 0 && (
                  <p className="text-xs font-mono text-cyber-text-dim">No rolls yet.</p>
                )}
              </div>
            </div>
          )}
        </div>

        {/* Full Character Sheet (collapsible) */}
        <div className="bg-cyber-card border border-cyber-border rounded-lg p-4 mb-4">
          <button onClick={() => setShowSheet((s) => !s)}
            className="text-lg font-semibold text-cyber-heading font-mono flex items-center gap-2 w-full text-left">
            // Character Sheet
            <span className="text-xs text-cyber-text-dim ml-auto">{showSheet ? "▲ collapse" : "▼ expand"}</span>
          </button>

          {showSheet && (
            <div className="mt-4 space-y-4">

              {/* Qualities */}
              {base.qualities.length > 0 && (
                <SheetSection title="Qualities">
                  <div className="space-y-1">
                    {base.qualities.map((q) => (
                      <div key={q.id} className="flex items-center gap-2 bg-cyber-surface border border-cyber-border rounded px-3 py-1.5 text-sm">
                        <span className={`text-xs font-mono px-1.5 py-0.5 rounded shrink-0 ${q.quality_type === "Positive" ? "bg-cyber-green-dim/30 text-cyber-green border border-cyber-green-dim" : "bg-cyber-red-dim/30 text-cyber-red border border-cyber-red-dim"}`}>
                          {q.quality_type === "Positive" ? "+" : "−"}
                        </span>
                        <span className="flex-1 text-cyber-text">{q.name}</span>
                        <span className="text-cyber-text-dim font-mono text-xs shrink-0">{q.cost} BP</span>
                      </div>
                    ))}
                  </div>
                </SheetSection>
              )}

              {/* Augmentations */}
              {base.augmentations.length > 0 && (
                <SheetSection title="Augmentations">
                  <div className="space-y-1">
                    {base.augmentations.map((a) => (
                      <div key={a.id} className="flex items-center gap-2 bg-cyber-surface border border-cyber-border rounded px-3 py-1.5 text-sm">
                        <span className="text-xs font-mono px-1.5 py-0.5 rounded shrink-0 bg-cyber-blue/10 border border-cyber-blue/30 text-cyber-blue">
                          {a.augmentation_type.slice(0, 3)}
                        </span>
                        <span className="flex-1 text-cyber-text">{a.name}</span>
                        <span className="text-cyber-text-dim font-mono text-xs shrink-0">{a.grade}</span>
                        <span className="text-cyber-yellow font-mono text-xs shrink-0">
                          {(a.essence_cost / 100).toFixed(2)}E
                        </span>
                      </div>
                    ))}
                  </div>
                </SheetSection>
              )}

              {/* Spells */}
              {hasSpells && base.spells.length > 0 && (
                <SheetSection title="Spells">
                  <div className="space-y-1">
                    {base.spells.map((s) => (
                      <div key={s.id} className="flex items-center gap-2 bg-cyber-surface border border-cyber-border rounded px-3 py-1.5 text-sm">
                        <span className="flex-1 text-cyber-text">{s.name}</span>
                        <span className="text-cyber-text-dim font-mono text-xs shrink-0">{s.category} · {s.spell_type}</span>
                        <span className="text-cyber-blue font-mono text-xs shrink-0">{s.drain}</span>
                      </div>
                    ))}
                  </div>
                </SheetSection>
              )}

              {/* Adept Powers */}
              {hasPowers && base.adept_powers.length > 0 && (
                <SheetSection title="Adept Powers">
                  <div className="space-y-1">
                    {base.adept_powers.map((p) => (
                      <div key={p.id} className="flex items-center gap-2 bg-cyber-surface border border-cyber-border rounded px-3 py-1.5 text-sm">
                        <span className="flex-1 text-cyber-text">{p.name}</span>
                        {p.levels && <span className="text-cyber-text-dim font-mono text-xs">leveled</span>}
                        <span className="text-cyber-blue font-mono text-xs shrink-0">
                          {(p.cost / 100).toFixed(2)} PP
                        </span>
                      </div>
                    ))}
                  </div>
                </SheetSection>
              )}

              {/* Complex Forms */}
              {hasForms && base.complex_forms.length > 0 && (
                <SheetSection title="Complex Forms">
                  <div className="space-y-1">
                    {base.complex_forms.map((f) => (
                      <div key={f.id} className="flex items-center gap-2 bg-cyber-surface border border-cyber-border rounded px-3 py-1.5 text-sm">
                        <span className="flex-1 text-cyber-text">{f.name}</span>
                        <span className="text-cyber-text-dim font-mono text-xs shrink-0">{f.target} · {f.duration}</span>
                        <span className="text-cyber-blue font-mono text-xs shrink-0">{f.fading}</span>
                      </div>
                    ))}
                  </div>
                </SheetSection>
              )}

              {/* Contacts */}
              {base.contacts.length > 0 && (
                <SheetSection title="Contacts">
                  <div className="space-y-1">
                    {base.contacts.map((c) => (
                      <div key={c.id} className="flex items-center gap-2 bg-cyber-surface border border-cyber-border rounded px-3 py-1.5 text-sm">
                        <div className="flex-1 min-w-0">
                          <span className="text-cyber-text font-medium">{c.name}</span>
                          {c.archetype && <span className="text-cyber-text-dim font-mono text-xs ml-2">{c.archetype}</span>}
                        </div>
                        <span className="text-cyber-blue font-mono text-xs shrink-0">
                          {c.connection}/{c.loyalty}
                        </span>
                      </div>
                    ))}
                  </div>
                </SheetSection>
              )}

              {/* Knowledge Skills */}
              {base.knowledge_skills && base.knowledge_skills.length > 0 && (
                <SheetSection title="Knowledge &amp; Language Skills">
                  <div className="space-y-1">
                    {base.knowledge_skills.map((k) => (
                      <div key={k.name} className="flex items-center gap-2 bg-cyber-surface border border-cyber-border rounded px-3 py-1.5 text-sm">
                        <span className="flex-1 text-cyber-text">{k.name}</span>
                        <span className="text-cyber-text-dim font-mono text-xs shrink-0">{k.category}</span>
                        <span className="text-cyber-blue font-mono text-xs shrink-0">{k.rating}</span>
                      </div>
                    ))}
                  </div>
                </SheetSection>
              )}

              {/* Weapons */}
              {base.weapons.length > 0 && (
                <SheetSection title="Weapons">
                  <div className="space-y-1">
                    {base.weapons.map((w) => (
                      <div key={w.id} className="flex items-center gap-2 bg-cyber-surface border border-cyber-border rounded px-3 py-1.5 text-sm">
                        <span className="flex-1 text-cyber-text">{w.name}</span>
                        <span className="text-cyber-text-dim font-mono text-xs shrink-0">{w.category}</span>
                        <span className="text-cyber-yellow font-mono text-xs shrink-0">{w.damage}</span>
                      </div>
                    ))}
                  </div>
                </SheetSection>
              )}

              {/* Armor */}
              {base.armor.length > 0 && (
                <SheetSection title="Armor">
                  <div className="space-y-1">
                    {base.armor.map((a) => (
                      <div key={a.id} className="flex items-center gap-2 bg-cyber-surface border border-cyber-border rounded px-3 py-1.5 text-sm">
                        <span className="flex-1 text-cyber-text">{a.name}</span>
                        <span className="text-cyber-blue font-mono text-xs shrink-0">
                          {a.armor_value} armor
                        </span>
                      </div>
                    ))}
                  </div>
                </SheetSection>
              )}

              {/* Vehicles */}
              {base.vehicles.length > 0 && (
                <SheetSection title="Vehicles">
                  <div className="space-y-1">
                    {base.vehicles.map((v) => (
                      <div key={v.id} className="flex items-center gap-2 bg-cyber-surface border border-cyber-border rounded px-3 py-1.5 text-sm">
                        <span className="flex-1 text-cyber-text">{v.name}</span>
                        <span className="text-cyber-text-dim font-mono text-xs shrink-0">
                          Bod {v.body} / Han {v.handling} / Pil {v.pilot}
                        </span>
                      </div>
                    ))}
                  </div>
                </SheetSection>
              )}

              {base.qualities.length === 0 && base.augmentations.length === 0 &&
               base.spells.length === 0 && base.adept_powers.length === 0 &&
               base.complex_forms.length === 0 && base.contacts.length === 0 &&
               base.weapons.length === 0 && base.armor.length === 0 &&
               base.vehicles.length === 0 && (
                <p className="text-cyber-text-dim text-sm font-mono">No equipment recorded.</p>
              )}
            </div>
          )}
        </div>

        {/* Character Notes */}
        <div className="bg-cyber-card border border-cyber-border rounded-lg p-4 mb-4">
          <button
            onClick={() => {
              if (!showNotes && notesText === null) setNotesText(base.notes ?? "");
              setShowNotes((s) => !s);
            }}
            className="text-lg font-semibold text-cyber-heading font-mono flex items-center gap-2 w-full text-left"
          >
            // Character Notes
            <span className="text-xs text-cyber-text-dim ml-auto">{showNotes ? "▲ collapse" : "▼ expand"}</span>
          </button>

          {showNotes && (
            <div className="mt-3">
              <textarea
                value={notesText ?? ""}
                onChange={(e) => setNotesText(e.target.value)}
                placeholder="Backstory, run log, contacts encountered…"
                rows={8}
                className="w-full bg-cyber-surface border border-cyber-border rounded px-3 py-2 text-sm text-cyber-text placeholder-cyber-text-dim focus:border-cyber-green outline-none resize-y font-mono"
              />
              <div className="flex items-center gap-3 mt-2">
                <button
                  onClick={async () => {
                    setNotesSaving(true);
                    try { await updateNotes(characterId, notesText ?? ""); }
                    finally { setNotesSaving(false); }
                  }}
                  disabled={notesSaving}
                  className="px-3 py-1.5 rounded bg-cyber-green/10 border border-cyber-green text-cyber-green text-sm hover:bg-cyber-green/20 disabled:opacity-40 transition-colors"
                >
                  {notesSaving ? "Saving…" : "Save Notes"}
                </button>
                <span className="text-xs text-cyber-text-dim">Auto-saved on blur is not enabled — click Save.</span>
              </div>
            </div>
          )}
        </div>

        {/* Career Timeline */}
        <div className="bg-cyber-card border border-cyber-border rounded-lg p-4 mb-4">
          <button onClick={() => setShowLedger((s) => !s)}
            className="text-lg font-semibold text-cyber-heading font-mono flex items-center gap-2 w-full text-left">
            // Career Timeline
            <span className="text-xs text-cyber-text-dim ml-auto">{showLedger ? "▲ collapse" : "▼ expand"}</span>
          </button>

          {showLedger && (
            <div className="mt-3">
              {ledgerLoading && <p className="text-xs text-cyber-text-dim font-mono">Loading...</p>}
              {!ledgerLoading && ledgerEvents.length === 0 && (
                <p className="text-xs text-cyber-text-dim font-mono">No events recorded yet.</p>
              )}
              {!ledgerLoading && ledgerEvents.length > 0 && (
                <div className="space-y-1 max-h-64 overflow-y-auto">
                  {ledgerEvents.map((evt, i) => {
                    const { label, detail, color } = formatEvent(evt);
                    return (
                      <div key={i} className="bg-cyber-surface border border-cyber-border rounded px-3 py-1.5 flex items-center justify-between">
                        <span className={`text-xs font-mono ${color}`}>{label}</span>
                        <span className="text-xs text-cyber-text-dim ml-3 truncate max-w-xs text-right">{detail}</span>
                      </div>
                    );
                  })}
                </div>
              )}
            </div>
          )}
        </div>

        {/* Karma Improvements */}
        <div className="bg-cyber-card border border-cyber-border rounded-lg p-4">
          <button onClick={() => setShowImprove((s) => !s)}
            className="text-lg font-semibold text-cyber-heading font-mono flex items-center gap-2 w-full text-left">
            // Karma Improvements
            <span className="text-xs text-cyber-text-dim ml-auto">{showImprove ? "▲ collapse" : "▼ expand"}</span>
          </button>
          <p className="text-xs text-cyber-text-dim font-mono mt-1 mb-3">
            Available: <span className="text-cyber-green">{saved.total_karma_earned - saved.total_karma_spent}</span> karma
          </p>

          {showImprove && (
            <>
              <h3 className="text-xs font-mono text-cyber-text-dim mb-2">Attributes (cost = new rating × 5)</h3>
              <div className="grid grid-cols-2 sm:grid-cols-3 gap-2 mb-4">
                {(
                  [
                    ["BOD", "body", saved.computed_attributes.body],
                    ["AGI", "agility", saved.computed_attributes.agility],
                    ["REA", "reaction", saved.computed_attributes.reaction],
                    ["STR", "strength", saved.computed_attributes.strength],
                    ["WIL", "willpower", saved.computed_attributes.willpower],
                    ["LOG", "logic", saved.computed_attributes.logic],
                    ["INT", "intuition", saved.computed_attributes.intuition],
                    ["CHA", "charisma", saved.computed_attributes.charisma],
                    ["EDG", "edge", saved.computed_attributes.edge],
                  ] as [string, string, number][]
                ).map(([label, key, current]) => {
                  const cost = (current + 1) * 5;
                  const canAfford = saved.total_karma_earned - saved.total_karma_spent >= cost;
                  return (
                    <div key={key} className="bg-cyber-surface border border-cyber-border rounded px-2 py-1.5 flex items-center justify-between">
                      <span className="text-xs font-mono text-cyber-blue">{label} {current}</span>
                      <button
                        onClick={() => handleAttrImprove(key, current)}
                        disabled={applying || !canAfford}
                        title={`${cost} karma to reach ${current + 1}`}
                        className="text-xs px-2 py-0.5 border border-cyber-border hover:border-cyber-green text-cyber-text hover:text-cyber-green rounded transition-colors disabled:opacity-30">
                        +{cost}k
                      </button>
                    </div>
                  );
                })}
              </div>

              {base.skills.length > 0 && (
                <>
                  <h3 className="text-xs font-mono text-cyber-text-dim mb-2">Skills (cost = new rating × 2)</h3>
                  <div className="space-y-1 max-h-48 overflow-y-auto">
                    {base.skills.map((skill) => {
                      const cost = (skill.rating + 1) * 2;
                      const canAfford = saved.total_karma_earned - saved.total_karma_spent >= cost;
                      return (
                        <div key={skill.name} className="bg-cyber-surface border border-cyber-border rounded px-2 py-1 flex items-center justify-between">
                          <span className="text-sm text-cyber-text">{skill.name}</span>
                          <div className="flex items-center gap-2">
                            <span className="text-xs font-mono text-cyber-blue">{skill.rating}</span>
                            <button
                              onClick={() => handleSkillImprove(skill.name, skill.rating)}
                              disabled={applying || !canAfford}
                              title={`${cost} karma to reach ${skill.rating + 1}`}
                              className="text-xs px-2 py-0.5 border border-cyber-border hover:border-cyber-green text-cyber-text hover:text-cyber-green rounded transition-colors disabled:opacity-30">
                              +{cost}k
                            </button>
                          </div>
                        </div>
                      );
                    })}
                  </div>
                </>
              )}

              {base.contacts.length > 0 && (
                <>
                  <h3 className="text-xs font-mono text-cyber-text-dim mb-2 mt-4">
                    Contacts (cost = new rating in karma)
                  </h3>
                  <div className="space-y-1 max-h-48 overflow-y-auto">
                    {base.contacts.map((c) => {
                      const connCost = c.connection + 1;
                      const loyalCost = c.loyalty + 1;
                      const available = saved.total_karma_earned - saved.total_karma_spent;
                      const canConn = available >= connCost && c.connection < 6;
                      const canLoyal = available >= loyalCost && c.loyalty < 6;
                      return (
                        <div key={c.id} className="bg-cyber-surface border border-cyber-border rounded px-2 py-1.5 flex items-center justify-between">
                          <div className="flex-1 min-w-0">
                            <span className="text-sm text-cyber-text">{c.name}</span>
                            {c.archetype && <span className="text-cyber-text-dim font-mono text-xs ml-2">{c.archetype}</span>}
                          </div>
                          <div className="flex items-center gap-2 shrink-0">
                            <span className="text-xs font-mono text-cyber-text-dim">C{c.connection}</span>
                            <button
                              onClick={() => handleContactImprove(c.id, c.name, c.connection, c.loyalty, "connection")}
                              disabled={applying || !canConn}
                              title={`${connCost}k → Connection ${c.connection + 1}`}
                              className="text-xs px-1.5 py-0.5 border border-cyber-border hover:border-cyber-green text-cyber-text hover:text-cyber-green rounded transition-colors disabled:opacity-30">
                              +C{connCost}k
                            </button>
                            <span className="text-xs font-mono text-cyber-text-dim">L{c.loyalty}</span>
                            <button
                              onClick={() => handleContactImprove(c.id, c.name, c.connection, c.loyalty, "loyalty")}
                              disabled={applying || !canLoyal}
                              title={`${loyalCost}k → Loyalty ${c.loyalty + 1}`}
                              className="text-xs px-1.5 py-0.5 border border-cyber-border hover:border-cyber-green text-cyber-text hover:text-cyber-green rounded transition-colors disabled:opacity-30">
                              +L{loyalCost}k
                            </button>
                          </div>
                        </div>
                      );
                    })}
                  </div>
                </>
              )}
            </>
          )}
        </div>
      </div>

      {/* Print-only sheet — hidden on screen, shown via @media print */}
      <div id="print-sheet" style={{ display: "none" }}>
        <h1>{base.name}</h1>
        <p style={{ color: "#555", fontSize: "0.85rem", marginBottom: "0.5rem" }}>
          {base.edition} · {base.metatype}{tradition ? ` · ${tradition}${base.tradition_name ? ` (${base.tradition_name})` : ""}` : ""}
        </p>

        <h2>Attributes</h2>
        <div className="grid-2">
          {([
            ["Body", attrs.body], ["Agility", attrs.agility], ["Reaction", attrs.reaction],
            ["Strength", attrs.strength], ["Willpower", attrs.willpower], ["Logic", attrs.logic],
            ["Intuition", attrs.intuition], ["Charisma", attrs.charisma], ["Edge", attrs.edge],
          ] as [string, number][]).map(([label, value]) => (
            <div key={label} className="attr-box">
              <div className="attr-label">{label}</div>
              <div className="attr-value">{value}</div>
            </div>
          ))}
          <div className="attr-box"><div className="attr-label">Essence</div><div className="attr-value">{(attrs.essence / 100).toFixed(2)}</div></div>
          {attrs.magic !== null && <div className="attr-box"><div className="attr-label">Magic</div><div className="attr-value">{attrs.magic}</div></div>}
          {attrs.resonance !== null && <div className="attr-box"><div className="attr-label">Resonance</div><div className="attr-value">{attrs.resonance}</div></div>}
        </div>

        <h2>Derived Stats</h2>
        <div className="grid-2">
          <div className="attr-box"><div className="attr-label">Physical CM</div><div className="attr-value">{saved.physical_condition_monitor}</div></div>
          <div className="attr-box"><div className="attr-label">Stun CM</div><div className="attr-value">{saved.stun_condition_monitor}</div></div>
          <div className="attr-box"><div className="attr-label">Initiative</div><div className="attr-value">{saved.initiative}+{saved.initiative_dice}d6</div></div>
          <div className="attr-box"><div className="attr-label">Karma / Nuyen</div><div className="attr-value">{saved.total_karma_earned - saved.total_karma_spent}k / ¥{saved.nuyen.toLocaleString()}</div></div>
        </div>

        {base.qualities.length > 0 && (<>
          <h2>Qualities</h2>
          {base.qualities.map((q) => (
            <div key={q.id} className="item-row"><span>{q.name}</span><span>{q.quality_type} ({q.cost} BP)</span></div>
          ))}
        </>)}

        {base.augmentations.length > 0 && (<>
          <h2>Augmentations</h2>
          {base.augmentations.map((a) => (
            <div key={a.id} className="item-row"><span>{a.name}</span><span>{a.augmentation_type} · {a.grade} · {(a.essence_cost / 100).toFixed(2)}E</span></div>
          ))}
        </>)}

        {base.skills.length > 0 && (<>
          <h2>Skills</h2>
          {base.skills.map((s) => (
            <div key={s.name} className="item-row"><span>{s.name}</span><span>{s.rating} ({s.linked_attribute})</span></div>
          ))}
        </>)}

        {base.knowledge_skills && base.knowledge_skills.length > 0 && (<>
          <h2>Knowledge &amp; Language Skills</h2>
          {base.knowledge_skills.map((k) => (
            <div key={k.name} className="item-row"><span>{k.name}</span><span>{k.rating} ({k.category})</span></div>
          ))}
        </>)}

        {hasSpells && base.spells.length > 0 && (<>
          <h2>Spells</h2>
          {base.spells.map((s) => (
            <div key={s.id} className="item-row"><span>{s.name}</span><span>{s.category} · {s.drain}</span></div>
          ))}
        </>)}

        {hasPowers && base.adept_powers.length > 0 && (<>
          <h2>Adept Powers</h2>
          {base.adept_powers.map((p) => (
            <div key={p.id} className="item-row"><span>{p.name}</span><span>{(p.cost / 100).toFixed(2)} PP</span></div>
          ))}
        </>)}

        {hasForms && base.complex_forms.length > 0 && (<>
          <h2>Complex Forms</h2>
          {base.complex_forms.map((f) => (
            <div key={f.id} className="item-row"><span>{f.name}</span><span>{f.target} · {f.fading}</span></div>
          ))}
        </>)}

        {base.contacts.length > 0 && (<>
          <h2>Contacts</h2>
          {base.contacts.map((c) => (
            <div key={c.id} className="item-row"><span>{c.name} {c.archetype ? `(${c.archetype})` : ""}</span><span>C{c.connection}/L{c.loyalty}</span></div>
          ))}
        </>)}

        {base.weapons.length > 0 && (<>
          <h2>Weapons</h2>
          {base.weapons.map((w) => (
            <div key={w.id} className="item-row"><span>{w.name}</span><span>{w.damage} · {w.mode}</span></div>
          ))}
        </>)}

        {base.armor.length > 0 && (<>
          <h2>Armor</h2>
          {base.armor.map((a) => (
            <div key={a.id} className="item-row"><span>{a.name}</span><span>Armor {a.armor_value}</span></div>
          ))}
        </>)}

        {base.vehicles.length > 0 && (<>
          <h2>Vehicles</h2>
          {base.vehicles.map((v) => (
            <div key={v.id} className="item-row"><span>{v.name}</span><span>Bod {v.body} / Han {v.handling} / Pil {v.pilot}</span></div>
          ))}
        </>)}
      </div>
    </div>
  );
}

function SheetSection({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div>
      <h3 className="text-sm font-mono text-cyber-text-dim mb-2">// {title}</h3>
      {children}
    </div>
  );
}

function formatEvent(evt: LedgerEvent): { label: string; detail: string; color: string } {
  if ("KarmaReceived" in evt) return { label: `+${evt.KarmaReceived.amount} Karma`, detail: evt.KarmaReceived.reason, color: "text-cyber-green" };
  if ("KarmaSpent" in evt) return { label: `-${evt.KarmaSpent.amount} Karma`, detail: evt.KarmaSpent.description, color: "text-cyber-red" };
  if ("NuyenReceived" in evt) return { label: `+¥${evt.NuyenReceived.amount.toLocaleString()}`, detail: evt.NuyenReceived.reason, color: "text-cyber-green" };
  if ("NuyenSpent" in evt) return { label: `-¥${evt.NuyenSpent.amount.toLocaleString()}`, detail: evt.NuyenSpent.description, color: "text-cyber-red" };
  if ("SkillImproved" in evt) return { label: `Skill: ${evt.SkillImproved.skill_name}`, detail: `${evt.SkillImproved.from} → ${evt.SkillImproved.to} (${evt.SkillImproved.karma_cost}k)`, color: "text-cyber-blue" };
  if ("AttributeImproved" in evt) return { label: `Attr: ${evt.AttributeImproved.attribute}`, detail: `${evt.AttributeImproved.from} → ${evt.AttributeImproved.to} (${evt.AttributeImproved.karma_cost}k)`, color: "text-cyber-blue" };
  if ("ContactChanged" in evt) return { label: `Contact updated`, detail: `C${evt.ContactChanged.new_connection}/L${evt.ContactChanged.new_loyalty}`, color: "text-cyber-blue" };
  if ("ContactAdded" in evt) return { label: `Contact: ${evt.ContactAdded.name}`, detail: `C${evt.ContactAdded.connection}/L${evt.ContactAdded.loyalty}`, color: "text-cyber-green" };
  if ("ContactLost" in evt) return { label: `Contact lost`, detail: evt.ContactLost.reason, color: "text-cyber-red" };
  return { label: "Event", detail: JSON.stringify(evt), color: "text-cyber-text-dim" };
}

function StatBox({ label, value, accent }: { label: string; value: string | number; accent?: "green" | "blue" | "purple" }) {
  const valueColor = accent === "green" ? "text-cyber-green" : accent === "blue" ? "text-cyber-blue" : accent === "purple" ? "text-cyber-purple" : "text-cyber-heading";
  return (
    <div className="bg-cyber-surface border border-cyber-border rounded px-3 py-2 text-center">
      <div className="text-cyber-text-dim text-xs font-mono">{label}</div>
      <div className={`text-lg font-bold font-mono ${valueColor}`}>{value}</div>
    </div>
  );
}
