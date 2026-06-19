import {
  useCharacterStore,
  GRADE_MULTIPLIER,
  SR5_ATTR_POINTS,
  SR5_SKILL_POINTS,
  SR5_RESOURCE_NUYEN,
  SR5_MAGIC_STARTING,
  SR5_SPECIAL_ATTR_POINTS,
  ATTRIBUTE_NAMES,
} from "../store/characterStore";

export default function SummaryBar() {
  const draft = useCharacterStore((s) => s.draft);
  const limits = useCharacterStore((s) => s.racialLimits);
  const errors = useCharacterStore((s) => s.validationErrors);

  if (!draft || !limits) return null;

  const attrBP =
    (draft.attributes.body - limits.body[0]) * 10 +
    (draft.attributes.agility - limits.agility[0]) * 10 +
    (draft.attributes.reaction - limits.reaction[0]) * 10 +
    (draft.attributes.strength - limits.strength[0]) * 10 +
    (draft.attributes.willpower - limits.willpower[0]) * 10 +
    (draft.attributes.logic - limits.logic[0]) * 10 +
    (draft.attributes.intuition - limits.intuition[0]) * 10 +
    (draft.attributes.charisma - limits.charisma[0]) * 10 +
    (draft.attributes.edge - limits.edge[0]) * 10;

  const skillBP = draft.skills.reduce((sum, s) => sum + s.rating * 4, 0);

  const qualBP = draft.qualities.reduce((sum, q) => {
    return sum + (q.quality_type === "Positive" ? q.cost : -q.cost);
  }, 0);

  const contactBP = draft.contacts.reduce(
    (sum, c) => sum + c.connection + c.loyalty,
    0,
  );

  // SR4: nuyen spent converts to BP at ¥5000/BP (mirrors sr4_bp::bp_cost_resources)
  const resourceBP = Math.floor(draft.nuyen_spent / 5000);

  const totalBP = attrBP + skillBP + qualBP + contactBP + resourceBP;

  // SR5 budget computations
  const isSR5 = draft.edition === "SR5";
  const sel = draft.priority_selection;
  const sr5AttrSpent = isSR5
    ? ATTRIBUTE_NAMES.filter((a) => a !== "edge").reduce(
        (sum, attr) => sum + (draft.attributes[attr] - (limits?.[attr][0] ?? 0)),
        0,
      )
    : 0;
  const sr5AttrAlloc = isSR5 && sel ? SR5_ATTR_POINTS[sel.attributes] : 0;
  const sr5SkillSpent = isSR5
    ? draft.skills.reduce((sum, s) => sum + s.rating, 0)
    : 0;
  const [sr5SkillAlloc] = isSR5 && sel ? SR5_SKILL_POINTS[sel.skills] : [0, 0];
  const sr5NuyenBudget = isSR5 && sel ? SR5_RESOURCE_NUYEN[sel.resources] : 0;
  const sr5QualKarma = isSR5
    ? draft.qualities.reduce(
        (sum, q) =>
          sum + (q.quality_type === "Positive" ? q.cost : -q.cost),
        0,
      )
    : 0;

  // SR5 special attribute pool: magic above starting + edge above racial min
  const sr5MagicStarting = isSR5 && sel ? SR5_MAGIC_STARTING[sel.magic_or_resonance] : 0;
  const sr5SpecialPool = isSR5 && sel ? SR5_SPECIAL_ATTR_POINTS[sel.metatype] : 0;
  const sr5MagicSpecial = isSR5 && draft.attributes.magic != null && draft.magic_tradition
    ? Math.max(0, draft.attributes.magic - sr5MagicStarting)
    : 0;
  const sr5EdgeSpecial = isSR5 ? Math.max(0, draft.attributes.edge - limits.edge[0]) : 0;
  const sr5SpecialSpent = sr5MagicSpecial + sr5EdgeSpecial;

  const augEssenceUsed = draft.augmentations.reduce((sum, a) => {
    const mult = GRADE_MULTIPLIER[a.grade] ?? 100;
    return sum + Math.floor((a.essence_cost * mult) / 100);
  }, 0);
  const essenceRemaining = 600 - augEssenceUsed;

  const realErrors = errors.filter((e) => e.severity === "Error");
  const warnings = errors.filter((e) => e.severity === "Warning");

  return (
    <div className="bg-cyber-surface border-t border-cyber-border px-6 py-3">
      <div className="flex items-center gap-6 text-sm font-mono">
        {isSR5 && sel && (
          <>
            <div>
              <span className="text-cyber-text-dim">Attr: </span>
              <span
                className={`font-bold ${sr5AttrSpent > sr5AttrAlloc ? "text-cyber-red" : "text-cyber-green"}`}
              >
                {sr5AttrSpent}
              </span>
              <span className="text-cyber-text-dim">/{sr5AttrAlloc}</span>
            </div>
            <div className="text-cyber-border">|</div>
            <div>
              <span className="text-cyber-text-dim">Skills: </span>
              <span
                className={`font-bold ${sr5SkillSpent > sr5SkillAlloc ? "text-cyber-red" : "text-cyber-green"}`}
              >
                {sr5SkillSpent}
              </span>
              <span className="text-cyber-text-dim">/{sr5SkillAlloc}</span>
            </div>
            <div className="text-cyber-border">|</div>
            <div className="text-cyber-text-dim">
              Qual:{" "}
              <span className={Math.abs(sr5QualKarma) > 25 ? "text-cyber-red" : "text-cyber-text"}>
                {sr5QualKarma > 0 ? "+" : ""}{sr5QualKarma}k
              </span>
            </div>
            <div className="text-cyber-border">|</div>
            <div className="text-cyber-text-dim">
              ¥:{" "}
              <span className={draft.nuyen_spent > sr5NuyenBudget ? "text-cyber-red" : "text-cyber-text"}>
                {draft.nuyen_spent.toLocaleString()}
              </span>
              <span className="text-cyber-text-dim">/{sr5NuyenBudget.toLocaleString()}</span>
            </div>
            <div className="text-cyber-border">|</div>
            <div>
              <span className="text-cyber-text-dim">SAP: </span>
              <span
                className={`font-bold ${sr5SpecialSpent > sr5SpecialPool ? "text-cyber-red" : "text-cyber-green"}`}
              >
                {sr5SpecialSpent}
              </span>
              <span className="text-cyber-text-dim">/{sr5SpecialPool}</span>
            </div>
            <div className="text-cyber-border">|</div>
          </>
        )}
        {draft.edition === "SR4" && (
          <>
            <div>
              <span className="text-cyber-text-dim">BP: </span>
              <span
                className={`font-bold ${totalBP > 400 ? "text-cyber-red" : "text-cyber-green"}`}
              >
                {totalBP}
              </span>
              <span className="text-cyber-text-dim"> / 400</span>
            </div>
            <div className="text-cyber-border">|</div>
            <div className="text-cyber-text-dim">
              Attr: <span className="text-cyber-text">{attrBP}</span>
            </div>
            <div className="text-cyber-text-dim">
              Skills: <span className="text-cyber-text">{skillBP}</span>
            </div>
            <div className="text-cyber-text-dim">
              Qual: <span className="text-cyber-text">{qualBP}</span>
            </div>
            {contactBP > 0 && (
              <div className="text-cyber-text-dim">
                Contacts: <span className="text-cyber-text">{contactBP}</span>
              </div>
            )}
            {resourceBP > 0 && (
              <div className="text-cyber-text-dim">
                Res: <span className="text-cyber-text">{resourceBP}</span>
              </div>
            )}
          </>
        )}
        <div className="text-cyber-text-dim">
          Essence:{" "}
          <span
            className={
              essenceRemaining <= 0 ? "text-cyber-red" : "text-cyber-blue"
            }
          >
            {(essenceRemaining / 100).toFixed(2)}
          </span>
        </div>
        <div className="flex-1" />
        {warnings.length > 0 && (
          <div className="text-cyber-yellow text-xs">
            {warnings.length} warning{warnings.length > 1 ? "s" : ""}
          </div>
        )}
        {realErrors.length > 0 && (
          <div className="text-cyber-red text-xs">
            {realErrors.length} error{realErrors.length > 1 ? "s" : ""}
          </div>
        )}
      </div>
      {(realErrors.length > 0 || warnings.length > 0) && (
        <div className="mt-2 space-y-0.5">
          {realErrors.map((e, i) => (
            <p key={`e-${i}`} className="text-cyber-red text-xs font-mono">
              [error] [{e.field}] {e.message}
            </p>
          ))}
          {warnings.map((e, i) => (
            <p key={`w-${i}`} className="text-cyber-yellow text-xs font-mono">
              [warn] [{e.field}] {e.message}
            </p>
          ))}
        </div>
      )}
    </div>
  );
}
