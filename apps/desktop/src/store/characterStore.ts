import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

// Types matching the Rust model types
export interface Attributes {
  body: number;
  agility: number;
  reaction: number;
  strength: number;
  willpower: number;
  logic: number;
  intuition: number;
  charisma: number;
  edge: number;
  essence: number;
  magic: number | null;
  resonance: number | null;
}

export interface RacialLimits {
  metatype: string;
  edition: string;
  body: [number, number];
  agility: [number, number];
  reaction: [number, number];
  strength: [number, number];
  willpower: [number, number];
  logic: [number, number];
  intuition: [number, number];
  charisma: [number, number];
  edge: [number, number];
}

export interface Skill {
  id: string;
  name: string;
  linked_attribute: string;
  group: string | null;
  rating: number;
  specializations: { name: string; bonus: number }[];
}

export interface Quality {
  id: string;
  name: string;
  quality_type: "Positive" | "Negative";
  cost: number;
  source: string;
  page: string;
  improvements: unknown[];
  incompatible_with: string[];
}

export interface ValidationError {
  severity: "Error" | "Warning";
  field: string;
  message: string;
}

export interface CharacterDraft {
  name: string;
  edition: "SR4" | "SR5";
  metatype: string;
  attributes: Attributes;
  skills: Skill[];
  skill_groups: unknown[];
  qualities: Quality[];
  augmentations: unknown[];
  spells: unknown[];
  adept_powers: unknown[];
  complex_forms: unknown[];
  contacts: unknown[];
  weapons: unknown[];
  armor: unknown[];
  gear: unknown[];
  vehicles: unknown[];
  priority_selection: unknown | null;
  creation_points_spent: number;
  nuyen_spent: number;
}

export type Edition = "SR4" | "SR5";
export type MetatypeKey =
  | "Human"
  | "Elf"
  | "Dwarf"
  | "Ork"
  | "Troll";

export const ATTRIBUTE_NAMES = [
  "body",
  "agility",
  "reaction",
  "strength",
  "willpower",
  "logic",
  "intuition",
  "charisma",
  "edge",
] as const;

export type AttributeName = (typeof ATTRIBUTE_NAMES)[number];

interface CharacterState {
  // Current draft being built
  draft: CharacterDraft | null;
  racialLimits: RacialLimits | null;
  validationErrors: ValidationError[];

  // Actions
  startNewCharacter: (
    edition: Edition,
    metatype: MetatypeKey,
    name: string,
  ) => Promise<void>;
  setAttribute: (attr: AttributeName, value: number) => void;
  addSkill: (skill: Skill) => void;
  removeSkill: (skillId: string) => void;
  updateSkillRating: (skillId: string, rating: number) => void;
  addQuality: (quality: Quality) => void;
  removeQuality: (qualityId: string) => void;
  validate: () => Promise<void>;
}

export const useCharacterStore = create<CharacterState>((set, get) => ({
  draft: null,
  racialLimits: null,
  validationErrors: [],

  startNewCharacter: async (edition, metatype, name) => {
    const limits = await invoke<RacialLimits>("get_racial_limits", {
      edition,
      metatype,
    });

    const draft: CharacterDraft = {
      name,
      edition,
      metatype,
      attributes: {
        body: limits.body[0],
        agility: limits.agility[0],
        reaction: limits.reaction[0],
        strength: limits.strength[0],
        willpower: limits.willpower[0],
        logic: limits.logic[0],
        intuition: limits.intuition[0],
        charisma: limits.charisma[0],
        edge: limits.edge[0],
        essence: 600,
        magic: null,
        resonance: null,
      },
      skills: [],
      skill_groups: [],
      qualities: [],
      augmentations: [],
      spells: [],
      adept_powers: [],
      complex_forms: [],
      contacts: [],
      weapons: [],
      armor: [],
      gear: [],
      vehicles: [],
      priority_selection: null,
      creation_points_spent: 0,
      nuyen_spent: 0,
    };

    set({ draft, racialLimits: limits, validationErrors: [] });
  },

  setAttribute: (attr, value) => {
    const { draft } = get();
    if (!draft) return;

    set({
      draft: {
        ...draft,
        attributes: { ...draft.attributes, [attr]: value },
      },
    });
  },

  addSkill: (skill) => {
    const { draft } = get();
    if (!draft) return;
    set({ draft: { ...draft, skills: [...draft.skills, skill] } });
  },

  removeSkill: (skillId) => {
    const { draft } = get();
    if (!draft) return;
    set({
      draft: {
        ...draft,
        skills: draft.skills.filter((s) => s.id !== skillId),
      },
    });
  },

  updateSkillRating: (skillId, rating) => {
    const { draft } = get();
    if (!draft) return;
    set({
      draft: {
        ...draft,
        skills: draft.skills.map((s) =>
          s.id === skillId ? { ...s, rating } : s,
        ),
      },
    });
  },

  addQuality: (quality) => {
    const { draft } = get();
    if (!draft) return;
    set({
      draft: { ...draft, qualities: [...draft.qualities, quality] },
    });
  },

  removeQuality: (qualityId) => {
    const { draft } = get();
    if (!draft) return;
    set({
      draft: {
        ...draft,
        qualities: draft.qualities.filter((q) => q.id !== qualityId),
      },
    });
  },

  validate: async () => {
    const { draft } = get();
    if (!draft) return;
    const errors = await invoke<ValidationError[]>("validate_draft", {
      draft,
    });
    set({ validationErrors: errors });
  },
}));
