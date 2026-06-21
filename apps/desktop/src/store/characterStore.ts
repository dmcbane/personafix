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

// Subset of Rust's LedgerEvent enum (externally tagged serde format)
export type LedgerEvent =
  | { KarmaReceived: { amount: number; reason: string; run_id: null } }
  | { KarmaSpent: { amount: number; description: string } }
  | { NuyenReceived: { amount: number; reason: string; run_id: null } }
  | { NuyenSpent: { amount: number; description: string } }
  | {
      SkillImproved: {
        skill_name: string;
        from: number;
        to: number;
        karma_cost: number;
      };
    }
  | {
      AttributeImproved: {
        attribute: string;
        from: number;
        to: number;
        karma_cost: number;
      };
    }
  | {
      ContactChanged: {
        contact_id: string;
        new_connection: number;
        new_loyalty: number;
      };
    }
  | {
      ContactAdded: {
        contact_id: string;
        name: string;
        connection: number;
        loyalty: number;
      };
    }
  | { ContactLost: { contact_id: string; reason: string } }
  | { Initiated: { new_grade: number; karma_cost: number } }
  | { Submerged: { new_grade: number; karma_cost: number } };

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

export type MagicTradition = "Magician" | "Adept" | "MysticAdept" | "Technomancer";

export interface CharacterDraft {
  name: string;
  edition: "SR4" | "SR5";
  metatype: string;
  attributes: Attributes;
  skills: Skill[];
  skill_groups: unknown[];
  knowledge_skills: KnowledgeSkill[];
  qualities: Quality[];
  augmentations: DraftAugmentation[];
  spells: DraftSpell[];
  adept_powers: DraftAdeptPower[];
  complex_forms: DraftComplexForm[];
  contacts: Contact[];
  weapons: DraftWeapon[];
  armor: DraftArmor[];
  gear: unknown[];
  vehicles: DraftVehicle[];
  priority_selection: PrioritySelection | null;
  magic_tradition: MagicTradition | null;
  tradition_name: string | null;
  creation_points_spent: number;
  nuyen_spent: number;
}

export interface ComputedCharacter {
  base: {
    id: string;
    name: string;
    edition: string;
    metatype: string;
    attributes: Attributes;
    skills: Skill[];
    knowledge_skills: KnowledgeSkill[];
    qualities: Quality[];
    augmentations: DraftAugmentation[];
    spells: DraftSpell[];
    adept_powers: DraftAdeptPower[];
    complex_forms: DraftComplexForm[];
    contacts: Contact[];
    weapons: DraftWeapon[];
    armor: DraftArmor[];
    gear: GearItem[];
    vehicles: DraftVehicle[];
    magic_tradition: MagicTradition | null;
    tradition_name: string | null;
    notes: string;
  };
  computed_attributes: Attributes;
  physical_condition_monitor: number;
  stun_condition_monitor: number;
  initiative: number;
  initiative_dice: number;
  total_karma_earned: number;
  total_karma_spent: number;
  nuyen: number;
  initiation_grade: number;
  submersion_grade: number;
  effective_armor: number;
}

export type AugmentationGrade = "Standard" | "Alpha" | "Beta" | "Delta" | "Used";
export type AugmentationType = "Cyberware" | "Bioware";

export const GRADE_MULTIPLIER: Record<AugmentationGrade, number> = {
  Standard: 100,
  Alpha: 80,
  Beta: 70,
  Delta: 50,
  Used: 125,
};

export interface DraftAugmentation {
  id: string;
  name: string;
  augmentation_type: AugmentationType;
  grade: AugmentationGrade;
  essence_cost: number;
  availability: string;
  cost: number;
  source: string;
  page: string;
  improvements: unknown[];
}

export interface DraftWeapon {
  id: string;
  name: string;
  category: string;
  damage: string;
  ap: string;
  mode: string;
  recoil_comp: number;
  ammo: string;
  availability: string;
  cost: number;
  source: string;
  page: string;
}

export interface DraftArmor {
  id: string;
  name: string;
  armor_value: number;
  availability: string;
  cost: number;
  source: string;
  page: string;
}

