import { useState } from "react";
import { useCharacterStore } from "../store/characterStore";
import AttributePanel from "./AttributePanel";
import SkillPanel from "./SkillPanel";
import QualityPanel from "./QualityPanel";
import SummaryBar from "./SummaryBar";

type Tab = "attributes" | "skills" | "qualities";

const TABS: { key: Tab; label: string }[] = [
  { key: "attributes", label: "Attributes" },
  { key: "skills", label: "Skills" },
  { key: "qualities", label: "Qualities" },
];

export default function BuilderShell() {
  const [activeTab, setActiveTab] = useState<Tab>("attributes");
  const draft = useCharacterStore((s) => s.draft);

  if (!draft) return null;

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="px-6 py-4 border-b border-gray-700">
        <h1 className="text-2xl font-bold">{draft.name}</h1>
        <p className="text-gray-400 text-sm">
          {draft.edition} {draft.metatype} — Character Builder
        </p>
      </div>

      {/* Tab bar */}
      <div className="flex border-b border-gray-700">
        {TABS.map((tab) => (
          <button
            key={tab.key}
            onClick={() => setActiveTab(tab.key)}
            className={`px-6 py-2.5 text-sm font-medium transition-colors ${
              activeTab === tab.key
                ? "text-white border-b-2 border-blue-500"
                : "text-gray-400 hover:text-gray-200"
            }`}
          >
            {tab.label}
          </button>
        ))}
      </div>

      {/* Panel content */}
      <div className="flex-1 overflow-y-auto p-6">
        {activeTab === "attributes" && <AttributePanel />}
        {activeTab === "skills" && <SkillPanel />}
        {activeTab === "qualities" && <QualityPanel />}
      </div>

      {/* Summary bar — always visible */}
      <SummaryBar />
    </div>
  );
}
