import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import {
  useCharacterStore,
  type Edition,
  type MetatypeKey,
} from "./store/characterStore";
import { useGameDataStore } from "./store/gameDataStore";
import { useSettingsStore, FONT_SIZE_PX } from "./store/settingsStore";
import BuilderShell from "./components/BuilderShell";
import SavedCharacterView from "./components/SavedCharacterView";
import SettingsPanel from "./components/SettingsPanel";

interface Campaign {
  id: string;
  name: string;
}

const EDITIONS: Edition[] = ["SR4", "SR5"];
const METATYPES: MetatypeKey[] = ["Human", "Elf", "Dwarf", "Ork", "Troll"];

function App() {
  const draft = useCharacterStore((s) => s.draft);
  const savedCharacter = useCharacterStore((s) => s.savedCharacter);
  const characters = useCharacterStore((s) => s.characters);
  const startNewCharacter = useCharacterStore((s) => s.startNewCharacter);
  const listCharacters = useCharacterStore((s) => s.listCharacters);
  const loadCharacter = useCharacterStore((s) => s.loadCharacter);

  const gameDataLoaded = useGameDataStore((s) => s.loaded);
  const gameDataLoading = useGameDataStore((s) => s.loading);
  const gameDataError = useGameDataStore((s) => s.error);
  const gameLoadMessage = useGameDataStore((s) => s.loadMessage);
  const loadGameData = useGameDataStore((s) => s.loadGameData);
  const checkFile = useGameDataStore((s) => s.checkFile);

  const { theme, font, fontSize } = useSettingsStore();
  const [settingsOpen, setSettingsOpen] = useState(false);

  const [campaign, setCampaign] = useState<Campaign | null>(null);
  const [campaignName, setCampaignName] = useState("My Campaign");
  const [charName, setCharName] = useState("Street Samurai");
  const [edition, setEdition] = useState<Edition>("SR4");
  const [metatype, setMetatype] = useState<MetatypeKey>("Human");
  const [error, setError] = useState<string | null>(null);
  const [gameDataPath, setGameDataPath] = useState("game_data.db");

  // Apply theme, font, and font-size to the html element on every change.
  useEffect(() => {
    const html = document.documentElement;
    html.setAttribute("data-theme", theme);
    if (font === "mono") {
      html.removeAttribute("data-font");
    } else {
      html.setAttribute("data-font", font);
    }
    html.style.fontSize = FONT_SIZE_PX[fontSize];
  }, [theme, font, fontSize]);

  // Auto-load game data on mount and whenever the edition selector changes.
  useEffect(() => {
    loadGameData(gameDataPath, edition).catch(() => {});
  }, [edition]);

  // Refresh character list whenever we return to the campaign screen.
  useEffect(() => {
    if (campaign && !draft && !savedCharacter) {
      listCharacters(campaign.id).catch(() => {});
    }
  }, [campaign, draft, savedCharacter]);

  // All hooks must be above this line — React requires consistent hook order.

  if (savedCharacter) {
    return (
      <>
        <SavedCharacterView />
        {settingsOpen && <SettingsPanel onClose={() => setSettingsOpen(false)} />}
      </>
    );
  }

  if (draft && campaign) {
    return (
      <>
        <BuilderShell campaignId={campaign.id} onSettingsOpen={() => setSettingsOpen(true)} />
        {settingsOpen && <SettingsPanel onClose={() => setSettingsOpen(false)} />}
      </>
    );
  }

  const handleCreateCampaign = async () => {
    try {
      const result = await invoke<Campaign>("create_campaign", {
        name: campaignName,
      });
      setCampaign(result);
      await listCharacters(result.id);
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
        await checkFile(selected);
      }
    } catch (err) {
      setError(String(err));
    }
  };

  const handleStartBuilder = async () => {
    try {
      await loadGameData(gameDataPath, edition);
    } catch {
      // Non-fatal — panels fall back to seed data if db unavailable
    }
    try {
      await startNewCharacter(edition, metatype, charName);
      setError(null);
    } catch (err) {
      setError(String(err));
    }
  };

  const handleLoadCharacter = async (id: string) => {
    try {
      await loadCharacter(id);
      setError(null);
    } catch (err) {
      setError(String(err));
    }
  };

  return (
    <>
      <div className="min-h-screen flex items-center justify-center p-8">
        {/* Settings button — top-right corner */}
        <button
          onClick={() => setSettingsOpen(true)}
          className="fixed top-4 right-4 z-40 p-2 rounded bg-cyber-card border border-cyber-border text-cyber-text-dim hover:text-cyber-text hover:border-cyber-border-bright transition-colors"
          aria-label="Open settings"
          title="Settings"
        >
          ⚙
        </button>

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
            <div className="space-y-4">
              {/* Character list */}
              {characters.length > 0 && (
                <div className="bg-cyber-card border border-cyber-border rounded-lg p-4">
                  <h2 className="text-sm font-semibold text-cyber-heading font-mono mb-3">
                    // {campaign.name}
                  </h2>
                  <div className="space-y-1">
                    {characters.map((c) => (
                      <div
                        key={c.id}
                        className="flex items-center gap-3 bg-cyber-surface border border-cyber-border rounded px-3 py-2"
                      >
                        <div className="flex-1 min-w-0">
                          <span className="text-cyber-text text-sm font-medium truncate block">
                            {c.name}
                          </span>
                          <span className="text-cyber-text-dim text-xs font-mono">
                            {c.edition} {c.metatype}
                          </span>
                        </div>
                        <button
                          onClick={() => handleLoadCharacter(c.id)}
                          className="px-3 py-1 text-xs font-mono border border-cyber-border-bright text-cyber-blue hover:bg-cyber-blue/10 rounded transition-colors shrink-0"
                        >
                          Open
                        </button>
                      </div>
                    ))}
                  </div>
                </div>
              )}

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

                {/* Game data status */}
                <div className="border-t border-cyber-border pt-3">
                  {gameDataLoading && (
                    <p className="text-cyber-text-dim text-xs font-mono">
                      Loading game data…
                    </p>
                  )}
                  {gameDataLoaded && !gameDataLoading && (
                    <p className="text-cyber-green text-xs font-mono truncate">
                      {gameLoadMessage ?? "Game data loaded"}
                    </p>
                  )}
                  {gameDataError && !gameDataLoaded && (
                    <details className="text-cyber-text-dim text-xs font-mono">
                      <summary className="cursor-pointer text-cyber-yellow">
                        Game data not found — using built-in data
                      </summary>
                      <div className="mt-2 space-y-2">
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
                          >
                            Browse
                          </button>
                        </div>
                        <button
                          onClick={handleLoadGameData}
                          disabled={gameDataLoading}
                          className="w-full px-3 py-1.5 rounded text-xs font-mono border border-cyber-border bg-cyber-card text-cyber-text-dim hover:border-cyber-border-bright transition-all"
                        >
                          Retry Load
                        </button>
                        <pre className="text-cyber-red text-xs font-mono whitespace-pre-wrap">
                          {gameDataError}
                        </pre>
                      </div>
                    </details>
                  )}
                </div>

                <button
                  onClick={handleStartBuilder}
                  className="w-full px-4 py-2 bg-cyber-green-dim hover:bg-cyber-green/20 border border-cyber-green-dim hover:border-cyber-green rounded text-sm font-medium text-cyber-green transition-all shadow-glow"
                >
                  Start Building
                </button>
              </div>
            </div>
          )}

          {error && (
            <div className="bg-cyber-red-dim/30 border border-cyber-red/50 rounded p-3 text-cyber-red text-sm">
              {error}
            </div>
          )}
        </div>
      </div>

      {settingsOpen && <SettingsPanel onClose={() => setSettingsOpen(false)} />}
    </>
  );
}

export default App;