export interface GearItem {
  id: string;
  name: string;
  category: string;
  rating: number | null;
  availability: string;
  cost: number;
  source: string;
  page: string;
}

export const KNOWLEDGE_SKILL_CATEGORIES = [
  "Academic",
  "Interest",
  "Language",
  "Professional",
  "Street",
  "Technical",
] as const;

export interface KnowledgeSkill {
  name: string;
  category: string;
  rating: number;
}

export interface DraftVehicle {
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
  source: string;
  page: string;
}

export interface DraftSpell {
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

export interface DraftAdeptPower {
  id: string;
  name: string;
  /** Power point cost in hundredths (25 = 0.25 PP) — matches Rust AdeptPower.cost: i32 */
  cost: number;
  levels: boolean;
  source: string;
  page: string;
}

export interface DraftComplexForm {
  id: string;
  name: string;
  target: string;
  duration: string;
  /** Fading value, e.g. "L-2" */
  fading: string;
  source: string;
  page: string;
}

export interface Contact {
  id: string;
  name: string;
  connection: number;
  loyalty: number;
  archetype: string;
  notes: string;
}

export interface CharacterSummary {
  id: string;
  name: string;
  edition: Edition;
  metatype: MetatypeKey;
  total_karma: number;
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

// SR5 priority numeric helpers (mirror Rust sr5_priority.rs constants)
export const SR5_ATTR_POINTS: Record<PriorityLevel, number> = {
  A: 24, B: 20, C: 16, D: 14, E: 12,
};
export const SR5_SKILL_POINTS: Record<PriorityLevel, [number, number]> = {
  A: [46, 10], B: [36, 5], C: [28, 2], D: [22, 0], E: [18, 0],
};
export const SR5_RESOURCE_NUYEN: Record<PriorityLevel, number> = {
  A: 450_000, B: 275_000, C: 140_000, D: 50_000, E: 6_000,
};
// Starting magic/resonance from magic priority; 0 = Mundane (no magic)
export const SR5_MAGIC_STARTING: Record<PriorityLevel, number> = {
  A: 6, B: 4, C: 3, D: 2, E: 0,
};
// Special attribute points from metatype priority (for raising Magic/Edge above starting/min)
export const SR5_SPECIAL_ATTR_POINTS: Record<PriorityLevel, number> = {
  A: 13, B: 11, C: 9, D: 4, E: 1,
};

interface CharacterState {
  // Current draft being built
  draft: CharacterDraft | null;
  racialLimits: RacialLimits | null;
  validationErrors: ValidationError[];
  // Saved character (after finalization or load)
  savedCharacter: ComputedCharacter | null;
  // Characters in the active campaign
  characters: CharacterSummary[];

