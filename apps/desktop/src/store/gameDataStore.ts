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
  incompatible_with: string[];
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

export interface GameArmor {
  id: string;
  name: string;
  armor_value: string;
  availability: string;
  cost: string;
  source: string;
  page: string;
}

export interface GameSpell {
  id: string;
  name: string;
  category: string;
  spell_type: string;
  range: string;
  damage: string;
  duration: string;
  drain: string;
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

export interface GameAdeptPower {
  id: string;
  name: string;
  /** Decimal string like "0.25" or "1.00" */
  cost: string;
  levels: boolean;
  source: string;
  page: string;
}

export interface GameComplexForm {
  id: string;
  name: string;
  target: string;
  duration: string;
  /** Fading value, e.g. "L-2" */
  fading: string;
  source: string;
  page: string;
}

export interface GameVehicle {
  id: string;
  name: string;
  handling: string;
  speed: string;
  acceleration: string;
  body: string;
  armor: string;
  pilot: string;
  sensor: string;
  availability: string;
  cost: string;
  edition: string;
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
  armor: GameArmor[];
  augmentations: GameAugmentation[];
  spells: GameSpell[];
  adeptPowers: GameAdeptPower[];
  complexForms: GameComplexForm[];
  vehicles: GameVehicle[];

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
  armor: [],
  augmentations: [],
  spells: [],
  adeptPowers: [],
  complexForms: [],
  vehicles: [],

  loadGameData: async (dbPath, edition) => {
    set({ loading: true, error: null, debugInfo: null, loadMessage: null });
    try {
      // load_game_data now returns a status message
      const msg = await invoke<string>("load_game_data", { path: dbPath });

      const [skills, qualities, weapons, armor, augmentations, spells, adeptPowers, complexForms, vehicles] =
        await Promise.all([
          invoke<GameSkill[]>("get_skills", { edition }),
          invoke<GameQuality[]>("get_qualities", { edition }),
          invoke<GameWeapon[]>("get_weapons", { edition }),
          invoke<GameArmor[]>("get_armor", { edition }),
          invoke<GameAugmentation[]>("get_augmentations", { edition }),
          invoke<GameSpell[]>("get_spells", { edition }),
          invoke<GameAdeptPower[]>("get_adept_powers"),
          invoke<GameComplexForm[]>("get_complex_forms"),
          invoke<GameVehicle[]>("get_vehicles"),
        ]);

      set({
        loaded: true,
        loading: false,
        loadMessage: `${msg} | ${skills.length} skills, ${qualities.length} qualities, ${weapons.length} weapons, ${armor.length} armor, ${augmentations.length} augmentations, ${spells.length} spells, ${adeptPowers.length} powers, ${complexForms.length} complex forms, ${vehicles.length} vehicles for ${edition}`,
        skills,
        qualities,
        weapons,
        armor,
        augmentations,
        spells,
        adeptPowers,
        complexForms,
        vehicles,
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
