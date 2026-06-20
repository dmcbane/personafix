import { useState } from "react";
import {
  useCharacterStore,
  SR5_SKILL_POINTS,
  KNOWLEDGE_SKILL_CATEGORIES,
  type Skill,
  type KnowledgeSkill,
} from "../store/characterStore";
import { useGameDataStore } from "../store/gameDataStore";

const FALLBACK_SKILLS = [
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
  const addSpecialization = useCharacterStore((s) => s.addSpecialization);
  const removeSpecialization = useCharacterStore((s) => s.removeSpecialization);
  const addKnowledgeSkill = useCharacterStore((s) => s.addKnowledgeSkill);
  const removeKnowledgeSkill = useCharacterStore((s) => s.removeKnowledgeSkill);
  const validate = useCharacterStore((s) => s.validate);
  const gameSkills = useGameDataStore((s) => s.skills);
  const gameDataLoaded = useGameDataStore((s) => s.loaded);
  const [selectedSkill, setSelectedSkill] = useState("");
  const [search, setSearch] = useState("");
  const [attrFilter, setAttrFilter] = useState<string>("all");
  const [specInputs, setSpecInputs] = useState<Record<string, string>>({});
  const [knowledgeName, setKnowledgeName] = useState("");
  const [knowledgeCategory, setKnowledgeCategory] = useState<string>(KNOWLEDGE_SKILL_CATEGORIES[0]);
  const [knowledgeRating, setKnowledgeRating] = useState(1);

  if (!draft) return null;

  const maxRating = 6;
  const bpCost = draft.skills.reduce((total, s) => total + s.rating * 4, 0);

  const isSR5 = draft.edition === "SR5";
  const sr5SkillSpent = draft.skills.reduce((sum, s) => sum + s.rating, 0);
  const [sr5SkillAlloc, sr5GroupAlloc] =
    isSR5 && draft.priority_selection
      ? SR5_SKILL_POINTS[draft.priority_selection.skills]
      : [0, 0];

  // Use game data if loaded, otherwise fallback
  const skillSource = gameDataLoaded
    ? gameSkills.map((gs) => ({ name: gs.name, attr: gs.linked_attribute, id: gs.id }))
    : FALLBACK_SKILLS.map((s) => ({ ...s, id: s.name.toLowerCase().replace(/ /g, "_") }));

  // Collect unique attributes for filter buttons
  const uniqueAttrs = [...new Set(skillSource.map((s) => s.attr))].sort();

  const availableSkills = skillSource
    .filter((cs) => !draft.skills.some((s) => s.name === cs.name))
    .filter((cs) => attrFilter === "all" || cs.attr === attrFilter)
    .filter((cs) =>
      search === "" || cs.name.toLowerCase().includes(search.toLowerCase()),
    );

  const handleAddSkill = () => {
    const skillDef = skillSource.find((s) => s.name === selectedSkill);
    if (!skillDef) return;

    const skill: Skill = {
      id: skillDef.id || skillDef.name.toLowerCase().replace(/ /g, "_"),
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

  const handleAddKnowledgeSkill = () => {
    if (!knowledgeName.trim()) return;
    const skill: KnowledgeSkill = {
      name: knowledgeName.trim(),
      category: knowledgeCategory,
      rating: knowledgeRating,
    };
    addKnowledgeSkill(skill);
    setKnowledgeName("");
    setKnowledgeRating(1);
  };

  return (
    <div>
      <h2 className="text-xl font-semibold mb-4 text-cyber-heading">
        // Skills
      </h2>
      <div className="flex gap-4 text-sm text-cyber-text-dim mb-4 font-mono">
        {draft.edition === "SR4" && (
          <span>
            BP spent: <span className="text-cyber-green">{bpCost}</span>
          </span>
        )}
        {isSR5 && (
          <span>
            Skills:{" "}
            <span
              className={
                sr5SkillSpent > sr5SkillAlloc ? "text-cyber-red" : "text-cyber-green"
              }
            >
              {sr5SkillSpent}
            </span>
            {" / "}
            {sr5SkillAlloc}
            {sr5GroupAlloc > 0 && (
              <span className="ml-2 text-cyber-text-dim">
                | Groups: 0/{sr5GroupAlloc}
              </span>
            )}
          </span>
        )}
        <span>
          Available:{" "}
          <span className="text-cyber-text">
            {gameDataLoaded ? gameSkills.length : FALLBACK_SKILLS.length}
          </span>
          {gameDataLoaded && (
            <span className="text-cyber-green-dim ml-1">(game data)</span>
          )}
        </span>
      </div>

      {/* Attribute filter */}
      <div className="flex gap-1.5 mb-3 flex-wrap">
        <button
          onClick={() => setAttrFilter("all")}
          className={`px-2.5 py-1 rounded text-xs font-mono transition-all ${
            attrFilter === "all"
              ? "bg-cyber-green-dim border border-cyber-green text-cyber-green shadow-glow"
              : "bg-cyber-card border border-cyber-border text-cyber-text-dim hover:border-cyber-border-bright"
          }`}
        >
          All
        </button>
        {uniqueAttrs.map((attr) => (
          <button
            key={attr}
            onClick={() => setAttrFilter(attr === attrFilter ? "all" : attr)}
            className={`px-2.5 py-1 rounded text-xs font-mono transition-all ${
              attrFilter === attr
                ? "bg-cyber-blue/20 border border-cyber-blue text-cyber-blue"
                : "bg-cyber-card border border-cyber-border text-cyber-text-dim hover:border-cyber-border-bright"
            }`}
          >
            {attr}
          </button>
        ))}
        {attrFilter !== "all" && (
          <span className="text-cyber-text-dim text-xs font-mono self-center ml-1">
            {availableSkills.length} skills
          </span>
        )}
      </div>

      {/* Search + Add */}
      <div className="flex gap-2 mb-4">
        <input
          type="text"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          placeholder="Search skills..."
          className="bg-cyber-card border border-cyber-border rounded px-3 py-1.5 text-sm w-40"
        />
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
              className="bg-cyber-card border border-cyber-border rounded px-3 py-2"
            >
              <div className="flex items-center gap-3">
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
              {/* Specialization chips */}
              {skill.specializations.length > 0 && (
                <div className="flex flex-wrap gap-1 mt-1.5 ml-1">
                  {skill.specializations.map((sp) => (
                    <span
                      key={sp.name}
                      className="inline-flex items-center gap-1 text-xs font-mono bg-cyber-surface border border-cyber-border rounded px-2 py-0.5 text-cyber-blue"
                    >
                      {sp.name} (+{sp.bonus})
                      <button
                        onClick={() => removeSpecialization(skill.id, sp.name)}
                        className="text-cyber-red/70 hover:text-cyber-red ml-0.5"
                        title="Remove"
                      >
                        ×
                      </button>
                    </span>
                  ))}
                </div>
              )}
              {/* Add specialization */}
              {skill.specializations.length < 1 && (
                <div className="flex gap-1 mt-1.5 ml-1">
                  <input
                    type="text"
                    placeholder="Add specialization…"
                    value={specInputs[skill.id] ?? ""}
                    onChange={(e) => setSpecInputs((prev) => ({ ...prev, [skill.id]: e.target.value }))}
                    onKeyDown={(e) => {
                      if (e.key === "Enter" && specInputs[skill.id]?.trim()) {
                        addSpecialization(skill.id, specInputs[skill.id]);
                        setSpecInputs((prev) => ({ ...prev, [skill.id]: "" }));
                      }
                    }}
                    className="flex-1 text-xs font-mono bg-cyber-surface border border-cyber-border rounded px-2 py-0.5 text-cyber-text placeholder:text-cyber-text-dim/50 focus:border-cyber-green-dim focus:outline-none"
                  />
                  <button
                    onClick={() => {
                      if (specInputs[skill.id]?.trim()) {
                        addSpecialization(skill.id, specInputs[skill.id]);
                        setSpecInputs((prev) => ({ ...prev, [skill.id]: "" }));
                      }
                    }}
                    className="text-xs px-2 py-0.5 border border-cyber-border hover:border-cyber-green-dim text-cyber-text-dim hover:text-cyber-text rounded transition-colors"
                  >
                    +Spec
                  </button>
                </div>
              )}
            </div>
          ))}
        </div>
      )}

      {/* Knowledge/Language Skills */}
      <div className="mt-8">
        <h3 className="text-lg font-semibold mb-3 text-cyber-heading">
          // Knowledge &amp; Language Skills
        </h3>
        <div className="flex gap-2 mb-3 flex-wrap">
          <input
            type="text"
            value={knowledgeName}
            onChange={(e) => setKnowledgeName(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && handleAddKnowledgeSkill()}
            placeholder="Skill name…"
            className="bg-cyber-surface border border-cyber-border rounded px-2 py-1 text-sm text-cyber-text placeholder-cyber-text-dim focus:border-cyber-green outline-none flex-1 min-w-[140px]"
          />
          <select
            value={knowledgeCategory}
            onChange={(e) => setKnowledgeCategory(e.target.value)}
            className="bg-cyber-surface border border-cyber-border rounded px-2 py-1 text-sm text-cyber-text focus:border-cyber-green outline-none"
          >
            {KNOWLEDGE_SKILL_CATEGORIES.map((cat) => (
              <option key={cat} value={cat}>{cat}</option>
            ))}
          </select>
          <div className="flex items-center gap-1">
            <button
              onClick={() => setKnowledgeRating(Math.max(1, knowledgeRating - 1))}
              disabled={knowledgeRating <= 1}
              className="w-7 h-7 rounded bg-cyber-surface border border-cyber-border hover:border-cyber-green-dim disabled:opacity-30 text-sm text-cyber-text transition-colors"
            >-</button>
            <span className="font-mono text-cyber-heading w-4 text-center">{knowledgeRating}</span>
            <button
              onClick={() => setKnowledgeRating(Math.min(12, knowledgeRating + 1))}
              disabled={knowledgeRating >= 12}
              className="w-7 h-7 rounded bg-cyber-surface border border-cyber-border hover:border-cyber-green-dim disabled:opacity-30 text-sm text-cyber-text transition-colors"
            >+</button>
          </div>
          <button
            onClick={handleAddKnowledgeSkill}
            disabled={!knowledgeName.trim()}
            className="px-3 py-1 rounded bg-cyber-green/10 border border-cyber-green text-cyber-green text-sm hover:bg-cyber-green/20 disabled:opacity-40 transition-colors"
          >
            + Add
          </button>
        </div>

        {draft.knowledge_skills.length === 0 ? (
          <p className="text-cyber-text-dim text-sm italic">No knowledge skills added.</p>
        ) : (
          <div className="space-y-1.5">
            {draft.knowledge_skills.map((k) => (
              <div
                key={k.name}
                className="flex items-center justify-between bg-cyber-surface border border-cyber-border rounded px-3 py-1.5"
              >
                <span className="text-cyber-text text-sm">
                  {k.name}{" "}
                  <span className="text-cyber-text-dim font-mono text-xs">({k.category})</span>
                </span>
                <div className="flex items-center gap-2">
                  <span className="font-mono text-cyber-heading">{k.rating}</span>
                  <button
                    onClick={() => removeKnowledgeSkill(k.name)}
                    className="text-cyber-red hover:text-cyber-red/80 text-sm transition-colors"
                  >X</button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