  // Actions
  startNewCharacter: (
    edition: Edition,
    metatype: MetatypeKey,
    name: string,
  ) => Promise<void>;
  setAttribute: (attr: AttributeName, value: number) => void;
  setMagic: (value: number | null) => void;
  setResonance: (value: number | null) => void;
  setMagicTradition: (tradition: MagicTradition | null) => void;
  setTraditionName: (name: string | null) => void;
  addSkill: (skill: Skill) => void;
  removeSkill: (skillId: string) => void;
  updateSkillRating: (skillId: string, rating: number) => void;
  addSpecialization: (skillId: string, specName: string) => void;
  removeSpecialization: (skillId: string, specName: string) => void;
  addQuality: (quality: Quality) => void;
  removeQuality: (qualityId: string) => void;
  addAugmentation: (aug: DraftAugmentation) => void;
  removeAugmentation: (augId: string) => void;
  addWeapon: (weapon: DraftWeapon) => void;
  removeWeapon: (weaponId: string) => void;
  addArmor: (armor: DraftArmor) => void;
  removeArmor: (armorId: string) => void;
  addVehicle: (vehicle: DraftVehicle) => void;
  removeVehicle: (vehicleId: string) => void;
  addKnowledgeSkill: (skill: KnowledgeSkill) => void;
  removeKnowledgeSkill: (name: string) => void;
  addSpell: (spell: DraftSpell) => void;
  removeSpell: (spellId: string) => void;
  addAdeptPower: (power: DraftAdeptPower) => void;
  removeAdeptPower: (powerId: string) => void;
  addComplexForm: (form: DraftComplexForm) => void;
  removeComplexForm: (formId: string) => void;
  addContact: (contact: Contact) => void;
  removeContact: (contactId: string) => void;
  setPriority: (category: PriorityCategory, level: PriorityLevel) => void;
  validate: () => Promise<void>;
  saveCharacter: (campaignId: string) => Promise<void>;
  listCharacters: (campaignId: string) => Promise<void>;
  loadCharacter: (id: string) => Promise<void>;
  applyEvent: (characterId: string, event: LedgerEvent) => Promise<void>;
  initiate: (characterId: string, currentGrade: number) => Promise<void>;
  submerge: (characterId: string, currentGrade: number) => Promise<void>;
  updateNotes: (characterId: string, notes: string) => Promise<void>;
  reset: () => void;
}

export const useCharacterStore = create<CharacterState>((set, get) => ({
  draft: null,
  racialLimits: null,
  validationErrors: [],
  savedCharacter: null,
  characters: [],

  startNewCharacter: async (edition, metatype, name) => {
    console.log("[personafix] startNewCharacter:", { edition, metatype, name });
    const limits = await invoke<RacialLimits>("get_racial_limits", {
      edition,
      metatype,
    });
    console.log("[personafix] racial limits:", limits);

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
        // SR5 default magic priority is C (Magician 3); SR4 starts mundane
        magic: edition === "SR5" ? SR5_MAGIC_STARTING["C"] : null,
        resonance: null,
      },
      skills: [],
      skill_groups: [],
      knowledge_skills: [],
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
      // SR5 default: priority C for magic = Magician. SR4 is mundane.
      magic_tradition: edition === "SR5" ? "Magician" : null,
      tradition_name: null,
      creation_points_spent: 0,
      nuyen_spent: 0,
    };

    console.log("[personafix] draft created:", draft);
    set({
      draft,
      racialLimits: limits,
      validationErrors: [],
      savedCharacter: null,
    });
    console.log("[personafix] state set, builder should render");
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

  setMagic: (value) => {
    const { draft } = get();
    if (!draft) return;
    set({
      draft: {
        ...draft,
        attributes: { ...draft.attributes, magic: value },
      },
    });
  },

  setResonance: (value) => {
    const { draft } = get();
    if (!draft) return;
    set({
      draft: {
        ...draft,
        attributes: { ...draft.attributes, resonance: value },
      },
    });
  },

  setMagicTradition: (tradition) => {
    const { draft } = get();
    if (!draft) return;
    const startingVal = draft.priority_selection
      ? SR5_MAGIC_STARTING[draft.priority_selection.magic_or_resonance]
      : 0;
    const isTech = tradition === "Technomancer";
    set({
      draft: {
        ...draft,
        magic_tradition: tradition,
        attributes: {
          ...draft.attributes,
          magic: !isTech && tradition && startingVal > 0 ? startingVal : null,
          resonance: isTech && startingVal > 0 ? startingVal : null,
        },
      },
    });
  },

  setTraditionName: (name) => {
    const { draft } = get();
    if (!draft) return;
    set({ draft: { ...draft, tradition_name: name } });
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

  addSpecialization: (skillId, specName) => {
    const { draft } = get();
    if (!draft) return;
    const trimmed = specName.trim();
    if (!trimmed) return;
    set({
      draft: {
        ...draft,
        skills: draft.skills.map((s) =>
          s.id === skillId && !s.specializations.some((sp) => sp.name === trimmed)
            ? { ...s, specializations: [...s.specializations, { name: trimmed, bonus: 2 }] }
            : s,
        ),
      },
    });
  },

  removeSpecialization: (skillId, specName) => {
    const { draft } = get();
    if (!draft) return;
    set({
      draft: {
        ...draft,
        skills: draft.skills.map((s) =>
          s.id === skillId
            ? { ...s, specializations: s.specializations.filter((sp) => sp.name !== specName) }
            : s,
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

  addAugmentation: (aug) => {
    const { draft } = get();
    if (!draft) return;
    set({ draft: { ...draft, augmentations: [...draft.augmentations, aug] } });
  },

  removeAugmentation: (augId) => {
    const { draft } = get();
    if (!draft) return;
    set({
      draft: {
        ...draft,
        augmentations: draft.augmentations.filter((a) => a.id !== augId),
      },
    });
  },

  addWeapon: (weapon) => {
    const { draft } = get();
    if (!draft) return;
    set({
      draft: {
        ...draft,
        weapons: [...draft.weapons, weapon],
        nuyen_spent: draft.nuyen_spent + weapon.cost,
      },
    });
  },

  removeWeapon: (weaponId) => {
    const { draft } = get();
    if (!draft) return;
    const removed = draft.weapons.find((w) => w.id === weaponId);
    set({
      draft: {
        ...draft,
        weapons: draft.weapons.filter((w) => w.id !== weaponId),
        nuyen_spent: Math.max(0, draft.nuyen_spent - (removed?.cost ?? 0)),
      },
    });
  },

  addArmor: (armor) => {
    const { draft } = get();
    if (!draft) return;
    set({
      draft: {
        ...draft,
        armor: [...draft.armor, armor],
        nuyen_spent: draft.nuyen_spent + armor.cost,
      },
    });
  },

  removeArmor: (armorId) => {
    const { draft } = get();
    if (!draft) return;
    const removed = draft.armor.find((a) => a.id === armorId);
    set({
      draft: {
        ...draft,
        armor: draft.armor.filter((a) => a.id !== armorId),
        nuyen_spent: Math.max(0, draft.nuyen_spent - (removed?.cost ?? 0)),
      },
    });
  },

  addVehicle: (vehicle) => {
    const { draft } = get();
    if (!draft) return;
    if (draft.vehicles.some((v) => v.id === vehicle.id)) return;
    set({
      draft: {
        ...draft,
        vehicles: [...draft.vehicles, vehicle],
        nuyen_spent: draft.nuyen_spent + Number(vehicle.cost),
      },
    });
  },

  removeVehicle: (vehicleId) => {
    const { draft } = get();
    if (!draft) return;
    const removed = draft.vehicles.find((v) => v.id === vehicleId);
    set({
      draft: {
        ...draft,
        vehicles: draft.vehicles.filter((v) => v.id !== vehicleId),
        nuyen_spent: Math.max(0, draft.nuyen_spent - Number(removed?.cost ?? 0)),
      },
    });
  },

  addKnowledgeSkill: (skill) => {
    const { draft } = get();
    if (!draft) return;
    if (draft.knowledge_skills.some((k) => k.name.toLowerCase() === skill.name.toLowerCase())) return;
    set({ draft: { ...draft, knowledge_skills: [...draft.knowledge_skills, skill] } });
  },

  removeKnowledgeSkill: (name) => {
    const { draft } = get();
    if (!draft) return;
    set({
      draft: {
        ...draft,
        knowledge_skills: draft.knowledge_skills.filter((k) => k.name !== name),
      },
    });
  },

  addSpell: (spell) => {
    const { draft } = get();
    if (!draft) return;
    set({ draft: { ...draft, spells: [...draft.spells, spell] } });
  },

  removeSpell: (spellId) => {
    const { draft } = get();
    if (!draft) return;
    set({
      draft: {
        ...draft,
        spells: draft.spells.filter((s) => s.id !== spellId),
      },
    });
  },

  addAdeptPower: (power) => {
    const { draft } = get();
    if (!draft) return;
    set({ draft: { ...draft, adept_powers: [...draft.adept_powers, power] } });
  },

  removeAdeptPower: (powerId) => {
    const { draft } = get();
    if (!draft) return;
    set({
      draft: {
        ...draft,
        adept_powers: draft.adept_powers.filter((p) => p.id !== powerId),
      },
    });
  },

  addComplexForm: (form) => {
    const { draft } = get();
    if (!draft) return;
    set({ draft: { ...draft, complex_forms: [...draft.complex_forms, form] } });
  },

  removeComplexForm: (formId) => {
    const { draft } = get();
    if (!draft) return;
    set({
      draft: {
        ...draft,
        complex_forms: draft.complex_forms.filter((f) => f.id !== formId),
      },
    });
  },

  addContact: (contact) => {
    const { draft } = get();
    if (!draft) return;
    set({ draft: { ...draft, contacts: [...draft.contacts, contact] } });
  },

  removeContact: (contactId) => {
    const { draft } = get();
    if (!draft) return;
    set({
      draft: {
        ...draft,
        contacts: draft.contacts.filter((c) => c.id !== contactId),
      },
    });
  },

  setPriority: (category, level) => {
    const { draft } = get();
    if (!draft || !draft.priority_selection) return;
    const newSelection = { ...draft.priority_selection, [category]: level };
    // Auto-sync magic attribute and tradition when magic_or_resonance priority changes
    let newAttrs = draft.attributes;
    let newTradition = draft.magic_tradition;
    if (category === "magic_or_resonance") {
      const startingMagic = SR5_MAGIC_STARTING[level];
      const isTech = draft.magic_tradition === "Technomancer";
      newAttrs = {
        ...draft.attributes,
        magic: !isTech && startingMagic > 0 ? startingMagic : null,
        resonance: isTech && startingMagic > 0 ? startingMagic : null,
      };
      // Reset tradition when switching to Mundane (E)
      if (level === "E") {
        newTradition = null;
      } else if (!draft.magic_tradition) {
        // Default to Magician when first choosing a magic priority
        newTradition = "Magician";
      }
    }
    set({
      draft: {
        ...draft,
        priority_selection: newSelection,
        attributes: newAttrs,
        magic_tradition: newTradition,
      },
    });
  },

  validate: async () => {
    const { draft } = get();
    if (!draft) return;
    try {
      console.log("[personafix] validate_draft calling IPC...");
      const errors = await invoke<ValidationError[]>("validate_draft", {
        draft,
      });
      console.log("[personafix] validation result:", errors.length, "errors");
      set({ validationErrors: errors });
    } catch (err) {
      console.error("[personafix] validate_draft failed:", err);
    }
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
      knowledge_skills: draft.knowledge_skills,
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
      magic_tradition: draft.magic_tradition,
      tradition_name: draft.tradition_name,
      notes: "",
    };

    const computed = await invoke<ComputedCharacter>(
      "save_character_base",
      { base },
    );
    set({ savedCharacter: computed, draft: null });
  },

  listCharacters: async (campaignId) => {
    const chars = await invoke<CharacterSummary[]>("list_characters", {
      campaignId,
    });
    set({ characters: chars });
  },

  loadCharacter: async (id) => {
    const computed = await invoke<ComputedCharacter>("get_character", { id });
    set({ savedCharacter: computed, draft: null });
  },

  applyEvent: async (characterId, event) => {
    const computed = await invoke<ComputedCharacter>("apply_event", {
      characterId,
      event,
    });
    set({ savedCharacter: computed });
  },

  initiate: async (characterId, currentGrade) => {
    const newGrade = currentGrade + 1;
    const karmaCost = 10 + newGrade * 3;
    const event: LedgerEvent = { Initiated: { new_grade: newGrade, karma_cost: karmaCost } };
    const computed = await invoke<ComputedCharacter>("apply_event", { characterId, event });
    set({ savedCharacter: computed });
  },

  submerge: async (characterId, currentGrade) => {
    const newGrade = currentGrade + 1;
    const karmaCost = 10 + newGrade * 3;
    const event: LedgerEvent = { Submerged: { new_grade: newGrade, karma_cost: karmaCost } };
    const computed = await invoke<ComputedCharacter>("apply_event", { characterId, event });
    set({ savedCharacter: computed });
  },

  updateNotes: async (characterId, notes) => {
    await invoke("update_notes", { characterId, notes });
    const { savedCharacter } = get();
    if (savedCharacter) {
      set({
        savedCharacter: {
          ...savedCharacter,
          base: { ...savedCharacter.base, notes },
        },
      });
    }
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
