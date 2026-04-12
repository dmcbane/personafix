import { useCharacterStore } from "../store/characterStore";

export default function SavedCharacterView() {
  const saved = useCharacterStore((s) => s.savedCharacter);
  const reset = useCharacterStore((s) => s.reset);

  if (!saved) return null;

  const attrs = saved.computed_attributes;

  return (
    <div className="min-h-screen p-8">
      <div className="max-w-2xl mx-auto">
        <div className="flex items-center justify-between mb-6">
          <div>
            <h1 className="text-3xl font-bold text-cyber-heading">
              {saved.base.name}
            </h1>
            <p className="text-cyber-text-dim font-mono">
              {saved.base.edition} {saved.base.metatype} // Character Sheet
            </p>
          </div>
          <button
            onClick={reset}
            className="px-4 py-2 bg-cyber-card border border-cyber-border hover:border-cyber-border-bright rounded text-sm text-cyber-text transition-colors"
          >
            New Character
          </button>
        </div>

        {/* Attributes */}
        <div className="bg-cyber-card border border-cyber-border rounded-lg p-4 mb-4">
          <h2 className="text-lg font-semibold mb-3 text-cyber-heading font-mono">
            // Attributes
          </h2>
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
              <div
                key={label}
                className="bg-cyber-surface border border-cyber-border rounded px-3 py-2 text-center"
              >
                <div className="text-cyber-blue text-xs font-mono">
                  {label}
                </div>
                <div className="text-xl font-bold text-cyber-heading">
                  {value}
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Derived Stats */}
        <div className="bg-cyber-card border border-cyber-border rounded-lg p-4 mb-4">
          <h2 className="text-lg font-semibold mb-3 text-cyber-heading font-mono">
            // Derived Stats
          </h2>
          <div className="grid grid-cols-2 sm:grid-cols-4 gap-2">
            <StatBox
              label="Physical CM"
              value={saved.physical_condition_monitor}
            />
            <StatBox label="Stun CM" value={saved.stun_condition_monitor} />
            <StatBox
              label="Initiative"
              value={`${saved.initiative} + ${saved.initiative_dice}d6`}
            />
            <StatBox
              label="Essence"
              value={(attrs.essence / 100).toFixed(2)}
              accent="blue"
            />
            {attrs.magic !== null && (
              <StatBox label="Magic" value={attrs.magic} accent="purple" />
            )}
            {attrs.resonance !== null && (
              <StatBox
                label="Resonance"
                value={attrs.resonance}
                accent="blue"
              />
            )}
          </div>
        </div>

        {/* Career */}
        <div className="bg-cyber-card border border-cyber-border rounded-lg p-4">
          <h2 className="text-lg font-semibold mb-3 text-cyber-heading font-mono">
            // Career
          </h2>
          <div className="grid grid-cols-3 gap-2">
            <StatBox
              label="Karma Earned"
              value={saved.total_karma_earned}
              accent="green"
            />
            <StatBox label="Karma Spent" value={saved.total_karma_spent} />
            <StatBox
              label="Nuyen"
              value={`${saved.nuyen.toLocaleString()}`}
              accent="green"
            />
          </div>
        </div>
      </div>
    </div>
  );
}

function StatBox({
  label,
  value,
  accent,
}: {
  label: string;
  value: string | number;
  accent?: "green" | "blue" | "purple";
}) {
  const valueColor =
    accent === "green"
      ? "text-cyber-green"
      : accent === "blue"
        ? "text-cyber-blue"
        : accent === "purple"
          ? "text-cyber-purple"
          : "text-cyber-heading";

  return (
    <div className="bg-cyber-surface border border-cyber-border rounded px-3 py-2 text-center">
      <div className="text-cyber-text-dim text-xs font-mono">{label}</div>
      <div className={`text-lg font-bold font-mono ${valueColor}`}>
        {value}
      </div>
    </div>
  );
}
