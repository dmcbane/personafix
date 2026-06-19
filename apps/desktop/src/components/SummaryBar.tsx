import { useCharacterStore, GRADE_MULTIPLIER } from "../store/characterStore";

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

  const totalBP = attrBP + skillBP + qualBP + contactBP;

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
