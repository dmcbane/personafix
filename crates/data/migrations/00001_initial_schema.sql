-- Initial schema for personafix Shadowrun character manager.
-- All game data tables have edition and source columns for SR4/SR5 coexistence.

-- Sourcebooks reference table
CREATE TABLE IF NOT EXISTS sourcebooks (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    abbreviation TEXT NOT NULL,
    edition TEXT NOT NULL CHECK (edition IN ('SR4', 'SR5'))
);

-- Metatype definitions with racial attribute limits
CREATE TABLE IF NOT EXISTS metatypes (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    edition TEXT NOT NULL CHECK (edition IN ('SR4', 'SR5')),
    body_min INTEGER NOT NULL,
    body_max INTEGER NOT NULL,
    agility_min INTEGER NOT NULL,
    agility_max INTEGER NOT NULL,
    reaction_min INTEGER NOT NULL,
    reaction_max INTEGER NOT NULL,
    strength_min INTEGER NOT NULL,
    strength_max INTEGER NOT NULL,
    willpower_min INTEGER NOT NULL,
    willpower_max INTEGER NOT NULL,
    logic_min INTEGER NOT NULL,
    logic_max INTEGER NOT NULL,
    intuition_min INTEGER NOT NULL,
    intuition_max INTEGER NOT NULL,
    charisma_min INTEGER NOT NULL,
    charisma_max INTEGER NOT NULL,
    edge_min INTEGER NOT NULL,
    edge_max INTEGER NOT NULL,
    source TEXT NOT NULL,
    page TEXT NOT NULL
);

-- Skill definitions
CREATE TABLE IF NOT EXISTS skills_data (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    linked_attribute TEXT NOT NULL,
    skill_group TEXT,
    edition TEXT NOT NULL CHECK (edition IN ('SR4', 'SR5')),
    source TEXT NOT NULL,
    page TEXT NOT NULL
);

-- Qualities (positive and negative)
CREATE TABLE IF NOT EXISTS qualities (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    quality_type TEXT NOT NULL CHECK (quality_type IN ('Positive', 'Negative')),
    cost INTEGER NOT NULL,
    edition TEXT NOT NULL CHECK (edition IN ('SR4', 'SR5')),
    improvements_json TEXT NOT NULL DEFAULT '[]',
    incompatible_with_json TEXT NOT NULL DEFAULT '[]',
    source TEXT NOT NULL,
    page TEXT NOT NULL
);

-- Weapons
CREATE TABLE IF NOT EXISTS weapons (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    category TEXT NOT NULL,
    damage TEXT NOT NULL,
    ap TEXT NOT NULL DEFAULT '0',
    mode TEXT NOT NULL DEFAULT '',
    recoil_comp INTEGER NOT NULL DEFAULT 0,
    ammo TEXT NOT NULL DEFAULT '',
    availability TEXT NOT NULL,
    cost INTEGER NOT NULL,
    edition TEXT NOT NULL CHECK (edition IN ('SR4', 'SR5')),
    source TEXT NOT NULL,
    page TEXT NOT NULL
);

-- Armor
CREATE TABLE IF NOT EXISTS armor (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    armor_value INTEGER NOT NULL,
    availability TEXT NOT NULL,
    cost INTEGER NOT NULL,
    edition TEXT NOT NULL CHECK (edition IN ('SR4', 'SR5')),
    source TEXT NOT NULL,
    page TEXT NOT NULL
);

-- General gear
CREATE TABLE IF NOT EXISTS gear (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    category TEXT NOT NULL,
    rating INTEGER,
    availability TEXT NOT NULL,
    cost INTEGER NOT NULL,
    edition TEXT NOT NULL CHECK (edition IN ('SR4', 'SR5')),
    source TEXT NOT NULL,
    page TEXT NOT NULL
);

-- Augmentations (cyberware and bioware)
CREATE TABLE IF NOT EXISTS augmentations (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    augmentation_type TEXT NOT NULL CHECK (augmentation_type IN ('Cyberware', 'Bioware')),
    essence_cost INTEGER NOT NULL,
    availability TEXT NOT NULL,
    cost INTEGER NOT NULL,
    improvements_json TEXT NOT NULL DEFAULT '[]',
    edition TEXT NOT NULL CHECK (edition IN ('SR4', 'SR5')),
    source TEXT NOT NULL,
    page TEXT NOT NULL
);

-- Spells
CREATE TABLE IF NOT EXISTS spells (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    category TEXT NOT NULL,
    spell_type TEXT NOT NULL CHECK (spell_type IN ('Physical', 'Mana')),
    range TEXT NOT NULL,
    damage TEXT NOT NULL DEFAULT '',
    duration TEXT NOT NULL,
    drain TEXT NOT NULL,
    edition TEXT NOT NULL CHECK (edition IN ('SR4', 'SR5')),
    source TEXT NOT NULL,
    page TEXT NOT NULL
);

