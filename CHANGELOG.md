# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.50.0] - 2026-06-21

### Changed
- Edition (SR4/SR5) is now a campaign-level setting, not a per-character setting.
  - `campaigns` table gains an `edition` column (migration 00006).
  - `create_campaign` IPC now requires an `edition` argument; it is stored on the campaign row.
  - `Campaign` struct and TypeScript interface both expose `edition`.
  - The edition selector moved from the "New Character" form to the "New Campaign" form on the home screen (toggle buttons instead of a dropdown).
  - When a campaign is created or opened the edition state syncs automatically, triggering game-data reload for the correct ruleset.
  - The campaign character-list screen shows the edition as a badge; character creation inherits it without offering an override.

## [0.49.0] - 2026-06-21

### Added
- Open Campaign dialog now starts in the campaigns storage directory (`~/.local/share/personafix/campaigns/` on Linux) instead of the home root, eliminating the multi-minute freeze on first open.
- New `get_campaigns_dir` IPC command; campaigns directory path is shown on the home screen so users know where files are stored.
- Recent campaigns list now shows the full file path beneath the name so duplicate-named campaigns are distinguishable.
- Per-entry error display: if a recent campaign fails to open, the error appears inline next to that entry (rather than replacing the page-level error).
- Campaign name deduplication: creating a campaign with an existing name auto-appends ` (2)`, ` (3)`, etc. instead of silently overwriting.
- "← Home" button in the campaign character-list screen to return to campaign selection.
- "← Back" button in BuilderShell header to discard the draft and return to the campaign screen.
- `cancelDraft` store action: clears `draft` and `validationErrors`.

## [0.48.3] - 2026-06-21

### Fixed
- SR5 Technomancer: `setPriority` / `setMagicTradition` now correctly routes the
  starting value to `attributes.resonance` (not `magic`) when tradition is Technomancer;
  added `setResonance` store action; AttributePanel magic/resonance slider now shows
  "RES" label and calls `setResonance` for Technomancers.
- SR5 Technomancer: SummaryBar SAP calculation now reads `attributes.resonance` instead
  of `attributes.magic` when computing points spent above starting value.
- SR5 quality karma limit: SummaryBar now checks positive and negative pools separately
  (both must stay ≤ 25k) instead of `|net| > 25` which missed balanced-but-over-limit builds.
- SR5 priority table: `SR5_MAGIC_STARTING["B"]` corrected from 6 to 4 (SR5 CRB p. 65).

## [0.48.1] - 2026-06-21

### Fixed
- Tauri error objects now display the human-readable `.message` field instead of `[object Object]`
  - Added `fmtErr()` helper in `src/utils.ts`; replaced all `String(err)` catch clauses in `App.tsx` and `SavedCharacterView.tsx`
  - Fixed dead-code warning: `open_campaign` Tauri wrapper now delegates to `open_campaign_db` helper

## [0.48.0] - 2026-06-21

### Added
- P16: Play Mode — full-screen `PlayView` with condition monitors, wound modifier, initiative roller, edge tracker, and dice roller
  - `DiceRoller.tsx`: reusable `rollPool()`, `DieIcon`, `DiceRollerPanel`
  - "⚔ Play" button in character sheet header; "← Sheet" returns
- P15: Print sheet now includes Gear and Notes sections
- P14: Chummer 5 import/export
  - New `crates/import-export` crate: parses `.chum5` XML via `quick-xml`/serde, maps to `CharacterBase`
  - `import_chummer_character` / `export_chummer_character` IPC commands
  - "Import Chummer…" button in campaign character list; "↓ Chummer" export in character sheet header
- P13: Campaign persistence layer
  - "Open…" file-picker button to open existing `.srx` campaign files
  - Recent campaigns list (LRU JSON at `{data_dir}/personafix/recent_campaigns.json`)
  - JSON character backup/restore: `export_character_json` / `import_character_json` IPC + UI buttons

