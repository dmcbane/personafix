export interface RollResult {
  dice: number[];
  hits: number;
  ones: number;
  isGlitch: boolean;
  isCriticalGlitch: boolean;
  label: string;
}

export function rollPool(count: number, label: string): RollResult {
  const dice = Array.from({ length: Math.max(1, count) }, () => Math.floor(Math.random() * 6) + 1);
  const hits = dice.filter((d) => d >= 5).length;
  const ones = dice.filter((d) => d === 1).length;
  const isGlitch = ones > dice.length / 2;
  return { dice, hits, ones, isGlitch, isCriticalGlitch: isGlitch && hits === 0, label };
}

export function DieIcon({ value }: { value: number }) {
  const isHit = value >= 5;
  const isOne = value === 1;
  const color = isHit
    ? "text-cyber-green bg-cyber-green/10 border-cyber-green/50"
    : isOne
      ? "text-cyber-red bg-cyber-red/10 border-cyber-red/50"
      : "text-cyber-text-dim bg-cyber-surface border-cyber-border";
  return (
    <span className={`inline-flex items-center justify-center w-7 h-7 rounded text-xs font-bold font-mono border ${color}`}>
      {value}
    </span>
  );
}

interface DiceRollerProps {
  pool: number;
  onPoolChange: (n: number) => void;
  results: RollResult[];
  onRoll: () => void;
  scrollRef?: React.RefObject<HTMLDivElement | null>;
}

export function DiceRollerPanel({ pool, onPoolChange, results, onRoll, scrollRef }: DiceRollerProps) {
  return (
    <div className="space-y-2">
      <div className="flex items-center gap-2">
        <input
          type="number"
          min={1}
          max={30}
          value={pool}
          onChange={(e) => onPoolChange(Math.max(1, Math.min(30, Number(e.target.value))))}
          className="w-16 bg-cyber-card border border-cyber-border rounded px-2 py-1 text-sm font-mono text-center"
        />
        <span className="text-cyber-text-dim text-xs font-mono">dice</span>
        <button
          onClick={onRoll}
          className="px-4 py-1 bg-cyber-green-dim hover:bg-cyber-green/20 border border-cyber-green-dim hover:border-cyber-green rounded text-sm font-mono text-cyber-green transition-all"
        >
          Roll
        </button>
      </div>

      {results.length > 0 && (
        <div ref={scrollRef} className="max-h-48 overflow-y-auto space-y-2 pr-1">
          {[...results].reverse().map((r, i) => (
            <div key={i} className="bg-cyber-surface border border-cyber-border rounded p-2 space-y-1">
              <div className="flex items-center justify-between">
                <span className="text-xs font-mono text-cyber-text-dim">{r.label}</span>
                <span className={`text-sm font-bold font-mono ${r.isCriticalGlitch ? "text-cyber-red" : r.isGlitch ? "text-cyber-yellow" : "text-cyber-green"}`}>
                  {r.hits} hit{r.hits !== 1 ? "s" : ""}
                  {r.isCriticalGlitch ? " · CRITICAL GLITCH" : r.isGlitch ? " · glitch" : ""}
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
  );
}