-- Adept powers
CREATE TABLE IF NOT EXISTS adept_powers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    cost INTEGER NOT NULL,
    levels INTEGER NOT NULL DEFAULT 0,
    edition TEXT NOT NULL CHECK (edition IN ('SR4', 'SR5')),
    source TEXT NOT NULL,
    page TEXT NOT NULL
);

-- Complex forms (technomancer)
CREATE TABLE IF NOT EXISTS complex_forms (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    target TEXT NOT NULL,
    duration TEXT NOT NULL,
    fading TEXT NOT NULL,
    edition TEXT NOT NULL CHECK (edition IN ('SR4', 'SR5')),
    source TEXT NOT NULL,
    page TEXT NOT NULL
);

-- Critter powers
CREATE TABLE IF NOT EXISTS critter_powers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    power_type TEXT NOT NULL,
    range TEXT NOT NULL DEFAULT '',
    duration TEXT NOT NULL DEFAULT '',
    edition TEXT NOT NULL CHECK (edition IN ('SR4', 'SR5')),
    source TEXT NOT NULL,
    page TEXT NOT NULL
);

-- Vehicles and drones
CREATE TABLE IF NOT EXISTS vehicles (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    handling TEXT NOT NULL,
    speed INTEGER NOT NULL,
    acceleration INTEGER NOT NULL,
    body INTEGER NOT NULL,
    armor INTEGER NOT NULL,
    pilot INTEGER NOT NULL,
    sensor INTEGER NOT NULL,
    availability TEXT NOT NULL,
    cost INTEGER NOT NULL,
    edition TEXT NOT NULL CHECK (edition IN ('SR4', 'SR5')),
    source TEXT NOT NULL,
    page TEXT NOT NULL
);

-- ============================================================
-- CHARACTER DATA TABLES
-- ============================================================

-- Campaigns: grouping container for characters.
CREATE TABLE IF NOT EXISTS campaigns (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    modified_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Characters: metadata only. The character state comes from base + ledger.
CREATE TABLE IF NOT EXISTS characters (
    id TEXT PRIMARY KEY,
    campaign_id TEXT NOT NULL REFERENCES campaigns(id),
    edition TEXT NOT NULL CHECK (edition IN ('SR4', 'SR5')),
    name TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    modified_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Character base: non-ledgered state (attributes, metatype, priority selections).
CREATE TABLE IF NOT EXISTS character_base (
    character_id TEXT PRIMARY KEY REFERENCES characters(id),
    metatype TEXT NOT NULL,
    attributes_json TEXT NOT NULL,
    skills_json TEXT NOT NULL DEFAULT '[]',
    skill_groups_json TEXT NOT NULL DEFAULT '[]',
    qualities_json TEXT NOT NULL DEFAULT '[]',
    augmentations_json TEXT NOT NULL DEFAULT '[]',
    spells_json TEXT NOT NULL DEFAULT '[]',
    adept_powers_json TEXT NOT NULL DEFAULT '[]',
    complex_forms_json TEXT NOT NULL DEFAULT '[]',
    contacts_json TEXT NOT NULL DEFAULT '[]',
    weapons_json TEXT NOT NULL DEFAULT '[]',
    armor_json TEXT NOT NULL DEFAULT '[]',
    gear_json TEXT NOT NULL DEFAULT '[]',
    vehicles_json TEXT NOT NULL DEFAULT '[]',
    priority_selection_json TEXT
);

-- Ledger: append-only event log. NEVER UPDATE OR DELETE rows.
CREATE TABLE IF NOT EXISTS ledger (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    character_id TEXT NOT NULL REFERENCES characters(id),
    event_type TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    occurred_at TEXT NOT NULL DEFAULT (datetime('now')),
    run_id TEXT
);

-- Trigger to prevent UPDATE on ledger rows.
CREATE TRIGGER IF NOT EXISTS ledger_no_update
    BEFORE UPDATE ON ledger
BEGIN
    SELECT RAISE(ABORT, 'ledger table is append-only: UPDATE is not permitted');
END;

-- Trigger to prevent DELETE on ledger rows.
CREATE TRIGGER IF NOT EXISTS ledger_no_delete
    BEFORE DELETE ON ledger
BEGIN
    SELECT RAISE(ABORT, 'ledger table is append-only: DELETE is not permitted');
END;

-- Index for efficient ledger queries by character.
CREATE INDEX IF NOT EXISTS idx_ledger_character_id ON ledger(character_id);
CREATE INDEX IF NOT EXISTS idx_ledger_run_id ON ledger(run_id);
CREATE INDEX IF NOT EXISTS idx_characters_campaign_id ON characters(campaign_id);
