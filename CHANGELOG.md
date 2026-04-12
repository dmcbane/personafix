# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
