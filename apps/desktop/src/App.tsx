import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Campaign {
  id: string;
  name: string;
}

interface CharacterSummary {
  id: string;
  name: string;
  edition: string;
  metatype: string;
  total_karma: number;
}

function App() {
  const [campaign, setCampaign] = useState<Campaign | null>(null);
  const [characters, setCharacters] = useState<CharacterSummary[]>([]);
  const [status, setStatus] = useState<string>("Ready");

  const handleCreateCampaign = async () => {
    try {
      const result = await invoke<Campaign>("create_campaign", {
        name: "Test Campaign",
      });
      setCampaign(result);
      setStatus(`Campaign created: ${result.name} (${result.id})`);
    } catch (err) {
      setStatus(`Error: ${err}`);
    }
  };

  const handleListCharacters = async () => {
    if (!campaign) return;
    try {
      const result = await invoke<CharacterSummary[]>("list_characters", {
        campaignId: campaign.id,
      });
      setCharacters(result);
      setStatus(`Found ${result.length} characters`);
    } catch (err) {
      setStatus(`Error: ${err}`);
    }
  };

  const handleCreateCharacter = async () => {
    if (!campaign) return;
    try {
      const result = await invoke<CharacterSummary>("create_character", {
        campaignId: campaign.id,
        edition: "SR4",
        name: "Test Runner",
        metatype: "Human",
      });
      setStatus(`Character created: ${result.name}`);
      handleListCharacters();
    } catch (err) {
      setStatus(`Error: ${err}`);
    }
  };

  return (
    <div className="min-h-screen p-8">
      <h1 className="text-3xl font-bold mb-6">personafix</h1>
      <p className="text-gray-400 mb-8">Shadowrun Character Manager</p>

      <div className="space-y-4">
        <div className="flex gap-4">
          <button
            onClick={handleCreateCampaign}
            className="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded text-white"
          >
            Create Campaign
          </button>
          <button
            onClick={handleListCharacters}
            disabled={!campaign}
            className="px-4 py-2 bg-green-600 hover:bg-green-700 rounded text-white disabled:opacity-50"
          >
            List Characters
          </button>
          <button
            onClick={handleCreateCharacter}
            disabled={!campaign}
            className="px-4 py-2 bg-purple-600 hover:bg-purple-700 rounded text-white disabled:opacity-50"
          >
            Create Character
          </button>
        </div>

        <div className="bg-gray-800 rounded p-4 font-mono text-sm">
          <p className="text-gray-400">Status: {status}</p>
          {campaign && (
            <p className="text-green-400 mt-1">
              Campaign: {campaign.name} ({campaign.id})
            </p>
          )}
        </div>

        {characters.length > 0 && (
          <div className="bg-gray-800 rounded p-4">
            <h2 className="text-lg font-semibold mb-2">Characters</h2>
            <ul className="space-y-1">
              {characters.map((c) => (
                <li key={c.id} className="text-gray-300">
                  {c.name} — {c.edition} {c.metatype} (Karma: {c.total_karma})
                </li>
              ))}
            </ul>
          </div>
        )}
      </div>
    </div>
  );
}

export default App;
