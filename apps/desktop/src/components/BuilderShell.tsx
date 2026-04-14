import { useState } from "react";
import { useCharacterStore } from "../store/characterStore";
import AttributePanel from "./AttributePanel";
import SkillPanel from "./SkillPanel";
import QualityPanel from "./QualityPanel";
import PriorityPanel from "./PriorityPanel";
import SummaryBar from "./SummaryBar";

type Tab = "priority" | "attributes" | "skills" | "qualities";

export default function BuilderShell({
  campaignId,
}: {
  campaignId: string;
}) {
  const draft = useCharacterStore((s) => s.draft);
  const validationErrors = useCharacterStore((s) => s.validationErrors);
  const saveCharacter = useCharacterStore((s) => s.saveCharacter);
  const validate = useCharacterStore((s) => s.validate);
  const [saving, setSaving] = useState(false);
  const [saveError, setSaveError] = useState<string | null>(null);

  console.log("[personafix] BuilderShell render, draft:", draft?.name, "campaignId:", campaignId);

  if (!draft) return null;

  const isSR5 = draft.edition === "SR5";

  const tabs: { key: Tab; label: string }[] = [
    ...(isSR5 ? [{ key: "priority" as Tab, label: "Priority" }] : []),
    { key: "attributes", label: "Attributes" },
    { key: "skills", label: "Skills" },
    { key: "qualities", label: "Qualities" },
  ];

  const [activeTab, setActiveTab] = useState<Tab>(
    isSR5 ? "priority" : "attributes",
  );

  const handleSave = async () => {
    setSaving(true);
    setSaveError(null);
    await validate();

    const errors = useCharacterStore.getState().validationErrors;
    const realErrors = errors.filter((e) => e.severity === "Error");
    if (realErrors.length > 0) {
      setSaveError(
        `Cannot save: ${realErrors.length} validation error${realErrors.length > 1 ? "s" : ""}`,
      );
      setSaving(false);
      return;
    }

    try {
      await saveCharacter(campaignId);
    } catch (err) {
      setSaveError(String(err));
    }
    setSaving(false);
  };

  const realErrors = validationErrors.filter((e) => e.severity === "Error");

  return (
    <div className="flex flex-col h-screen">
      {/* Header */}
      <div className="px-6 py-4 border-b border-cyber-border flex items-center justify-between bg-cyber-surface">
        <div>
          <h1 className="text-2xl font-bold text-cyber-heading">
            {draft.name}
          </h1>
          <p className="text-cyber-text-dim text-sm font-mono">
            {draft.edition} {draft.metatype} // Character Builder
          </p>
        </div>
        <div className="flex items-center gap-3">
          {saveError && (
            <span className="text-cyber-red text-sm">{saveError}</span>
          )}
          <button
            onClick={handleSave}
            disabled={saving}
            className="px-5 py-2 bg-cyber-green-dim hover:bg-cyber-green/20 border border-cyber-green-dim hover:border-cyber-green disabled:opacity-50 rounded text-sm font-medium text-cyber-green transition-all shadow-glow"
          >
            {saving ? "Saving..." : "Save Character"}
          </button>
        </div>
      </div>

      {/* Tab bar */}
      <div className="flex border-b border-cyber-border bg-cyber-surface">
        {tabs.map((tab) => (
          <button
            key={tab.key}
            onClick={() => setActiveTab(tab.key)}
            className={`px-6 py-2.5 text-sm font-medium font-mono transition-colors ${
              activeTab === tab.key
                ? "text-cyber-green border-b-2 border-cyber-green"
                : "text-cyber-text-dim hover:text-cyber-text"
            }`}
          >
            {tab.label}
            {tab.key !== "priority" &&
              realErrors.some((e) =>
                e.field.startsWith(
                  tab.key === "attributes"
                    ? "attributes"
                    : tab.key === "skills"
                      ? "skills"
                      : "qualities",
                ),
              ) && (
                <span className="ml-1 text-cyber-red text-xs">!</span>
              )}
          </button>
        ))}
      </div>

      {/* Panel content */}
      <div className="flex-1 overflow-y-auto p-6">
        {activeTab === "priority" && <PriorityPanel />}
        {activeTab === "attributes" && <AttributePanel />}
        {activeTab === "skills" && <SkillPanel />}
        {activeTab === "qualities" && <QualityPanel />}
      </div>

      {/* Summary bar */}
      <SummaryBar />
    </div>
  );
}
