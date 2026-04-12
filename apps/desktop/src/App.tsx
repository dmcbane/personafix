import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  useCharacterStore,
  type Edition,
  type MetatypeKey,
} from "./store/characterStore";
import BuilderShell from "./components/BuilderShell";

interface Campaign {
  id: string;
  name: string;
}

const EDITIONS: Edition[] = ["SR4", "SR5"];
const METATYPES: MetatypeKey[] = ["Human", "Elf", "Dwarf", "Ork", "Troll"];

function App() {
  const draft = useCharacterStore((s) => s.draft);
  const startNewCharacter = useCharacterStore((s) => s.startNewCharacter);

  const [campaign, setCampaign] = useState<Campaign | null>(null);
  const [campaignName, setCampaignName] = useState("My Campaign");
  const [charName, setCharName] = useState("Street Samurai");
  const [edition, setEdition] = useState<Edition>("SR4");
  const [metatype, setMetatype] = useState<MetatypeKey>("Human");
  const [error, setError] = useState<string | null>(null);

  // If we have a draft in progress, show the builder
  if (draft) {
    return <BuilderShell />;
  }

  const handleCreateCampaign = async () => {
    try {
      const result = await invoke<Campaign>("create_campaign", {
        name: campaignName,
      });
      setCampaign(result);
      setError(null);
    } catch (err) {
      setError(String(err));
    }
  };

  const handleStartBuilder = async () => {
    try {
      await startNewCharacter(edition, metatype, charName);
      setError(null);
    } catch (err) {
      setError(String(err));
    }
  };

  return (
    <div className="min-h-screen flex items-center justify-center p-8">
      <div className="w-full max-w-md space-y-6">
        <div className="text-center">
          <h1 className="text-4xl font-bold mb-2">personafix</h1>
          <p className="text-gray-400">Shadowrun Character Manager</p>
        </div>

        {!campaign ? (
          <div className="bg-gray-800 rounded-lg p-6 space-y-4">
            <h2 className="text-lg font-semibold">Create Campaign</h2>
            <input
              type="text"
              value={campaignName}
              onChange={(e) => setCampaignName(e.target.value)}
              placeholder="Campaign name"
              className="w-full bg-gray-700 border border-gray-600 rounded px-3 py-2 text-sm"
            />
            <button
              onClick={handleCreateCampaign}
              className="w-full px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded text-sm font-medium"
            >
              Create Campaign
            </button>
          </div>
        ) : (
          <div className="bg-gray-800 rounded-lg p-6 space-y-4">
            <div className="text-sm text-gray-400">
              Campaign: <span className="text-white">{campaign.name}</span>
            </div>
            <h2 className="text-lg font-semibold">New Character</h2>
            <input
              type="text"
              value={charName}
              onChange={(e) => setCharName(e.target.value)}
              placeholder="Character name"
              className="w-full bg-gray-700 border border-gray-600 rounded px-3 py-2 text-sm"
            />
            <div className="grid grid-cols-2 gap-3">
              <div>
                <label className="text-xs text-gray-400 block mb-1">
                  Edition
                </label>
                <select
                  value={edition}
                  onChange={(e) => setEdition(e.target.value as Edition)}
                  className="w-full bg-gray-700 border border-gray-600 rounded px-3 py-2 text-sm"
                >
                  {EDITIONS.map((e) => (
                    <option key={e} value={e}>
                      {e}
                    </option>
                  ))}
                </select>
              </div>
              <div>
                <label className="text-xs text-gray-400 block mb-1">
                  Metatype
                </label>
                <select
                  value={metatype}
                  onChange={(e) => setMetatype(e.target.value as MetatypeKey)}
                  className="w-full bg-gray-700 border border-gray-600 rounded px-3 py-2 text-sm"
                >
                  {METATYPES.map((m) => (
                    <option key={m} value={m}>
                      {m}
                    </option>
                  ))}
                </select>
              </div>
            </div>
            <button
              onClick={handleStartBuilder}
              className="w-full px-4 py-2 bg-purple-600 hover:bg-purple-700 rounded text-sm font-medium"
            >
              Start Building
            </button>
          </div>
        )}

        {error && (
          <div className="bg-red-900/50 border border-red-700 rounded p-3 text-red-300 text-sm">
            {error}
          </div>
        )}
      </div>
    </div>
  );
}

export default App;
