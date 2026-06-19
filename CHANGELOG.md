# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.23.0] - 2026-06-19

### Added
- Career Timeline panel (P5-3): collapsible ledger view in `SavedCharacterView`
  loads `get_ledger` on expand and live-reloads after each applied event
- `formatEvent` renders each `LedgerEvent` variant as a labeled row with color:
  green for income (karma/nuyen received), red for spending, blue for improvements
- Unknown event types fall back to raw JSON so no events are silently dropped
- Bump 0.22.0 → 0.23.0; P5-3 done

## [0.22.0] - 2026-06-19

### Added
- Career event UI (P5-1 + P5-2): `SavedCharacterView` gains "Apply Event" controls
  for Karma Received and Nuyen Received; karma/nuyen totals update live after each event
- Karma Improvements panel (P5-2): collapsible section shows all 9 attributes with karma
  cost (+1 costs new_rating × 5) and all character skills with karma cost (new_rating × 2);
  buttons disabled when not enough karma available
- `applyEvent` action in `characterStore` calls `apply_event` IPC and updates
  `savedCharacter` with the returned `ComputedCharacter`
- `LedgerEvent` TypeScript union: KarmaReceived, KarmaSpent, NuyenReceived, NuyenSpent,
  SkillImproved, AttributeImproved (externally-tagged serde format)
- `ComputedCharacter.base.skills` and `base.attributes` added to TS interface so the
  improvement panel can show current ratings
- Bump 0.21.0 → 0.22.0; P5-1, P5-2 done; P5-3 doing

## [0.21.0] - 2026-06-19

### Added
- Gear/Weapons/Armor panel (P3-3): GearPanel.tsx with Weapons (742 SR4) and
  Armor (192 SR4) sections; searchable, weapons filterable by category; equipped
  items shown at top with remove button
- `get_armor` IPC command + `GameArmor` struct; armor loaded alongside other
  game data in `loadGameData`
- `DraftWeapon`, `DraftArmor` typed in characterStore; `addWeapon`, `removeWeapon`,
  `addArmor`, `removeArmor` store actions; `weapons`/`armor` arrays typed (were
  `unknown[]`)

## [0.20.0] - 2026-06-19

### Added
- Augmentations panel (P3-2): browse 436 SR4 cyberware/bioware items by type and
  name; choose grade (Std/α/β/δ/Used) and rating before install; essence cost
  shown per item and as running total; removal supported
- Magic panel (P3-5): browse 253 SR4 spells by category and type (Physical/Mana);
  warns when Magic attribute is 0; only known-category spells shown so Rust
  deserialization succeeds on character save
- `get_spells` IPC command + `query_spells_db` helper; `GameSpell` struct; `spells`
  loaded in `loadGameData` alongside existing game data types
- `DraftAugmentation`, `DraftSpell`, `AugmentationGrade`, `AugmentationType`,
  `GRADE_MULTIPLIER` added to `characterStore`; `augmentations`/`spells` typed
  (were `unknown[]`)
- SummaryBar essence display now computes remaining essence from installed
  augmentations using the Rust-matching formula
  `adjusted = floor(essence_cost * grade_multiplier / 100)`; turns red at ≤ 0
- Deleted dead `crates/data/src/sqlite.rs` + `pub mod sqlite` (P2-1): 11 `todo!()`
  stubs removed; `GameDataRepository` trait kept for future web backend
- Bump 0.19.0 → 0.20.0; mark P2-1, P3-2, P3-5 done

## [0.19.0] - 2026-06-19

### Changed
- Skills and Qualities panels now use real game_data.db data automatically (P3-1):
  game data loads on app mount and whenever the edition selector changes — no
  manual "Load Game Data" click required
- `handleStartBuilder` always loads game data before opening the builder (was
  conditional on it already being loaded, which left the builder on seed data if
  "Load Game Data" had never been clicked)
- Game data status in the new-character form is now a read-only status line
  ("Game data loaded" / "using built-in data") rather than a manual action panel;
  the path override and retry controls are collapsed into a `<details>` element
  shown only when the DB isn't found
- **Gate note:** L4 required — run `make dev`, open builder, confirm skill and
  quality lists have 78 / 483 SR4 entries instead of the ~10 seed entries

## [0.18.0] - 2026-06-19

### Added
- Game data pipeline (P1-1 + P1-2): both Chummer vendor repos were present;
  `make migrate` now produces `game_data.db` from SR4 (78 skills, 483 qualities,
  743 weapons, 192 armor, 439 augmentations, 253 spells) and SR5 data