## [0.45.0] - 2026-06-20

### Added
- P12-4: Specializations on existing skills — add/display per skill.
  - `addSpecialization` / `removeSpecialization` store actions
  - L2 roundtrip test: specializations survive save/load (skills_json roundtrip)
  - SkillPanel: inline add-specialization input + Enter/button per skill row (capped at 1 per skill); chips with remove button
  - SavedCharacterView: "Active Skills" SheetSection shows skills with specialization chips (↳ name +2)
  - Print sheet shows specializations as indented sub-rows under each skill

## [0.44.0] - 2026-06-20

### Added
- P12-3: Effective armor display with edition-specific stacking rules.
  - `effective_armor(ratings)` method on `CharacterRules` trait; SR4 = full sum, SR5 = highest + floor(each-additional / 2)
  - `effective_armor: u32` field on `ComputedCharacter`, computed in both SR4 and SR5 `apply_improvements`
  - L1 tests for both stacking rules (sr4 full-sum, sr5 stacking)
  - SavedCharacterView Derived Stats shows effective armor when armor is equipped; "(stacked)" label for SR5 multi-armor
  - Print sheet shows Effective Armor in derived stats block

## [0.43.0] - 2026-06-20

### Added
- P12-2: Initiation & Submersion for career Awakened/Emerged characters.
  - `initiation_karma_cost(grade)` and `submersion_karma_cost(grade)` default methods on `CharacterRules` trait (formula: 10 + grade × 3)
  - `initiation_grade: u8` and `submersion_grade: u8` on `ComputedCharacter`
  - Projection tracks grades from `Initiated`/`Submerged` ledger events
  - `initiate` and `submerge` store actions
  - `Initiated` and `Submerged` variants added to TypeScript `LedgerEvent` type
  - SavedCharacterView: Initiate/Submerge buttons in Karma Improvements panel (shows for Awakened traditions and Technomancers)
  - Grade displayed in Derived Stats block when > 0
  - `formatEvent` handles Initiated/Submerged in ledger timeline

## [0.42.0] - 2026-06-20

### Added
- P12-1: Magician tradition sub-selection for SR5 Magicians and Mystic Adepts.
  - `tradition_name: Option<String>` field on `CharacterDraft` and `CharacterBase`
  - DB migration `00005_tradition_name.sql` (nullable TEXT column)
  - `update_tradition_name` Tauri IPC command + `update_tradition_name_db` testable core
  - L2 roundtrip test `test_tradition_name_roundtrip` verifies save/load/update/clear
  - MagicPanel shows Hermetic / Shaman / Other toggle buttons for SR5 Magician/MysticAdept
  - SavedCharacterView header and print sheet display tradition alongside archetype
  - `setTraditionName` action in `characterStore`

## [0.41.0] - 2026-06-20

### Added
- P11-4: Bioware essence halving audit confirmed — no code fix needed.
- `docs/dev-journal/2026-06-20-bioware-essence-halving.md` explains that Chummer's bioware
  XML pre-halves essence costs at the source (Cat's Eyes `<ess>0.1</ess>` is already
  the effective 0.1E), so the grade-multiplier-only engine is correct for both
  cyberware and bioware.
- `docs/sr5-parity-checklist.md` updated: bioware essence halving marked verified.
- Bump 0.40.0 → 0.41.0; P11-4 done. All P11 items complete.

## [0.40.0] - 2026-06-20

### Added
- P11-3: Contact karma improvement in career view.
- `handleContactImprove` in SavedCharacterView: applies `ContactChanged` + `KarmaSpent` events.
- Contacts section in Karma Improvements panel: +C and +L buttons per contact (cost = new rating).
- Buttons disabled when not enough karma or at cap (6).
- `ContactChanged`, `ContactAdded`, `ContactLost` variants added to `LedgerEvent` TypeScript type.
- `formatEvent` updated to display contact change/add/lost events in career timeline.
- Bump 0.39.0 → 0.40.0; P11-3 done.

