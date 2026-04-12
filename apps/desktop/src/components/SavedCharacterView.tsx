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
            <h1 className="text-3xl font-bold">{saved.base.name}</h1>
            <p className="text-gray-400">
              {saved.base.edition} {saved.base.metatype} — Character Sheet
            </p>
          </div>
          <button
            onClick={reset}
            className="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded text-sm"
          >
            New Character
          </button>
        </div>

        {/* Attributes */}
        <div className="bg-gray-800 rounded-lg p-4 mb-4">
          <h2 className="text-lg font-semibold mb-3">Attributes</h2>
          <div className="grid grid-cols-3 sm:grid-cols-5 gap-2">
            {([
              ["BOD", attrs.body],
              ["AGI", attrs.agility],
              ["REA", attrs.reaction],
              ["STR", attrs.strength],
              ["WIL", attrs.willpower],
              ["LOG", attrs.logic],
              ["INT", attrs.intuition],
              ["CHA", attrs.charisma],
              ["EDG", attrs.edge],
            ] as [string, number][]).map(([label, value]) => (
              <div
                key={label}
                className="bg-gray-700 rounded px-3 py-2 text-center"
              >
                <div className="text-gray-400 text-xs">{label}</div>
                <div className="text-xl font-bold">{value}</div>
              </div>
            ))}
          </div>
        </div>

        {/* Derived Stats */}
        <div className="bg-gray-800 rounded-lg p-4 mb-4">
          <h2 className="text-lg font-semibold mb-3">Derived Stats</h2>
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
            />
            {attrs.magic !== null && (
              <StatBox label="Magic" value={attrs.magic} />
            )}
            {attrs.resonance !== null && (
              <StatBox label="Resonance" value={attrs.resonance} />
            )}
          </div>
        </div>

        {/* Karma & Nuyen */}
        <div className="bg-gray-800 rounded-lg p-4">
          <h2 className="text-lg font-semibold mb-3">Career</h2>
          <div className="grid grid-cols-3 gap-2">
            <StatBox label="Karma Earned" value={saved.total_karma_earned} />
            <StatBox label="Karma Spent" value={saved.total_karma_spent} />
            <StatBox
              label="Nuyen"
              value={`${saved.nuyen.toLocaleString()}¥`}
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
}: {
  label: string;
  value: string | number;
}) {
  return (
    <div className="bg-gray-700 rounded px-3 py-2 text-center">
      <div className="text-gray-400 text-xs">{label}</div>
      <div className="text-lg font-bold">{value}</div>
    </div>
  );
}
