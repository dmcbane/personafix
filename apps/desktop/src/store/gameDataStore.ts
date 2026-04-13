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
  skills: GameSkill[];
  qualities: GameQuality[];
  weapons: GameWeapon[];
  augmentations: GameAugmentation[];

  loadGameData: (dbPath: string, edition: string) => Promise<void>;
}

export const useGameDataStore = create<GameDataState>((set) => ({
  loaded: false,
  loading: false,
  error: null,
  skills: [],
  qualities: [],
  weapons: [],
  augmentations: [],

  loadGameData: async (dbPath, edition) => {
    set({ loading: true, error: null });
    try {
      await invoke("load_game_data", { path: dbPath });

      const [skills, qualities, weapons, augmentations] = await Promise.all([
        invoke<GameSkill[]>("get_skills", { edition }),
        invoke<GameQuality[]>("get_qualities", { edition }),
        invoke<GameWeapon[]>("get_weapons", { edition }),
        invoke<GameAugmentation[]>("get_augmentations", { edition }),
      ]);

      set({
        loaded: true,
        loading: false,
        skills,
        qualities,
        weapons,
        augmentations,
      });
    } catch (err) {
      set({
        loading: false,
        error: String(err),
      });
    }
  },
}));