## [0.39.0] - 2026-06-20

### Added
- P11-2: Character notes — free-text backstory/run-log field per character.
- DB migration `00004_notes.sql` adds `notes TEXT NOT NULL DEFAULT ''` to `character_base`.
- `notes: String` field on `CharacterBase` (with `#[serde(default)]`).
- `update_notes` IPC command + `update_notes_db` function for efficient note-only saves.
- TypeScript: `notes` on `ComputedCharacter.base`, `updateNotes` store action.
- SavedCharacterView: "// Character Notes" collapsible section with textarea + Save button.
- Bump 0.38.0 → 0.39.0; P11-2 done.

## [0.38.0] - 2026-06-20

### Added
- P11-1: Knowledge/language skills — free-form player-defined skills added to model.
- `KnowledgeSkill` struct in `crates/core/src/model/skills.rs` (name, category, rating).
- `knowledge_skills: Vec<KnowledgeSkill>` on `CharacterDraft` and `CharacterBase`.
- DB migration `00003_knowledge_skills.sql` adding `knowledge_skills_json` column.
- IPC: `save_character_base_db`/`get_character_db` updated to persist/load knowledge skills.
- TypeScript: `KnowledgeSkill` interface, `KNOWLEDGE_SKILL_CATEGORIES` constant, store actions.
- SkillPanel: Knowledge & Language Skills sub-section with name input, category select, rating stepper.
- SavedCharacterView: knowledge skills section in character sheet and print layout.
- Bump 0.37.0 → 0.38.0; P11-1 done.

## [0.37.0] - 2026-06-19

### Added
- P10-4: Print/export character sheet.
- "⎙ Print" button in saved character sheet header triggers `window.print()`.
- Hidden `#print-sheet` div rendered alongside the main view; hidden on screen,
  revealed by `@media print` CSS rule which hides all other body content.
- Print layout includes: attributes grid, derived stats, qualities, augmentations,
  skills, spells/powers/forms (by tradition), contacts, weapons, armor, vehicles.
- `@media print` rules in `index.css` with `attr-box`, `item-row`, `grid-2` classes.
- Bump 0.36.0 → 0.37.0; P10-4 done.

## [0.36.0] - 2026-06-19

### Added
- P10-3: Vehicles migration and builder panel.
- SR4 (483 vehicles) and SR5 (380 vehicles) parsed from `vehicles.xml` via
  `ChummerVehicles`/`XmlVehicle`/`ParsedVehicle` types; seeded into `vehicles` table.
- `get_vehicles` IPC command; `query_vehicles_db` in commands.rs.
- `GameVehicle` in gameDataStore; loaded in parallel on startup alongside other data.
- `DraftVehicle` in characterStore; `addVehicle`/`removeVehicle` actions; 
  `CharacterDraft.vehicles` typed as `DraftVehicle[]` (was `unknown[]`).
- GearPanel extended with "Vehicles" section tab (alongside Weapons/Armor).
- SavedCharacterView: Vehicles section in character sheet; `base.vehicles` typed.
- L3a smoke test extended: asserts vehicles non-empty from real game_data.db.
- SR5 parity checklist updated: vehicles checked.
- Bump 0.35.0 → 0.36.0

## [0.35.0] - 2026-06-19

### Added
- P10-2: `docs/sr5-parity-checklist.md` — SR5-specific parity checklist covering
  priority table, awakened subtypes, contacts, skills, qualities, augmentations,
  weapons/armor, complex forms, career play, and character sheet. Items checked/unchecked
  based on current implementation state.
- Bump 0.34.0 → 0.35.0

## [0.34.0] - 2026-06-19

### Added
- P10-1: Dice roller integrated into saved character sheet (collapsible section).
- SR4/SR5 dice pool mechanic: roll N d6, each 5/6 is a hit; die faces shown color-coded
  (green = hit, red = 1, gray = miss); hit count prominent; glitch and critical glitch
  detection (>half dice are 1s = glitch; glitch + 0 hits = critical glitch).
