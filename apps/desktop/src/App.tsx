import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import {
  useCharacterStore,
  type Edition,
  type MetatypeKey,
} from "./store/characterStore";
import { useGameDataStore } from "./store/gameDataStore";
import BuilderShell from "./components/BuilderShell";
import SavedCharacterView from "./components/SavedCharacterView";

interface Campaign {
  id: string;
  name: string;
}

const EDITIONS: Edition[] = ["SR4", "SR5"];
const METATYPES: MetatypeKey[] = ["Human", "Elf", "Dwarf", "Ork", "Troll"];

function App() {
  const draft = useCharacterStore((s) => s.draft);
  const savedCharacter = useCharacterStore((s) => s.savedCharacter);
  const startNewCharacter = useCharacterStore((s) => s.startNewCharacter);

  const gameDataLoaded = useGameDataStore((s) => s.loaded);
  const gameDataLoading = useGameDataStore((s) => s.loading);
  const gameDataError = useGameDataStore((s) => s.error);
  const gameDebugInfo = useGameDataStore((s) => s.debugInfo);
  const gameLoadMessage = useGameDataStore((s) => s.loadMessage);
  const loadGameData = useGameDataStore((s) => s.loadGameData);
  const checkFile = useGameDataStore((s) => s.checkFile);

  const [campaign, setCampaign] = useState<Campaign | null>(null);
  const [campaignName, setCampaignName] = useState("My Campaign");
  const [charName, setCharName] = useState("Street Samurai");
  const [edition, setEdition] = useState<Edition>("SR4");
  const [metatype, setMetatype] = useState<MetatypeKey>("Human");
  const [error, setError] = useState<string | null>(null);
  const [gameDataPath, setGameDataPath] = useState("game_data.db");

  // All hooks must be above this line — React requires consistent hook order.

  if (savedCharacter) {
    return <SavedCharacterView />;
  }

  if (draft && campaign) {
    return <BuilderShell campaignId={campaign.id} />;
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

  const handleLoadGameData = async () => {
    await loadGameData(gameDataPath, edition);
  };

  const handleBrowseGameData = async () => {
    try {
      const selected = await open({
        title: "Select game_data.db",
        filters: [
          { name: "SQLite Database", extensions: ["db", "sqlite", "srx"] },
          { name: "All Files", extensions: ["*"] },
        ],
        multiple: false,
        directory: false,
      });
      if (selected) {
        setGameDataPath(selected);
        // Auto-check the file
        await checkFile(selected);
      }
    } catch (err) {
      setError(String(err));
    }
  };

  const handleDebugCheck = async () => {
    await checkFile(gameDataPath);
  };

  const handleStartBuilder = async () => {
    // Reload game data if edition changed
    if (gameDataLoaded) {
      try {
        await loadGameData(gameDataPath, edition);
      } catch {
        // Non-fatal — will use fallback data
      }
    }
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
          <img
            src="/icon.png"
            alt="personafix"
            className="w-24 h-24 mx-auto mb-4 rounded-2xl shadow-glow"
          />
          <h1 className="text-4xl font-bold mb-2 text-cyber-heading">
            personafix
          </h1>
          <p className="text-cyber-text-dim">Shadowrun Character Manager</p>
        </div>

        {!campaign ? (
          <div className="bg-cyber-card border border-cyber-border rounded-lg p-6 space-y-4">
            <h2 className="text-lg font-semibold text-cyber-heading">
              Create Campaign
            </h2>
            <input
              type="text"
              value={campaignName}
              onChange={(e) => setCampaignName(e.target.value)}
              placeholder="Campaign name"
              className="w-full bg-cyber-card border border-cyber-border rounded px-3 py-2 text-sm"
            />
            <button
              onClick={handleCreateCampaign}
              className="w-full px-4 py-2 bg-cyber-green-dim hover:bg-cyber-green/20 border border-cyber-green-dim hover:border-cyber-green rounded text-sm font-medium text-cyber-green transition-all shadow-glow"
            >
              Create Campaign
            </button>
          </div>
        ) : (
          <div className="bg-cyber-card border border-cyber-border rounded-lg p-6 space-y-4">
            <div className="text-sm text-cyber-text-dim font-mono">
              Campaign:{" "}
              <span className="text-cyber-green">{campaign.name}</span>
            </div>
            <h2 className="text-lg font-semibold text-cyber-heading">
              New Character
            </h2>
            <input
              type="text"
              value={charName}
              onChange={(e) => setCharName(e.target.value)}
              placeholder="Character name"
              className="w-full bg-cyber-card border border-cyber-border rounded px-3 py-2 text-sm"
            />
            <div className="grid grid-cols-2 gap-3">
              <div>
                <label className="text-xs text-cyber-text-dim block mb-1 font-mono">
                  Edition
                </label>
                <select
                  value={edition}
                  onChange={(e) => setEdition(e.target.value as Edition)}
                  className="w-full bg-cyber-card border border-cyber-border rounded px-3 py-2 text-sm"
                >
                  {EDITIONS.map((e) => (
                    <option key={e} value={e}>
                      {e}
                    </option>
                  ))}
                </select>
              </div>
              <div>
                <label className="text-xs text-cyber-text-dim block mb-1 font-mono">
                  Metatype
                </label>
                <select
                  value={metatype}
                  onChange={(e) => setMetatype(e.target.value as MetatypeKey)}
                  className="w-full bg-cyber-card border border-cyber-border rounded px-3 py-2 text-sm"
                >
                  {METATYPES.map((m) => (
                    <option key={m} value={m}>
                      {m}
                    </option>
                  ))}
                </select>
              </div>
            </div>

            {/* Game data */}
            <div className="border-t border-cyber-border pt-3">
              <label className="text-xs text-cyber-text-dim block mb-1 font-mono">
                Game Data DB
              </label>
              <div className="flex items-center gap-2">
                <input
                  type="text"
                  value={gameDataPath}
                  onChange={(e) => setGameDataPath(e.target.value)}
                  placeholder="Path to game_data.db"
                  className="flex-1 bg-cyber-card border border-cyber-border rounded px-3 py-1.5 text-xs font-mono"
                />
                <button
                  onClick={handleBrowseGameData}
                  className="px-3 py-1.5 rounded text-xs font-mono border border-cyber-border bg-cyber-card text-cyber-text-dim hover:border-cyber-border-bright transition-all"
                  title="Browse for game_data.db"
                >
                  Browse
                </button>
              </div>
              <div className="flex gap-2 mt-2">
                <button
                  onClick={handleLoadGameData}
                  disabled={gameDataLoading}
                  className={`flex-1 px-3 py-1.5 rounded text-xs font-mono border transition-all ${
                    gameDataLoaded
                      ? "bg-cyber-green-dim/20 border-cyber-green-dim text-cyber-green"
                      : "bg-cyber-card border-cyber-border text-cyber-text-dim hover:border-cyber-border-bright"
                  }`}
                >
                  {gameDataLoading
                    ? "Loading..."
                    : gameDataLoaded
                      ? "Reload"
                      : "Load Game Data"}
                </button>
                <button
                  onClick={handleDebugCheck}
                  className="px-3 py-1.5 rounded text-xs font-mono border border-cyber-border bg-cyber-card text-cyber-text-dim hover:border-cyber-border-bright transition-all"
                  title="Check if file exists and show path info"
                >
                  Debug
                </button>
              </div>
              {gameLoadMessage && (
                <p className="text-cyber-green text-xs font-mono mt-2 whitespace-pre-wrap">
                  {gameLoadMessage}
                </p>
              )}
              {gameDataError && (
                <p className="text-cyber-red text-xs font-mono mt-2 whitespace-pre-wrap">
                  {gameDataError}
                </p>
              )}
              {gameDebugInfo && (
                <pre className="text-cyber-blue text-xs font-mono mt-2 bg-cyber-surface border border-cyber-border rounded p-2 whitespace-pre-wrap">
                  {gameDebugInfo}
                </pre>
              )}
            </div>

            <button
              onClick={handleStartBuilder}
              className="w-full px-4 py-2 bg-cyber-green-dim hover:bg-cyber-green/20 border border-cyber-green-dim hover:border-cyber-green rounded text-sm font-medium text-cyber-green transition-all shadow-glow"
            >
              Start Building
            </button>
          </div>
        )}

        {error && (
          <div className="bg-cyber-red-dim/30 border border-cyber-red/50 rounded p-3 text-cyber-red text-sm">
            {error}
          </div>
        )}
      </div>
    </div>
  );
}

export default App;
