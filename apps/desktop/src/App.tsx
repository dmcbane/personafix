import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { fmtErr } from "./utils";
import {
  useCharacterStore,
  type Edition,
  type MetatypeKey,
} from "./store/characterStore";
import { useGameDataStore } from "./store/gameDataStore";
import { useSettingsStore, FONT_SIZE_PX } from "./store/settingsStore";
import BuilderShell from "./components/BuilderShell";
import SavedCharacterView from "./components/SavedCharacterView";
import PlayView from "./components/PlayView";
import SettingsPanel from "./components/SettingsPanel";

interface Campaign {
  id: string;
  name: string;
}

interface RecentCampaign {
  path: string;
  name: string;
  last_opened: string;
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
  const [playMode, setPlayMode] = useState(false);

  const [campaign, setCampaign] = useState<Campaign | null>(null);
  const [recents, setRecents] = useState<RecentCampaign[]>([]);
  const [recentErrors, setRecentErrors] = useState<Record<string, string>>({});
  const [campaignsDir, setCampaignsDir] = useState<string | null>(null);
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

  // Load recent campaigns and campaigns directory on mount.
  useEffect(() => {
    invoke<RecentCampaign[]>("get_recent_campaigns").then(setRecents).catch(() => {});
    invoke<string>("get_campaigns_dir").then(setCampaignsDir).catch(() => {});
  }, []);

  // Auto-load game data on mount and whenever the edition selector changes.
  useEffect(() => {
    loadGameData(gameDataPath, edition).catch(() => {});
  }, [edition]);

  // Refresh character list whenever we return to the campaign screen.
  useEffect(() => {
    if (campaign && !draft && !savedCharacter) {
      setPlayMode(false);
      listCharacters(campaign.id).catch(() => {});
    }
  }, [campaign, draft, savedCharacter]);

  // All hooks must be above this line — React requires consistent hook order.

  if (playMode && savedCharacter) {
    return <PlayView onBack={() => setPlayMode(false)} />;
  }

