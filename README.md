# personafix

[![CI](https://github.com/dmcbane/personafix/actions/workflows/ci.yml/badge.svg)](https://github.com/dmcbane/personafix/actions/workflows/ci.yml)

A cross-platform Shadowrun character creation and career management tool supporting both SR4 and SR5 editions.

**[Project Homepage](https://dmcbane.github.io/personafix/)** | **[Downloads](https://github.com/dmcbane/personafix/releases)** | **[Changelog](CHANGELOG.md)**

> A personafix chip in Shadowrun literally overwrites the user's identity and personality. Seems appropriate for a tool that builds your runner from scratch.

## Features

- **Dual edition support** — SR4 (Build Points) and SR5 (Priority System) in the same app, no edition-specific forks
- **Character builder** — interactive attribute, skill, and quality selection with real-time validation and point tracking
- **Append-only career ledger** — every change is an event; the current character sheet is always a projection of creation base + career history
- **Portable character files** — each campaign is a `.srx` SQLite file you can copy, email, or back up
- **Rules engine in pure Rust** — compiles to native (desktop) and WASM (future web), no runtime dependencies
- **Game data migration** — imports SR4 and SR5 data from ChummerGenSR4 and Chummer5a XML files

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) (LTS)
- System libraries for Tauri ([see Tauri prerequisites](https://v2.tauri.app/start/prerequisites/))

### Development

```sh
git clone https://github.com/dmcbane/personafix.git
cd personafix
make install    # Install Node dependencies
make dev        # Launch the desktop app with hot reload
```

### Build Production Installers

```sh
make build
```

Produces native installers in `apps/desktop/src-tauri/target/release/bundle/`:
- **Linux**: `.AppImage` + `.deb`
- **macOS**: `.dmg`
- **Windows**: `.msi` + `.exe`

### Run Tests

```sh
make test       # All 153+ Rust tests
make lint       # Clippy + TypeScript type check
make help       # Show all available commands
```

## Architecture

```
personafix/
  crates/core/          Pure Rust rules engine (SR4 + SR5)
  crates/data/          SQLite data repository + schema migrations
  crates/migrate/       XML -> SQLite game data migration tool
  apps/desktop/         Tauri 2.x desktop app
    src-tauri/            Rust backend (IPC commands)
    src/                  React + TypeScript + Tailwind frontend
```

The **rules engine** (`crates/core`) is the heart of the project. It has no I/O, no database, no UI dependencies — just pure game math behind a `CharacterRules` trait that SR4 and SR5 each implement. This makes it testable, portable, and future-proof for WASM compilation.

The **desktop app** uses [Tauri](https://tauri.app/) to wrap the rules engine with a React frontend. All character mutations flow through a single `apply_event` IPC command that appends to the ledger and re-projects the character state.

For more details, see [CLAUDE.md](CLAUDE.md) (development guide) or the [design document](docs/shadowrun-manager-implementation-plan.docx).

## Roadmap

| Phase | Target | Status |
|-------|--------|--------|
| Phase 1 | Desktop app (Tauri + React) | In progress |
| Phase 2 | Web app (Next.js + Axum + WASM) | Planned |
| Phase 3 | Mobile companion (Tauri Mobile) | Planned |

See the [CHANGELOG](CHANGELOG.md) for version history.

## License

Dual-licensed under [MIT](LICENSE-MIT) and [Apache 2.0](LICENSE-APACHE).
