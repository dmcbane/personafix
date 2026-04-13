import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

export interface GameSkill {
  id: string;
  name: string;
  linked_attribute: string;
  skill_group: string | null;
  source: string;
  page: string;
}

export interface GameQuality {
  id: string;
  name: string;
  quality_type: "Positive" | "Negative";
  cost: number;
  source: string;
  page: string;
}

export interface GameWeapon {
  id: string;
  name: string;
  category: string;
  damage: string;
  ap: string;
  mode: string;
  recoil_comp: string;
  ammo: string;
  availability: string;
  cost: string;
  source: string;
  page: string;
}

export interface GameAugmentation {
  id: string;
  name: string;
  augmentation_type: string;
  essence_cost: string;
  capacity: string;
  availability: string;
  cost: string;
  source: string;
  page: string;
}

interface GameDataState {
  loaded: boolean;
  loading: boolean;
  error: string | null;
  debugInfo: string | null;
  loadMessage: string | null;
  skills: GameSkill[];
  qualities: GameQuality[];
  weapons: GameWeapon[];
  augmentations: GameAugmentation[];

  loadGameData: (dbPath: string, edition: string) => Promise<void>;
  checkFile: (path: string) => Promise<void>;
}

export const useGameDataStore = create<GameDataState>((set) => ({
  loaded: false,
  loading: false,
  error: null,
  debugInfo: null,
  loadMessage: null,
  skills: [],
  qualities: [],
  weapons: [],
  augmentations: [],

  loadGameData: async (dbPath, edition) => {
    set({ loading: true, error: null, debugInfo: null, loadMessage: null });
    try {
      // load_game_data now returns a status message
      const msg = await invoke<string>("load_game_data", { path: dbPath });

      const [skills, qualities, weapons, augmentations] = await Promise.all([
        invoke<GameSkill[]>("get_skills", { edition }),
        invoke<GameQuality[]>("get_qualities", { edition }),
        invoke<GameWeapon[]>("get_weapons", { edition }),
        invoke<GameAugmentation[]>("get_augmentations", { edition }),
      ]);

      set({
        loaded: true,
        loading: false,
        loadMessage: `${msg} | ${skills.length} skills, ${qualities.length} qualities, ${weapons.length} weapons, ${augmentations.length} augmentations for ${edition}`,
        skills,
        qualities,
        weapons,
        augmentations,
      });
    } catch (err: unknown) {
      // Extract the error message — Tauri wraps errors in objects
      const errMsg =
        typeof err === "object" && err !== null && "message" in err
          ? (err as { message: string }).message
          : String(err);
      set({
        loading: false,
        error: errMsg,
      });
    }
  },

  checkFile: async (path) => {
    try {
      const info = await invoke<string>("debug_check_file", { path });
      set({ debugInfo: info });
    } catch (err) {
      set({ debugInfo: `Debug check failed: ${err}` });
    }
  },
}));