- Quick Roll shortcuts: each skill pool (rating + linked attribute) shown as a button;
  clicking pre-fills pool size and immediately rolls.
- Edge Roll button: adds the character's edge attribute to the current pool.
- Roll history: last 10 rolls shown most-recent-first.
- Backlog extended with P10-1 (done), P10-2 SR5 parity checklist, P10-3 vehicles, P10-4 print.
- Bump 0.33.0 → 0.34.0

## [0.33.0] - 2026-06-19

### Fixed
- P9-1: `DraftAdeptPower.cost` was typed as `string` in TypeScript but `i32` in Rust;
  serde would reject the IPC payload on any save with adept powers. Fixed by storing cost
  as integer centessences (`number`, e.g. 25 for 0.25 PP) matching the Rust type. Display
  converts with `ppFromCentessence()`. `handleAddPower` converts from GameAdeptPower's
  decimal string via `Math.round(parseFloat(gp.cost) * 100)`.

### Added
- P9-1: `ComputedCharacter.base` now exposes all `CharacterBase` fields: `qualities`,
  `augmentations`, `spells`, `adept_powers`, `complex_forms`, `contacts`, `weapons`,
  `armor`, `magic_tradition`. Previously only `skills` was typed, causing the sheet view
  to show a blank equipment section.
- P9-1: `SavedCharacterView` now shows collapsible "// Character Sheet" section with
  Qualities (positive/negative), Augmentations (type badge, grade, essence), Spells
  (category, type, drain), Adept Powers (PP cost), Complex Forms (target, duration,
  fading), Contacts (connection/loyalty + archetype), Weapons (category, damage), and
  Armor (armor value). Section only rendered for the character's tradition.
- P9-1: Nuyen Spend button in the Career section (alongside Karma and Nuyen Received).
  Guards against spending more than available balance.
- SR4 parity checklist updated to reflect phases 1–5 completion.

## [0.32.0] - 2026-06-19

### Added
- P8-1: SR5 complex forms panel for Technomancers.
- Migration: parses `complexforms.xml` (38 SR5 forms); `ParsedComplexForm`,
  `ChummerComplexForms`, `XmlComplexForm` types added; `parse_complex_forms()` in sr5.rs.
- DB seeder seeds `complex_forms` table with id/name/target/duration/fading/edition.
- `GameComplexForm`, `query_complex_forms_db`, `get_complex_forms` IPC command.
- `gameDataStore`: `complexForms` state, loaded alongside spells/powers on startup.
- `characterStore`: `DraftComplexForm` type; `addComplexForm`/`removeComplexForm` actions.
- `MagicPanel`: Technomancers see a real complex form browser with search + dropdown +
  Resonance-rating cap tracker; fading value shown per form.
- L3a smoke test extended: asserts complex_forms non-empty and all fading values present.
- Bump 0.31.0 → 0.32.0; P8-1 done.

## [0.31.0] - 2026-06-19

### Added
- P7-1: Accessibility settings panel — color theme switcher and font selector.
- Five color themes: Cyber Dark (default), High Contrast Dark, Colorblind R/G safe
  (Deuteranopia/Protanopia), Colorblind B/Y safe (Tritanopia), Light mode.
- Themes implemented via CSS custom properties (RGB channel format) on `[data-theme]`
  so Tailwind `/NN` opacity modifiers work correctly at runtime.
- OpenDyslexic font bundled from system installation (SIL OFL) in `public/fonts/`;
  `@font-face` declarations in `index.css`.
- Font selector: Fira Code mono (default), System sans-serif, OpenDyslexic.
- Font size stepper: Small (13px), Normal (15px), Large (17px), X-Large (20px).
- `settingsStore.ts` (Zustand persist → localStorage key `personafix-settings`).
- `SettingsPanel.tsx` modal with theme buttons, font radio, size stepper.
- Settings gear button in home screen (top-right) and builder header.
- `index.css`: all hardcoded hex colors migrated to CSS variables; scrollbar,
  select arrow, focus ring, and inputs all theme-aware.
