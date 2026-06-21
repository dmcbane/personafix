import { useRef, useState } from "react";
import { useCharacterStore } from "../store/characterStore";
import { rollPool, DieIcon, type RollResult } from "./DiceRoller";

interface Props {
  onBack: () => void;
}

function ConditionBox({ filled }: { filled: boolean }) {
  return (
    <span
      className={`inline-flex w-6 h-6 border rounded items-center justify-center text-xs font-bold transition-colors ${
        filled
          ? "bg-cyber-red/20 border-cyber-red text-cyber-red"
          : "bg-cyber-surface border-cyber-border text-cyber-text-dim"
      }`}
    >
      {filled ? "✕" : "○"}
    </span>
  );
}

export default function PlayView({ onBack }: Props) {
  const saved = useCharacterStore((s) => s.savedCharacter);
  if (!saved) return null;

  const attrs = saved.computed_attributes;
  const physBoxes = saved.physical_condition_monitor;
  const stunBoxes = saved.stun_condition_monitor;
  const edgeMax = attrs.edge;

  const [physFilled, setPhysFilled] = useState(0);
  const [stunFilled, setStunFilled] = useState(0);
  const [edgeSpent, setEdgeSpent] = useState(0);
  const [dicePool, setDicePool] = useState(attrs.reaction + attrs.intuition);
  const [rollResults, setRollResults] = useState<RollResult[]>([]);
  const [initScore, setInitScore] = useState<number | null>(null);
  const [initPass, setInitPass] = useState(1);
  const rollScrollRef = useRef<HTMLDivElement>(null);

  const woundMod = -Math.floor((physFilled + stunFilled) / 3);
  const edgeRemaining = edgeMax - edgeSpent;

  const rollInitiative = () => {
    const base = attrs.reaction + attrs.intuition;
    const dice = saved.initiative_dice;
    const r = rollPool(dice, `Initiative (REA ${attrs.reaction} + INT ${attrs.intuition} + ${dice}d6)`);
    const score = base + r.hits;
    setInitScore(score);
    setInitPass(1);
    setRollResults((prev) => [...prev, r]);
    setTimeout(() => rollScrollRef.current?.scrollTo({ top: 0, behavior: "smooth" }), 50);
  };

  const rollDice = () => {
    const r = rollPool(dicePool, `${dicePool}d6`);
    setRollResults((prev) => [...prev, r]);
    setTimeout(() => rollScrollRef.current?.scrollTo({ top: 0, behavior: "smooth" }), 50);
  };

  const togglePhys = (i: number) => setPhysFilled((prev) => (prev === i + 1 ? i : i + 1));
  const toggleStun = (i: number) => setStunFilled((prev) => (prev === i + 1 ? i : i + 1));

  return (
    <div className="min-h-screen p-6 max-w-4xl mx-auto">
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-2xl font-bold text-cyber-heading">{saved.base.name}</h1>
          <p className="text-cyber-text-dim text-sm font-mono">
            {saved.base.edition} {saved.base.metatype} // Play Mode
          </p>
        </div>
        <button
          onClick={onBack}
          className="px-4 py-2 bg-cyber-card border border-cyber-border hover:border-cyber-border-bright rounded text-sm text-cyber-text transition-colors"
        >
          ← Sheet
        </button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        {/* Column 1: Condition Monitors */}
        <div className="space-y-4">
          {/* Physical CM */}
          <div className="bg-cyber-card border border-cyber-border rounded-lg p-4">
            <div className="flex items-center justify-between mb-3">
              <h2 className="text-sm font-semibold text-cyber-heading font-mono">// Physical</h2>
              <span className="text-xs font-mono text-cyber-text-dim">
                {physFilled}/{physBoxes}
              </span>
            </div>
            <div className="flex flex-wrap gap-1 mb-2">
              {Array.from({ length: physBoxes }).map((_, i) => (
                <button key={i} onClick={() => togglePhys(i)}>
                  <ConditionBox filled={i < physFilled} />
                </button>
              ))}
            </div>
            <button
              onClick={() => setPhysFilled(0)}
              className="text-xs font-mono text-cyber-text-dim hover:text-cyber-text transition-colors"
            >
              Clear
            </button>
          </div>

          {/* Stun CM */}
          <div className="bg-cyber-card border border-cyber-border rounded-lg p-4">
            <div className="flex items-center justify-between mb-3">
              <h2 className="text-sm font-semibold text-cyber-heading font-mono">// Stun</h2>
              <span className="text-xs font-mono text-cyber-text-dim">
                {stunFilled}/{stunBoxes}
              </span>
            </div>
            <div className="flex flex-wrap gap-1 mb-2">
              {Array.from({ length: stunBoxes }).map((_, i) => (
                <button key={i} onClick={() => toggleStun(i)}>
                  <ConditionBox filled={i < stunFilled} />
                </button>
              ))}
            </div>
            <button
              onClick={() => setStunFilled(0)}
              className="text-xs font-mono text-cyber-text-dim hover:text-cyber-text transition-colors"
            >
              Clear
            </button>
          </div>

          {/* Wound modifier */}
          {woundMod < 0 && (
            <div className="bg-cyber-red-dim/20 border border-cyber-red/30 rounded p-3 text-center">
              <span className="text-cyber-red font-mono font-bold text-lg">{woundMod}</span>
              <p className="text-cyber-red text-xs font-mono">wound modifier</p>
            </div>
          )}
        </div>

        {/* Column 2: Initiative + Edge */}
        <div className="space-y-4">
          {/* Initiative */}
          <div className="bg-cyber-card border border-cyber-border rounded-lg p-4">
            <h2 className="text-sm font-semibold text-cyber-heading font-mono mb-3">// Initiative</h2>
            <div className="text-xs text-cyber-text-dim font-mono mb-3">
              REA {attrs.reaction} + INT {attrs.intuition} + {saved.initiative_dice}d6
            </div>
            <button
              onClick={rollInitiative}
              className="w-full px-4 py-2 bg-cyber-green-dim hover:bg-cyber-green/20 border border-cyber-green-dim hover:border-cyber-green rounded text-sm font-mono text-cyber-green transition-all mb-3"
            >
              Roll Initiative
            </button>
            {initScore !== null && (
              <div className="space-y-2">
                <div className="text-center">
                  <span className="text-2xl font-bold font-mono text-cyber-green">{initScore}</span>
                  <span className="text-cyber-text-dim text-sm font-mono ml-2">score</span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-xs text-cyber-text-dim font-mono">Pass</span>
                  <div className="flex items-center gap-2">
                    <button
                      onClick={() => setInitPass((p) => Math.max(1, p - 1))}
                      className="w-6 h-6 border border-cyber-border rounded text-cyber-text-dim hover:text-cyber-text text-sm"
                    >
                      −
                    </button>
                    <span className="font-mono text-cyber-text w-4 text-center">{initPass}</span>
                    <button
                      onClick={() => setInitPass((p) => p + 1)}
                      className="w-6 h-6 border border-cyber-border rounded text-cyber-text-dim hover:text-cyber-text text-sm"
                    >
                      +
                    </button>
                  </div>
                </div>
                <div className="text-center">
                  <span className="text-lg font-bold font-mono text-cyber-blue">
                    {Math.max(0, initScore - (initPass - 1) * 10)}
                  </span>
                  <span className="text-cyber-text-dim text-xs font-mono ml-1">this pass</span>
                </div>
              </div>
            )}
          </div>

          {/* Edge */}
          <div className="bg-cyber-card border border-cyber-border rounded-lg p-4">
            <div className="flex items-center justify-between mb-3">
              <h2 className="text-sm font-semibold text-cyber-heading font-mono">// Edge</h2>
              <span className="text-xs font-mono text-cyber-text-dim">
                {edgeRemaining}/{edgeMax}
              </span>
            </div>
            <div className="flex flex-wrap gap-1 mb-2">
              {Array.from({ length: edgeMax }).map((_, i) => (
                <button
                  key={i}
                  onClick={() => setEdgeSpent(i < edgeSpent ? i : i + 1)}
                >
                  <ConditionBox filled={i < edgeSpent} />
                </button>
              ))}
            </div>
            <button
              onClick={() => setEdgeSpent(0)}
              className="text-xs font-mono text-cyber-text-dim hover:text-cyber-text transition-colors"
            >
              Reset
            </button>
          </div>
        </div>

        {/* Column 3: Dice Roller */}
        <div className="bg-cyber-card border border-cyber-border rounded-lg p-4">
          <h2 className="text-sm font-semibold text-cyber-heading font-mono mb-3">// Dice Roller</h2>
          <div className="flex items-center gap-2 mb-3">
            <input
              type="number"
              min={1}
              max={30}
              value={dicePool}
              onChange={(e) => setDicePool(Math.max(1, Math.min(30, Number(e.target.value))))}
              className="w-16 bg-cyber-card border border-cyber-border rounded px-2 py-1 text-sm font-mono text-center"
            />
            <span className="text-cyber-text-dim text-xs font-mono">dice</span>
            <button
              onClick={rollDice}
              className="px-4 py-1 bg-cyber-green-dim hover:bg-cyber-green/20 border border-cyber-green-dim hover:border-cyber-green rounded text-sm font-mono text-cyber-green transition-all"
            >
              Roll
            </button>
          </div>

          {rollResults.length > 0 && (
            <div ref={rollScrollRef} className="max-h-64 overflow-y-auto space-y-2 pr-1">
              {[...rollResults].reverse().map((r, i) => (
                <div key={i} className="bg-cyber-surface border border-cyber-border rounded p-2 space-y-1">
                  <div className="flex items-center justify-between">
                    <span className="text-xs font-mono text-cyber-text-dim truncate">{r.label}</span>
                    <span
                      className={`text-sm font-bold font-mono shrink-0 ml-1 ${
                        r.isCriticalGlitch
                          ? "text-cyber-red"
                          : r.isGlitch
                            ? "text-cyber-yellow"
                            : "text-cyber-green"
                      }`}
                    >
                      {r.hits}h
                      {r.isCriticalGlitch ? " CG" : r.isGlitch ? " G" : ""}
                    </span>
                  </div>
                  <div className="flex flex-wrap gap-1">
                    {r.dice.map((d, j) => <DieIcon key={j} value={d} />)}
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>

      {/* Quick stats bar */}
      <div className="mt-4 bg-cyber-card border border-cyber-border rounded-lg p-3">
        <div className="flex flex-wrap gap-4 text-xs font-mono">
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
              ...(attrs.magic != null ? [["MAG", attrs.magic] as [string, number]] : []),
              ...(attrs.resonance != null ? [["RES", attrs.resonance] as [string, number]] : []),
            ] as [string, number][]
          ).map(([label, val]) => (
            <span key={label} className="text-cyber-text-dim">
              {label} <span className="text-cyber-text font-bold">{val}</span>
            </span>
          ))}
          {woundMod < 0 && (
            <span className="text-cyber-red">
              Wound <span className="font-bold">{woundMod}</span>
            </span>
          )}
        </div>
      </div>
    </div>
  );
}
