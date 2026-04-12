import { useCharacterStore } from "../store/characterStore";

export default function SummaryBar() {
  const draft = useCharacterStore((s) => s.draft);
  const limits = useCharacterStore((s) => s.racialLimits);
  const errors = useCharacterStore((s) => s.validationErrors);

  if (!draft || !limits) return null;

  // Calculate BP totals (SR4)
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

  const totalBP = attrBP + skillBP + qualBP;

  const realErrors = errors.filter((e) => e.severity === "Error");

  return (
    <div className="bg-gray-800 border-t border-gray-700 px-6 py-3">
      <div className="flex items-center gap-6 text-sm">
        {draft.edition === "SR4" && (
          <>
            <div>
              <span className="text-gray-400">BP: </span>
              <span
                className={`font-mono font-bold ${totalBP > 400 ? "text-red-400" : "text-green-400"}`}
              >
                {totalBP}
              </span>
              <span className="text-gray-500"> / 400</span>
            </div>
            <div className="text-gray-600">|</div>
            <div className="text-gray-400">
              Attr: <span className="text-white font-mono">{attrBP}</span>
            </div>
            <div className="text-gray-400">
              Skills: <span className="text-white font-mono">{skillBP}</span>
            </div>
            <div className="text-gray-400">
              Qual: <span className="text-white font-mono">{qualBP}</span>
            </div>
          </>
        )}
        <div className="text-gray-400">
          Essence:{" "}
          <span className="text-cyan-400 font-mono">
            {(draft.attributes.essence / 100).toFixed(2)}
          </span>
        </div>
        <div className="flex-1" />
        {realErrors.length > 0 && (
          <div className="text-red-400 text-xs">
            {realErrors.length} error{realErrors.length > 1 ? "s" : ""}
          </div>
        )}
      </div>
      {realErrors.length > 0 && (
        <div className="mt-2 space-y-0.5">
          {realErrors.map((e, i) => (
            <p key={i} className="text-red-400 text-xs">
              {e.message}
            </p>
          ))}
        </div>
      )}
    </div>
  );
}