- `game_data.db` added to `.gitignore` (contains copyrighted Shadowrun content;
  not redistributed — decision documented in `docs/dev-journal/2026-06-19-game-data-licensing.md`)
- L3a data-smoke test (`data_smoke_real_game_data_db`) un-ignored; passes green
  against the generated DB

## [0.17.0] - 2026-06-19

### Added
- Contacts panel (P3-4): tabbed "Contacts" view in BuilderShell lets you add
  contacts with name, archetype (Fixer, Street Doc, Decker, …), connection (1–6),
  and loyalty (1–6); remove button on each row
- `Contact` interface added to `characterStore.ts`; `contacts: unknown[]` typed as
  `Contact[]`; `addContact` / `removeContact` store actions
- SummaryBar now includes contact BP in the total (1 per connection + loyalty point)
  and shows a `Contacts: N` breakdown chip when contacts are present
- **Gate note:** L4 required — run `make dev`, add contacts, confirm BP counter updates

## [0.16.0] - 2026-06-19

### Added
- Character list / load screen (P3-6): after creating a campaign, existing
  characters are listed with name, edition, and metatype; each has an **Open**
  button that loads the character sheet via `get_character` IPC
- `listCharacters` and `loadCharacter` actions added to `characterStore`
- List auto-refreshes whenever returning to the campaign screen (after save or
  from SavedCharacterView), so newly saved characters appear immediately
- **← Characters** button in `SavedCharacterView` replaces "New Character"
  label — behavior unchanged (`reset()`) but now clearly navigates back to the
  list where you can create a new one or reopen another
- **Gate note:** L4 still required — run `make dev`, save two characters, use
  Open to reopen one

## [0.15.1] - 2026-06-19

### Fixed
- Incompatible quality warning message was inconsistent: first quality used
  display name, second used raw ID (e.g. "Lucky is incompatible with bad_luck"
  instead of "Lucky is incompatible with Bad Luck"); now looks up the
  conflicting quality's display name from the draft, falling back to ID if not
  found

## [0.15.0] - 2026-06-19

### Fixed
- Conflict detection end-to-end: `incompatible_with` was silently dropped at
  every layer between the DB and the UI; the warning rendered as empty
  - `query_qualities_db` now selects `incompatible_with_json` and deserializes
    it into `GameQuality.incompatible_with: Vec<String>`
  - `GameQuality` in `gameDataStore.ts` gains `incompatible_with: string[]`
  - `QualityPanel` now passes `gq.incompatible_with` through when mapping game
    data (previously hardcoded `[]`)
  - Fallback quality list adds **Lucky** (Positive, 20 BP, incompatible with
    Bad Luck) and wires `Bad Luck ↔ Lucky` mutual incompatibility — so the
    warning is testable without game_data.db

## [0.14.0] - 2026-06-19

### Added
- Conflict detection surfaced in the builder UI (P4-4):
  - SummaryBar now renders `Warning` items in yellow (`#f9c74f`) below `Error`
    items in red — each showing `[warn] [field] message` — so incompatible
    qualities are visible without blocking character creation
  - Qualities tab badge shows `~` in yellow when any incompatibility warning
    is present, alongside the existing `!` in red for hard errors
  - Added `cyber-yellow` / `cyber-yellow-dim` to the Tailwind color palette
- **Gate note:** L4 (human spot-check via `make dev`) still required before
  this checklist item is fully closed

## [0.13.0] - 2026-06-19

### Added
- L3a data-smoke test scaffold (`data_smoke_real_game_data_db` in `commands.rs`):
  verifies that `query_skills_db` and `query_qualities_db` return non-empty,
  well-formed SR4 rows from a real `game_data.db`; ignored until backlog P1-1
  produces the file; accepts `GAME_DATA_DB` env var to override the default path

## [0.12.0] - 2026-06-19

### Added
- SR4 conflict detection (P4-3): `SR4Rules::validate_creation` now reports an
  `Error` when both `magic` and `resonance` attributes are set simultaneously —
  a character cannot be both Awakened and a technomancer in SR4

## [0.11.0] - 2026-06-19

### Added
- SR4 conflict detection (P4-2): `SR4Rules::validate_creation` now reports an
  `Error` when augmentations collectively exceed 6.00 essence (remaining <= 0),
  reusing `calculate_essence` to guarantee consistency with `apply_improvements`

## [0.10.0] - 2026-06-19

### Added
- SR4 conflict detection (P4-1): `SR4Rules::validate_creation` now reports mutually
  incompatible qualities as `ValidationSeverity::Warning` using the existing
  `Quality::incompatible_with` field — conflicts are surfaced without blocking creation

## [0.9.0] - 2026-06-19