- Fixed pre-existing clippy `type_complexity` lint in `commands.rs` (SpellRow type alias).
- Bump 0.30.0 → 0.31.0; P7-1 done.

## [0.30.0] - 2026-06-19

### Added
- P6-5: SR5 adept powers panel. Parses `powers.xml` from chummer5a (109 SR5 powers);
  seeds `adept_powers` table in `game_data.db`; `get_adept_powers` IPC command.
- `DraftAdeptPower` type in characterStore; `addAdeptPower`/`removeAdeptPower` actions.
- `GameAdeptPower` type in gameDataStore; loaded alongside spells on startup.
- MagicPanel: Adept/MysticAdept characters see a real power browser with search +
  dropdown + pool tracker (PP spent/total = Magic rating); zero-cost and leveled powers
  supported.
- L3a smoke test extended: asserts adept powers non-empty and all costs parse as
  non-negative decimals.
- Bump 0.29.0 → 0.30.0; P6-5 done.

## [0.29.0] - 2026-06-19

### Added
- P6-4: SR5 contact free pool (Charisma × 3). SR5 validation warns when total
  contact points (sum of connection + loyalty) exceed free pool; overflow flagged
  as a Warning (not Error) with message showing karma cost
  (L1 tests: validate_contacts_over_free_pool_flagged_as_warning,
   validate_contacts_within_free_pool_is_clean)
- ContactPanel: SR5 mode shows "X/Y free (CHA × 3)" + "+N karma overflow" in yellow
  when over pool; SR4 mode unchanged (BP per point)
- Bump 0.28.0 → 0.29.0; P6-4 done

## [0.28.0] - 2026-06-19

### Added
- P6-3: SR5 special attribute pool. `special_attribute_points(metatype_priority) -> i32`
  in sr5_priority.rs (A=13, B=11, C=9, D=4, E=1)
- SR5 validation checks that magic above starting + edge above racial min ≤ special pool;
  error on "special_attribute_pool" field (L1 tests: overspent_errors + within_budget_passes)
- SR5 magic slider in AttributePanel now allows values up to racial max (6) above the
  starting value; shows SAP used in yellow/red next to the slider
- AttributePanel shows special pool summary "(used/total) includes Edge above racial min"
- SummaryBar shows "SAP: X/Y" chip for SR5 drafts, turns red when over budget
- `SR5_SPECIAL_ATTR_POINTS` constant exported from characterStore
- Updated `validate_magic_over_priority_max_errors` test to `validate_magic_over_racial_max_errors`:
  now tests magic > 6 (true racial cap) rather than magic > starting value
- Bump 0.27.0 → 0.28.0; P6-3 done

## [0.27.0] - 2026-06-19

### Added
- P6-2: SR5 Awakened subtype enforcement. `MagicTradition` enum (Magician, Adept,
  MysticAdept, Technomancer) added to Rust model (`crates/core`) and stored in
  `character_base.magic_tradition` SQLite column (migration 00002)
- `MagicTradition` field on `CharacterDraft` and `CharacterBase` (serde-defaulted
  for backwards compatibility)
- SR5 validation: non-Mundane magic priority requires `magic_tradition` to be set
  (L1 test: `validate_magic_tradition_required_when_awakened`)
- PriorityPanel: tradition selector appears when magic priority ≠ E; shows
  Magician / Adept / Mystic Adept / Technomancer buttons with descriptions
- MagicPanel: gates spell list to Magician/MysticAdept; shows Adept Powers
  placeholder (with power point budget) for Adept/MysticAdept; shows Complex Forms
  placeholder for Technomancer; Mundane SR5 characters see a "no magic" message
