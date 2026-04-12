# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**personafix** is a ShadowRun character creation and management tool supporting SR4 and SR5 editions. Named after the in-game "personafix" chip. Three-phase roadmap: Desktop (Tauri + React) -> Web (Next.js + Axum) -> Mobile (Tauri Mobile).

## Build & Test Commands

```sh
cargo build                          # Build all crates
cargo test                           # Run all tests
cargo test -p personafix-core        # Test only the core crate
cargo test <test_name>               # Run a single test by name
cargo clippy -- -D warnings          # Lint (treat warnings as errors)
cargo fmt --check                    # Check formatting
cargo fmt                            # Auto-format
cargo run --bin personafix-migrate   # Run the data migration tool
```

To generate TypeScript bindings from Rust types:
```sh
TS_RS_EXPORT_DIR=/absolute/path/to/personafix/packages/types cargo test -p personafix-core
```

## Architecture

Cargo workspace with three crates:

- **`crates/core`** — Pure rules engine. No I/O, no DB, no UI. Must compile to both native and WASM. Contains:
  - `model/` — Pure data types (Character, Attributes, Skills, Qualities, Gear, Magic, Ledger events, etc.)
  - `rules/` — `CharacterRules` trait with `SR4Rules` and `SR5Rules` implementations. All game math lives here.
  - `engine/` — Improvement resolver, modifier stacker, validation runner
  - `ledger/` — Append-only event types and projection logic

- **`crates/data`** — `GameDataRepository` trait + SQLite implementation. SQLite migrations in `crates/data/migrations/`.

- **`crates/migrate`** — One-time XML -> SQLite migration binary for ChummerGenSR4/Chummer5a data files.

TypeScript types are auto-generated from Rust structs via `ts-rs` derive macros into `packages/types/`.

## Key Design Constraints

- **Append-only ledger**: Character state is never mutated in place. All changes are events. Current sheet is a projection. The `ledger` table has triggers preventing UPDATE/DELETE.
- **Edition as a discriminant**: SR4 and SR5 coexist via trait dispatch (`CharacterRules`), not boolean flags.
- **Essence as centessence**: Stored as integer hundredths (600 = 6.00) to avoid floating point.
- **Portable character files**: Each campaign is a `.srx` SQLite file.

## Licensing

Dual-licensed under MIT and Apache 2.0.

## Design Reference

`docs/shadowrun-manager-implementation-plan.docx` contains the full implementation plan (3-phase roadmap, schema, IPC surface, deployment). Binary .docx — open externally to review.
