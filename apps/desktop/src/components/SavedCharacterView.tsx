import { useState } from "react";
import { useCharacterStore } from "../store/characterStore";

export default function SavedCharacterView() {
  const saved = useCharacterStore((s) => s.savedCharacter);
  const reset = useCharacterStore((s) => s.reset);
  const applyEvent = useCharacterStore((s) => s.applyEvent);

  const [karmaAmount, setKarmaAmount] = useState(5);
  const [karmaReason, setKarmaReason] = useState("Run reward");
  const [nuyenAmount, setNuyenAmount] = useState(5000);
  const [nuyenReason, setNuyenReason] = useState("Run reward");
  const [applying, setApplying] = useState(false);
  const [eventError, setEventError] = useState<string | null>(null);
  const [showImprove, setShowImprove] = useState(false);

  if (!saved) return null;

  const characterId = saved.base.id;

  const handleKarmaReceived = async () => {
    if (karmaAmount <= 0) return;
    setApplying(true);
    setEventError(null);
    try {
      await applyEvent(characterId, {
        KarmaReceived: { amount: karmaAmount, reason: karmaReason, run_id: null },
      });
    } catch (err) {
      setEventError(String(err));
    } finally {
      setApplying(false);
    }
  };

  const handleSkillImprove = async (skillName: string, from: number) => {
    const to = from + 1;
    const karmaCost = to * 2;
    const available =
      saved.total_karma_earned - saved.total_karma_spent;
    if (available < karmaCost) {
      setEventError(`Not enough karma (need ${karmaCost}, have ${available})`);
      return;
    }
    setApplying(true);
    setEventError(null);
    try {
      await applyEvent(characterId, {
        SkillImproved: { skill_name: skillName, from, to, karma_cost: karmaCost },
      });
      // Also spend the karma
      await applyEvent(characterId, {
        KarmaSpent: {
          amount: karmaCost,
          description: `${skillName} ${from} → ${to}`,
        },
      });
    } catch (err) {
      setEventError(String(err));
    } finally {
      setApplying(false);
    }
  };

  const handleAttrImprove = async (attr: string, from: number) => {
    const to = from + 1;
    const karmaCost = to * 5;
    const available =
      saved.total_karma_earned - saved.total_karma_spent;
    if (available < karmaCost) {
      setEventError(`Not enough karma (need ${karmaCost}, have ${available})`);
      return;
    }
    setApplying(true);
    setEventError(null);
    try {
      await applyEvent(characterId, {
        AttributeImproved: {
          attribute: attr,
          from,
          to,
          karma_cost: karmaCost,
        },
      });
      await applyEvent(characterId, {
        KarmaSpent: {
          amount: karmaCost,
          description: `${attr} ${from} → ${to}`,
        },
      });
    } catch (err) {
      setEventError(String(err));
    } finally {
      setApplying(false);
    }
  };

  const handleNuyenReceived = async () => {
    if (nuyenAmount <= 0) return;
    setApplying(true);
    setEventError(null);
    try {
      await applyEvent(characterId, {
        NuyenReceived: { amount: nuyenAmount, reason: nuyenReason, run_id: null },
      });
    } catch (err) {
      setEventError(String(err));
    } finally {
      setApplying(false);
    }
  };

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
            ← Characters
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
          <div className="grid grid-cols-3 gap-2 mb-4">
            <StatBox
              label="Karma Earned"
              value={saved.total_karma_earned}
              accent="green"
            />
            <StatBox label="Karma Spent" value={saved.total_karma_spent} />
            <StatBox
              label="Nuyen"
              value={`¥${saved.nuyen.toLocaleString()}`}
              accent="green"
            />
          </div>

          {/* Event entry */}
          <div className="border-t border-cyber-border pt-3 space-y-3">
            <h3 className="text-xs font-mono text-cyber-text-dim">
              Apply Event
            </h3>
            {/* Karma received */}
            <div className="flex gap-2 items-end">
              <div className="flex-1">
                <label className="text-xs text-cyber-text-dim font-mono block mb-1">
                  Karma +
                </label>
                <div className="flex gap-1">
                  <input
                    type="number"
                    min={1}
                    value={karmaAmount}
                    onChange={(e) => setKarmaAmount(Math.max(1, Number(e.target.value)))}
                    className="w-16 bg-cyber-surface border border-cyber-border rounded px-2 py-1.5 text-sm text-center"
                  />
                  <input
                    type="text"
                    value={karmaReason}
                    onChange={(e) => setKarmaReason(e.target.value)}
                    placeholder="Reason"
                    className="flex-1 bg-cyber-surface border border-cyber-border rounded px-2 py-1.5 text-sm"
                  />
                </div>
              </div>
              <button
                onClick={handleKarmaReceived}
                disabled={applying}
                className="px-3 py-1.5 text-xs font-mono border border-cyber-green text-cyber-green hover:bg-cyber-green/10 rounded transition-colors disabled:opacity-40 shrink-0"
              >
                Apply
              </button>
            </div>

            {/* Nuyen received */}
            <div className="flex gap-2 items-end">
              <div className="flex-1">
                <label className="text-xs text-cyber-text-dim font-mono block mb-1">
                  Nuyen +
                </label>
                <div className="flex gap-1">
                  <input
                    type="number"
                    min={1}
                    value={nuyenAmount}
                    onChange={(e) => setNuyenAmount(Math.max(1, Number(e.target.value)))}
                    className="w-24 bg-cyber-surface border border-cyber-border rounded px-2 py-1.5 text-sm text-center"
                  />
                  <input
                    type="text"
                    value={nuyenReason}
                    onChange={(e) => setNuyenReason(e.target.value)}
                    placeholder="Reason"
                    className="flex-1 bg-cyber-surface border border-cyber-border rounded px-2 py-1.5 text-sm"
                  />
                </div>
              </div>
              <button
                onClick={handleNuyenReceived}
                disabled={applying}
                className="px-3 py-1.5 text-xs font-mono border border-cyber-green text-cyber-green hover:bg-cyber-green/10 rounded transition-colors disabled:opacity-40 shrink-0"
              >
                Apply
              </button>
            </div>

            {eventError && (
              <p className="text-cyber-red text-xs font-mono">{eventError}</p>
            )}
          </div>
        </div>

        {/* Karma improvements (P5-2) */}
        <div className="bg-cyber-card border border-cyber-border rounded-lg p-4 mt-4">
          <button
            onClick={() => setShowImprove((s) => !s)}
            className="text-lg font-semibold text-cyber-heading font-mono flex items-center gap-2 w-full text-left"
          >
            // Karma Improvements
            <span className="text-xs text-cyber-text-dim ml-auto">
              {showImprove ? "▲ collapse" : "▼ expand"}
            </span>
          </button>
          <p className="text-xs text-cyber-text-dim font-mono mt-1 mb-3">
            Available:{" "}
            <span className="text-cyber-green">
              {saved.total_karma_earned - saved.total_karma_spent}
            </span>{" "}
            karma
          </p>

          {showImprove && (
            <>
              {/* Attribute improvements */}
              <h3 className="text-xs font-mono text-cyber-text-dim mb-2">
                Attributes (cost = new rating × 5)
              </h3>
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
                  const canAfford =
                    saved.total_karma_earned - saved.total_karma_spent >= cost;
                  return (
                    <div
                      key={key}
                      className="bg-cyber-surface border border-cyber-border rounded px-2 py-1.5 flex items-center justify-between"
                    >
                      <span className="text-xs font-mono text-cyber-blue">
                        {label} {current}
                      </span>
                      <button
                        onClick={() => handleAttrImprove(key, current)}
                        disabled={applying || !canAfford}
                        title={`${cost} karma to reach ${current + 1}`}
                        className="text-xs px-2 py-0.5 border border-cyber-border hover:border-cyber-green text-cyber-text hover:text-cyber-green rounded transition-colors disabled:opacity-30"
                      >
                        +{cost}k
                      </button>
                    </div>
                  );
                })}
              </div>

              {/* Skill improvements */}
              {saved.base.skills.length > 0 && (
                <>
                  <h3 className="text-xs font-mono text-cyber-text-dim mb-2">
                    Skills (cost = new rating × 2)
                  </h3>
                  <div className="space-y-1 max-h-48 overflow-y-auto">
                    {saved.base.skills.map((skill) => {
                      const cost = (skill.rating + 1) * 2;
                      const canAfford =
                        saved.total_karma_earned - saved.total_karma_spent >=
                        cost;
                      return (
                        <div
                          key={skill.name}
                          className="bg-cyber-surface border border-cyber-border rounded px-2 py-1 flex items-center justify-between"
                        >
                          <span className="text-sm text-cyber-text">
                            {skill.name}
                          </span>
                          <div className="flex items-center gap-2">
                            <span className="text-xs font-mono text-cyber-blue">
                              {skill.rating}
                            </span>
                            <button
                              onClick={() =>
                                handleSkillImprove(skill.name, skill.rating)
                              }
                              disabled={applying || !canAfford}
                              title={`${cost} karma to reach ${skill.rating + 1}`}
                              className="text-xs px-2 py-0.5 border border-cyber-border hover:border-cyber-green text-cyber-text hover:text-cyber-green rounded transition-colors disabled:opacity-30"
                            >
                              +{cost}k
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