### Added
- Autonomous iteration harness scaffolding: SR4 parity checklist, ordered backlog,
  conflict-detection TDD seed (3 ignored tests), and deterministic driver workflow script
- Dev journal entry documenting the harness design and the dead-code correction for
  `SqliteGameDataRepository`

## [0.8.0] - 2026-04-12

### Added
- Character save flow: validate via rules engine, persist to campaign DB, display computed character sheet
- SR5 Priority selection panel with interactive table and smart swap
- SavedCharacterView showing attributes, derived stats, and career totals
- Developer Makefile with `make dev`, `make build`, `make test`, etc.
- GitHub Actions CI pipeline (test, clippy, fmt, TypeScript on push/PR)
- GitHub Actions release pipeline (build installers for Linux/macOS/Windows on version tags)
- Updated CLAUDE.md with Quick Start and CI/CD documentation

## [0.7.0] - 2026-04-12

### Added
- Character builder UI: BuilderShell with tabbed Attributes/Skills/Qualities panels
- Zustand characterStore managing draft state with IPC validation
- AttributePanel: +/- controls with racial min/max bounds, BP cost display
- SkillPanel: add from common list, adjust ratings 1-6
- QualityPanel: positive/negative with BP tracking, filter by type
- SummaryBar: persistent BP breakdown, essence, validation errors
- IPC commands: `get_racial_limits`, `validate_draft`, `save_character_base`
- 5 additional desktop IPC tests

## [0.6.0] - 2026-04-12

### Added
- Tauri 2.x desktop application shell at `apps/desktop/`
- IPC command surface: `create_campaign`, `open_campaign`, `list_characters`, `create_character`, `get_character`, `apply_event`, `get_ledger`
- React frontend skeleton with Vite + TypeScript + Tailwind CSS
- 9 IPC command integration tests against in-memory SQLite

### Fixed
- Migration path resolution from `apps/desktop/src-tauri/` depth
- `list_characters` query now JOINs `character_base` for metatype data

## [0.5.0] - 2026-04-12

### Added
- Ledger projection: replays career events against CharacterBase to produce ComputedCharacter
- Handles all event types: karma, nuyen, skill/attribute improvements, gear, contacts, initiation, qualities
- Edition-agnostic projection via `&dyn CharacterRules`
- Full career test: 3 runs with karma/nuyen rewards, improvements, gear purchases

## [0.4.0] - 2026-04-12

### Added
- SR5 rules engine with Priority creation system
- Priority table constants and validation (each level A-E used exactly once)
- SR5 racial limits for all 5 metatypes
- SR5 karma costs (same formulas as SR4)
- SR5 creation validation: priority selection, attribute/skill point budgets, quality limits (25 karma)
- Canonical SR5 Adept test (priority-legal build with Magic 6)

## [0.3.0] - 2026-04-12

### Added
- SR4 rules engine with Build Point creation system
- BP calculation: attributes (10/point), skills (4/point), groups (10/point), qualities, resources (5000 nuyen/BP), contacts
- Karma advancement costs: skill (new_rating x 2), attribute (new_rating x 5)
- Essence calculation with grade multipliers (Standard/Alpha/Beta/Delta/Used)
- SR4 racial limits for all 5 metatypes
- Creation validation: 400 BP budget, attribute bounds, skill caps, quality limits (35 BP), resource cap
- Improvement resolver and modifier stacker in engine modules
- Canonical SR4 Street Samurai test

## [0.2.0] - 2026-04-12

### Added
- Data migration tool (`personafix-migrate`) converting Chummer XML to SQLite
- SR5 parser for Chummer5a data: sourcebooks, metatypes, skills, qualities, weapons, armor, augmentations, spells
- SR4 parser for ChummerGenSR4 data with UUID generation for entries lacking IDs
- DB seed function with transactional bulk inserts
- Schema updated: cost/essence/availability fields are TEXT (Chummer uses formulas)
- 16 parser tests with spot-checks against real Chummer data

## [0.1.0] - 2026-04-11

### Added
- Initial Cargo workspace with `crates/core`, `crates/data`, `crates/migrate`
- Core model types: Character, Attributes, Skills, Qualities, Gear, Magic, Augmentations, Contacts, Priority
- `CharacterRules` trait with empty `SR4Rules` and `SR5Rules` implementations
- Ledger event types (KarmaReceived, GearAcquired, SkillImproved, etc.)
- SQLite schema: game data tables, character/campaign tables, append-only ledger with UPDATE/DELETE triggers
- `GameDataRepository` trait with SQLite implementation stub
- TypeScript type generation via ts-rs
