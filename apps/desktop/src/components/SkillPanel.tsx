import { useState } from "react";
import { useCharacterStore, type Skill } from "../store/characterStore";

const COMMON_SKILLS = [
  { name: "Pistols", attr: "AGI" },
  { name: "Automatics", attr: "AGI" },
  { name: "Longarms", attr: "AGI" },
  { name: "Blades", attr: "AGI" },
  { name: "Clubs", attr: "AGI" },
  { name: "Unarmed Combat", attr: "AGI" },
  { name: "Dodge", attr: "REA" },
  { name: "Sneaking", attr: "AGI" },
  { name: "Perception", attr: "INT" },
  { name: "Intimidation", attr: "CHA" },
  { name: "Negotiation", attr: "CHA" },
  { name: "Con", attr: "CHA" },
  { name: "Etiquette", attr: "CHA" },
  { name: "First Aid", attr: "LOG" },
  { name: "Electronics", attr: "LOG" },
  { name: "Hacking", attr: "LOG" },
  { name: "Pilot Ground Craft", attr: "REA" },
  { name: "Running", attr: "STR" },
  { name: "Swimming", attr: "BOD" },
  { name: "Gymnastics", attr: "AGI" },
];

export default function SkillPanel() {
  const draft = useCharacterStore((s) => s.draft);
  const addSkill = useCharacterStore((s) => s.addSkill);
  const removeSkill = useCharacterStore((s) => s.removeSkill);
  const updateSkillRating = useCharacterStore((s) => s.updateSkillRating);
  const validate = useCharacterStore((s) => s.validate);
  const [selectedSkill, setSelectedSkill] = useState("");

  if (!draft) return null;

  const maxRating = 6;
  const bpCost = draft.skills.reduce((total, s) => total + s.rating * 4, 0);

  const availableSkills = COMMON_SKILLS.filter(
    (cs) => !draft.skills.some((s) => s.name === cs.name),
  );

  const handleAddSkill = () => {
    const skillDef = COMMON_SKILLS.find((s) => s.name === selectedSkill);
    if (!skillDef) return;

    const skill: Skill = {
      id: skillDef.name.toLowerCase().replace(/ /g, "_"),
      name: skillDef.name,
      linked_attribute: skillDef.attr,
      group: null,
      rating: 1,
      specializations: [],
    };

    addSkill(skill);
    setSelectedSkill("");
    validate();
  };

  const handleRatingChange = (skillId: string, rating: number) => {
    updateSkillRating(skillId, rating);
    validate();
  };

  return (
    <div>
      <h2 className="text-xl font-semibold mb-4 text-cyber-heading">
        // Skills
      </h2>
      {draft.edition === "SR4" && (
        <p className="text-sm text-cyber-text-dim mb-4 font-mono">
          BP spent: <span className="text-cyber-green">{bpCost}</span>
        </p>
      )}

      {/* Add skill */}
      <div className="flex gap-2 mb-4">
        <select
          value={selectedSkill}
          onChange={(e) => setSelectedSkill(e.target.value)}
          className="bg-cyber-card border border-cyber-border rounded px-3 py-1.5 text-sm flex-1 text-cyber-text"
        >
          <option value="">Add a skill...</option>
          {availableSkills.map((s) => (
            <option key={s.name} value={s.name}>
              {s.name} ({s.attr})
            </option>
          ))}
        </select>
        <button
          onClick={handleAddSkill}
          disabled={!selectedSkill}
          className="px-4 py-1.5 bg-cyber-green-dim hover:bg-cyber-green/20 border border-cyber-green-dim hover:border-cyber-green rounded text-sm disabled:opacity-50 text-cyber-green font-mono transition-all"
        >
          Add
        </button>
      </div>

      {/* Skill list */}
      {draft.skills.length === 0 ? (
        <p className="text-cyber-text-dim text-sm font-mono">
          No skills added yet.
        </p>
      ) : (
        <div className="space-y-2">
          {draft.skills.map((skill) => (
            <div
              key={skill.id}
              className="flex items-center gap-3 bg-cyber-card border border-cyber-border rounded px-3 py-2"
            >
              <span className="flex-1 text-sm">
                {skill.name}{" "}
                <span className="text-cyber-text-dim font-mono">
                  ({skill.linked_attribute})
                </span>
              </span>
              <button
                onClick={() =>
                  handleRatingChange(skill.id, Math.max(1, skill.rating - 1))
                }
                disabled={skill.rating <= 1}
                className="w-7 h-7 rounded bg-cyber-surface border border-cyber-border hover:border-cyber-green-dim disabled:opacity-30 text-sm text-cyber-text transition-colors"
              >
                -
              </button>
              <span className="font-mono text-lg w-4 text-center text-cyber-heading">
                {skill.rating}
              </span>
              <button
                onClick={() =>
                  handleRatingChange(
                    skill.id,
                    Math.min(maxRating, skill.rating + 1),
                  )
                }
                disabled={skill.rating >= maxRating}
                className="w-7 h-7 rounded bg-cyber-surface border border-cyber-border hover:border-cyber-green-dim disabled:opacity-30 text-sm text-cyber-text transition-colors"
              >
                +
              </button>
              <button
                onClick={() => {
                  removeSkill(skill.id);
                  validate();
                }}
                className="text-cyber-red hover:text-cyber-red/80 text-sm ml-2 transition-colors"
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
