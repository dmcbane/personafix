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

export type PriorityLevel = "A" | "B" | "C" | "D" | "E";

export interface PrioritySelection {
  metatype: PriorityLevel;
  attributes: PriorityLevel;
  magic_or_resonance: PriorityLevel;
  skills: PriorityLevel;
  resources: PriorityLevel;
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
  priority_selection: PrioritySelection | null;
  creation_points_spent: number;
  nuyen_spent: number;
}

export interface ComputedCharacter {
  base: {
    id: string;
    name: string;
    edition: string;
    metatype: string;
  };
  computed_attributes: Attributes;
  physical_condition_monitor: number;
  stun_condition_monitor: number;
  initiative: number;
  initiative_dice: number;
  total_karma_earned: number;
  total_karma_spent: number;
  nuyen: number;
}

export type Edition = "SR4" | "SR5";
export type MetatypeKey = "Human" | "Elf" | "Dwarf" | "Ork" | "Troll";

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

export const PRIORITY_LEVELS: PriorityLevel[] = ["A", "B", "C", "D", "E"];

export type PriorityCategory =
  | "metatype"
  | "attributes"
  | "magic_or_resonance"
  | "skills"
  | "resources";

export const PRIORITY_CATEGORIES: {
  key: PriorityCategory;
  label: string;
}[] = [
  { key: "metatype", label: "Metatype" },
  { key: "attributes", label: "Attributes" },
  { key: "magic_or_resonance", label: "Magic/Resonance" },
  { key: "skills", label: "Skills" },
  { key: "resources", label: "Resources" },
];

// Priority table data for display
export const PRIORITY_TABLE: Record<
  PriorityCategory,
  Record<PriorityLevel, string>
> = {
  metatype: {
    A: "Any (13 special)",
    B: "Any (11 special)",
    C: "Any (9 special)",
    D: "Human/Elf (0)",
    E: "Human (0)",
  },
  attributes: { A: "24", B: "20", C: "16", D: "14", E: "12" },
  magic_or_resonance: {
    A: "Magician (6)",
    B: "Adept (6)",
    C: "Magician (3)",
    D: "Adept (2)",
    E: "Mundane",
  },
  skills: {
    A: "46/10",
    B: "36/5",
    C: "28/2",
    D: "22/0",
    E: "18/0",
  },
  resources: {
    A: "450,000¥",
    B: "275,000¥",
    C: "140,000¥",
    D: "50,000¥",
    E: "6,000¥",
  },
};

interface CharacterState {
  // Current draft being built
  draft: CharacterDraft | null;
  racialLimits: RacialLimits | null;
  validationErrors: ValidationError[];
  // Saved character (after finalization)
  savedCharacter: ComputedCharacter | null;

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
  setPriority: (category: PriorityCategory, level: PriorityLevel) => void;
  validate: () => Promise<void>;
  saveCharacter: (campaignId: string) => Promise<void>;
  reset: () => void;
}

export const useCharacterStore = create<CharacterState>((set, get) => ({
  draft: null,
  racialLimits: null,
  validationErrors: [],
  savedCharacter: null,

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
      priority_selection:
        edition === "SR5"
          ? {
              metatype: "A",
              attributes: "B",
              magic_or_resonance: "C",
              skills: "D",
              resources: "E",
            }
          : null,
      creation_points_spent: 0,
      nuyen_spent: 0,
    };

    set({
      draft,
      racialLimits: limits,
      validationErrors: [],
      savedCharacter: null,
    });
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

  setPriority: (category, level) => {
    const { draft } = get();
    if (!draft || !draft.priority_selection) return;
    set({
      draft: {
        ...draft,
        priority_selection: {
          ...draft.priority_selection,
          [category]: level,
        },
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

  saveCharacter: async (campaignId) => {
    const { draft } = get();
    if (!draft) return;

    // Create character in DB, then save the full base
    const summary = await invoke<{ id: string }>("create_character", {
      campaignId,
      edition: draft.edition,
      name: draft.name,
      metatype: draft.metatype,
    });

    // Build CharacterBase from draft
    const base = {
      id: summary.id,
      campaign_id: campaignId,
      name: draft.name,
      edition: draft.edition,
      metatype: draft.metatype,
      attributes: draft.attributes,
      skills: draft.skills,
      skill_groups: draft.skill_groups,
      qualities: draft.qualities,
      augmentations: draft.augmentations,
      spells: draft.spells,
      adept_powers: draft.adept_powers,
      complex_forms: draft.complex_forms,
      contacts: draft.contacts,
      weapons: draft.weapons,
      armor: draft.armor,
      gear: draft.gear,
      vehicles: draft.vehicles,
      priority_selection: draft.priority_selection,
    };

    const computed = await invoke<ComputedCharacter>(
      "save_character_base",
      { base },
    );
    set({ savedCharacter: computed, draft: null });
  },

  reset: () => {
    set({
      draft: null,
      racialLimits: null,
      validationErrors: [],
      savedCharacter: null,
    });
  },
}));