  if (savedCharacter) {
    return (
      <>
        <SavedCharacterView onPlayMode={() => setPlayMode(true)} />
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

  const handleOpenCampaign = async () => {
    try {
      const selected = await open({
        title: "Open Campaign",
        filters: [{ name: "Campaign File", extensions: ["srx"] }],
        multiple: false,
        directory: false,
        ...(campaignsDir ? { defaultPath: campaignsDir } : {}),
      });
      if (!selected) return;
      const result = await invoke<Campaign>("open_campaign", { path: selected });
      setCampaign(result);
      await listCharacters(result.id);
      invoke<RecentCampaign[]>("get_recent_campaigns").then(setRecents).catch(() => {});
      setError(null);
    } catch (err) {
      setError(fmtErr(err));
    }
  };

  const handleOpenRecent = async (path: string) => {
    try {
      setRecentErrors((prev) => { const n = { ...prev }; delete n[path]; return n; });
      const result = await invoke<Campaign>("open_campaign", { path });
      setCampaign(result);
      await listCharacters(result.id);
      invoke<RecentCampaign[]>("get_recent_campaigns").then(setRecents).catch(() => {});
      setError(null);
    } catch (err) {
      setRecentErrors((prev) => ({ ...prev, [path]: fmtErr(err) }));
    }
  };

  const handleImportCharacter = async () => {
    if (!campaign) return;
    try {
      const selected = await open({
        title: "Import Character JSON",
        filters: [{ name: "JSON File", extensions: ["json"] }],
        multiple: false,
        directory: false,
      });
      if (!selected) return;
      await invoke<string>("import_character_json", {
        campaignId: campaign.id,
        filePath: selected,
      });
      await listCharacters(campaign.id);
      setError(null);
    } catch (err) {
      setError(fmtErr(err));
    }
  };

  const handleImportChummer = async () => {
    if (!campaign) return;
    try {
      const selected = await open({
        title: "Import Chummer Character",
        filters: [{ name: "Chummer File", extensions: ["chum5", "chum"] }],
        multiple: false,
        directory: false,
      });
      if (!selected) return;
      await invoke<string>("import_chummer_character", {
        campaignId: campaign.id,
        chumPath: selected,
      });
      await listCharacters(campaign.id);
      setError(null);
    } catch (err) {
      setError(fmtErr(err));
    }
  };

  const handleCreateCampaign = async () => {
    try {
      const result = await invoke<Campaign>("create_campaign", {
        name: campaignName,
      });
      setCampaign(result);
      await listCharacters(result.id);
      invoke<RecentCampaign[]>("get_recent_campaigns").then(setRecents).catch(() => {});
      setError(null);
    } catch (err) {
      setError(fmtErr(err));
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
      setError(fmtErr(err));
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
      setError(fmtErr(err));
    }
  };

  const handleLoadCharacter = async (id: string) => {
    try {
      await loadCharacter(id);
      setError(null);
    } catch (err) {
      setError(fmtErr(err));
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
            <div className="space-y-4">
              <div className="bg-cyber-card border border-cyber-border rounded-lg p-6 space-y-4">
                <h2 className="text-lg font-semibold text-cyber-heading">
                  Campaign
                </h2>
                <input
                  type="text"
                  value={campaignName}
                  onChange={(e) => setCampaignName(e.target.value)}
                  onKeyDown={(e) => e.key === "Enter" && handleCreateCampaign()}
                  placeholder="Campaign name"
                  className="w-full bg-cyber-card border border-cyber-border rounded px-3 py-2 text-sm"
                />
                <div className="flex gap-2">
                  <button
                    onClick={handleCreateCampaign}
                    className="flex-1 px-4 py-2 bg-cyber-green-dim hover:bg-cyber-green/20 border border-cyber-green-dim hover:border-cyber-green rounded text-sm font-medium text-cyber-green transition-all shadow-glow"
                  >
                    Create
                  </button>
                  <button
                    onClick={handleOpenCampaign}
                    className="flex-1 px-4 py-2 border border-cyber-border text-cyber-text-dim hover:border-cyber-border-bright hover:text-cyber-text rounded text-sm font-medium transition-all"
                  >
                    Open...
                  </button>
                </div>
                {campaignsDir && (
                  <p className="text-xs text-cyber-text-dim font-mono truncate" title={campaignsDir}>
                    Campaigns saved to: {campaignsDir}
                  </p>
                )}
              </div>

              {recents.length > 0 && (
                <div className="bg-cyber-card border border-cyber-border rounded-lg p-4 space-y-2">
                  <h3 className="text-xs font-mono text-cyber-text-dim">// Recent</h3>
                  {recents.map((r) => (
                    <div key={r.path}>
                      <button
                        onClick={() => handleOpenRecent(r.path)}
                        className="w-full text-left px-3 py-2 rounded border border-cyber-border text-cyber-text-dim hover:text-cyber-text hover:border-cyber-border-bright transition-colors text-sm"
                      >
                        <div className="font-medium text-cyber-text truncate">{r.name}</div>
                        <div className="text-xs font-mono text-cyber-text-dim truncate mt-0.5">{r.path}</div>
                      </button>
                      {recentErrors[r.path] && (
                        <p className="text-cyber-red text-xs font-mono px-3 pb-1">
                          {recentErrors[r.path]}
                        </p>
                      )}
                    </div>
                  ))}
                </div>
              )}
            </div>
          ) : (
            <div className="space-y-4">
              {/* Campaign header with back button */}
              <div className="flex items-center justify-between">
                <h2 className="text-sm font-mono text-cyber-text-dim">// {campaign.name}</h2>
                <button
                  onClick={() => setCampaign(null)}
                  className="text-xs font-mono text-cyber-text-dim hover:text-cyber-text transition-colors"
                >
                  ← Home
                </button>
              </div>

              {/* Character list */}
              <div className="bg-cyber-card border border-cyber-border rounded-lg p-4 space-y-3">
                <h2 className="text-sm font-semibold text-cyber-heading font-mono sr-only">
                  // {campaign.name}
                </h2>
                {characters.length > 0 && (
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
                )}
                <div className="flex gap-2 pt-1">
                  <button
                    onClick={handleImportCharacter}
                    className="flex-1 px-3 py-1.5 border border-cyber-border text-cyber-text-dim hover:border-cyber-border-bright hover:text-cyber-text rounded text-xs font-mono transition-colors"
                  >
                    Import JSON…
                  </button>
                  <button
                    onClick={handleImportChummer}
                    className="flex-1 px-3 py-1.5 border border-cyber-border text-cyber-text-dim hover:border-cyber-border-bright hover:text-cyber-text rounded text-xs font-mono transition-colors"
                  >
                    Import Chummer…
                  </button>
                </div>
              </div>

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
