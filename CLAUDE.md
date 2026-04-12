# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**personafix** is a ShadowRun character creation and management tool supporting SR4 and SR5 editions. Named after the in-game "personafix" chip. Three-phase roadmap: Desktop (Tauri + React) -> Web (Next.js + Axum) -> Mobile (Tauri Mobile).

## Quick Start

```sh
make install    # Install Node dependencies (first time)
make dev        # Launch the desktop app in dev mode
make test       # Run all Rust tests
make lint       # Clippy + TypeScript type check
make build      # Build production installers
make help       # Show all available commands
```

## Build & Test Commands

```sh
cargo build                          # Build all crates
cargo test                           # Run all tests (153+)
cargo test -p personafix-core        # Test only the core crate (121 tests)
cargo test -p personafix-desktop     # Test desktop IPC commands (16 tests)
cargo test -p personafix-migrate     # Test XML parsers (16 tests)
cargo test <test_name>               # Run a single test by name
cargo clippy -- -D warnings          # Lint (treat warnings as errors)
cargo fmt --all --check              # Check formatting
cargo fmt --all                      # Auto-format
```

Desktop app (Tauri):
```sh
cd apps/desktop && npm run tauri dev     # Dev mode with hot reload
cd apps/desktop && npm run tauri build   # Production installers
```

## Architecture

Cargo workspace with four crates:

- **`crates/core`** — Pure rules engine. No I/O, no DB, no UI. Must compile to both native and WASM. Contains:
  - `model/` — Pure data types (Character, Attributes, Skills, Qualities, Gear, Magic, Ledger events, etc.)
  - `rules/` — `CharacterRules` trait with `SR4Rules` and `SR5Rules` implementations. All game math lives here. `sr4_bp.rs` has BP cost helpers, `sr5_priority.rs` has priority table helpers.
  - `engine/` — Improvement resolver, modifier stacker
  - `ledger/` — Append-only event types and projection logic (replays events to compute current state)

- **`crates/data`** — `GameDataRepository` trait + SQLite implementation. SQLite migrations in `crates/data/migrations/`.

- **`crates/migrate`** — One-time XML -> SQLite migration binary for ChummerGenSR4/Chummer5a data files.

- **`apps/desktop/src-tauri`** — Tauri 2.x desktop app. IPC commands in `commands.rs` (testable `*_db` functions + thin Tauri wrappers). State management in `state.rs`. React frontend in `apps/desktop/src/`.

Frontend: React 19 + TypeScript + Vite + Tailwind CSS + Zustand. Character store in `src/store/characterStore.ts`.

## Key Design Constraints

- **Append-only ledger**: Character state is never mutated in place. All changes are events. Current sheet is a projection. The `ledger` table has triggers preventing UPDATE/DELETE.
- **Edition as a discriminant**: SR4 and SR5 coexist via trait dispatch (`CharacterRules`), not boolean flags.
- **Essence as centessence**: Stored as integer hundredths (600 = 6.00) to avoid floating point.
- **Portable character files**: Each campaign is a `.srx` SQLite file.
- **IPC commands are testable**: Core logic extracted into `*_db` functions taking `&SqlitePool`, tested against in-memory SQLite. Tauri `#[command]` functions are thin wrappers.

## CI/CD

- **CI** (`.github/workflows/ci.yml`): Runs on push to main and PRs. Tests, clippy, fmt check, TypeScript type check.
- **Release** (`.github/workflows/release.yml`): Triggered by `v*.*.*` tags. Builds Tauri installers for Linux, macOS, Windows via `tauri-apps/tauri-action`. Creates draft GitHub Release with artifacts.

To release: `git tag v0.8.0 && git push --tags`

## Licensing

Dual-licensed under MIT and Apache 2.0.

## Design Reference

`docs/shadowrun-manager-implementation-plan.docx` contains the full implementation plan (3-phase roadmap, schema, IPC surface, deployment). Binary .docx — open externally to review.