- `setMagicTradition` action in character store
- `setPriority` auto-clears tradition to null when magic priority switches to E;
  auto-sets "Magician" as default when first choosing a non-E magic priority
- DB save/load paths updated to persist `magic_tradition`
- Bump 0.26.0 → 0.27.0; P6-2 done

## [0.26.0] - 2026-06-19

### Changed
- Panel layout consistency (P6-1): Qualities, Augmentations, Gear, and Magic panels
  now share the Skills panel UX pattern — filters on top, search + drop-down select +
  Add button in the middle, installed/selected list at the bottom
- **QualityPanel**: type filter chips → search + select (shows +/- and cost) → Add → list
- **AugmentationPanel**: type filter chips → grade chips + rating input → search + select
  (shows essence cost at chosen grade) → Add → installed list; essence preview shown
  next to rating for the currently-selected aug
- **GearPanel**: section tabs (Weapons/Armor) → category filter chips (weapons) →
  search + select (shows damage/mode for weapons, armor value for armor) → Add → list
- **MagicPanel**: category filter chips → type filter (Physical/Mana) → search + select
  (shows category/type/drain) → Add → known spells list; spell count now shows X/Magic
  and turns red if over the Magic-rating cap
- Bump 0.25.0 → 0.26.0; P6-1 done

## [0.25.0] - 2026-06-19

### Fixed
- Gear now tracks nuyen in `nuyen_spent`: `addWeapon`/`addArmor` increment
  `nuyen_spent` by item cost; `removeWeapon`/`removeArmor` decrement it. This
  makes SR5 nuyen budget enforcement and SR4 resource-BP enforcement live in the UI.
- SummaryBar SR4 now includes resource BP (`nuyen_spent / 5000`) in the total BP
  counter, with a "Res: N" breakdown chip — previously gear cost was invisible to
  the budget display
- ContactPanel: "BP" labels now read "karma" when the draft is SR5
- MagicPanel: "max: Magic rating in SR4" → "max: Magic rating" (edition-agnostic)
- Bump 0.24.0 → 0.25.0

## [0.24.0] - 2026-06-19

### Added
- SR5 magic/resonance validation (TDD): `validate_creation` now errors when a non-Mundane
  magic priority is chosen but `magic`/`resonance` attribute is `None`, and when the magic
  rating exceeds the priority maximum; 3 new tests (2 error cases + mundane regression)
- `magic_starting_rating(level)` helper in `sr5_priority.rs` returns `Some(6/3/2)` for
  A/B/C/D and `None` for E (Mundane)
- SR5 budget display in **SummaryBar**: Attr X/Y, Skills X/Y, Qual ±Xk, Nuyen ¥X/Y —
  numbers turn red when over-budget; budget derived from priority selection live
- SR5 attribute point budget display in **AttributePanel**: "X of Y attr points used"
  (Edge excluded — comes from metatype special pool per SR5 rules)
- SR5 skill point budget in **SkillPanel**: "Skills: X/Y | Groups: 0/Y"
- **Magic slider in AttributePanel** for SR5: shown when magic_or_resonance priority ≠ E;
  capped at priority maximum (6/6/3/2); "Mundane" note shown when E
- `SR5_ATTR_POINTS`, `SR5_SKILL_POINTS`, `SR5_RESOURCE_NUYEN`, `SR5_MAGIC_STARTING`
  exported from `characterStore` (mirror Rust `sr5_priority.rs` constants exactly)
- `setMagic` store action for the optional magic attribute
- **Auto-sync magic on priority change**: `setPriority(magic_or_resonance, level)` now
  sets `draft.attributes.magic` to the starting rating for that level (null if Mundane)
- **Draft initialization**: SR5 character starts with `magic = SR5_MAGIC_STARTING["C"]`
  (default priority C = Magician 3) instead of null, so the default draft is valid
- Bump 0.23.0 → 0.24.0; SR5 end-to-end verification phase begun

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
