# Contributing to personafix

Thank you for your interest in contributing to personafix.

## Development Setup

### Prerequisites

- **Rust** stable toolchain via [rustup](https://rustup.rs/)
- **Node.js** LTS via [fnm](https://github.com/Schniz/fnm), [nvm](https://github.com/nvm-sh/nvm), or [direct install](https://nodejs.org/)
- **System libraries** for Tauri — see [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)
  - Ubuntu/Debian: `sudo apt-get install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev`
  - Fedora: `sudo dnf install webkit2gtk4.1-devel gtk3-devel libappindicator-gtk3-devel librsvg2-devel`
  - macOS: Xcode command line tools (`xcode-select --install`)

### Getting Started

```sh
git clone https://github.com/dmcbane/personafix.git
cd personafix
make install    # Install Node dependencies
make test       # Verify everything works
make dev        # Launch the app
```

### Useful Commands

```sh
make help          # Show all available commands
make test          # Run all Rust tests
make test-core     # Run core crate tests only
make test-desktop  # Run desktop IPC tests only
make lint          # Clippy + TypeScript type check
make fmt           # Auto-format Rust code
```

## Code Organization

| Path | Purpose |
|------|---------|
| `crates/core/` | Pure rules engine. No I/O, no DB, no UI. |
| `crates/data/` | SQLite data repository and migrations |
| `crates/migrate/` | One-time Chummer XML -> SQLite tool |
| `apps/desktop/src-tauri/` | Tauri Rust backend (IPC commands) |
| `apps/desktop/src/` | React + TypeScript frontend |

## Development Practices

### Test-Driven Development

Write tests **before** implementing features:

1. **Red**: Write a failing test that describes the desired behavior
2. **Green**: Write the minimal code to make the test pass
3. **Refactor**: Clean up without changing behavior

### Tauri IPC Commands

IPC command logic is extracted into testable `*_db` functions that take `&SqlitePool` directly. The `#[tauri::command]` functions are thin wrappers. This lets us test against in-memory SQLite without the Tauri runtime.

### Error Handling

Every `{:error, _}` or error branch must either log the error, propagate it, or present it to the user. Never silently swallow errors.

## Pull Request Process

1. Create a feature branch from `main`
2. Write tests first, then implement
3. Run `make test && make lint` before pushing
4. Open a PR against `main`
5. PR description should explain **why**, not just **what**

## Releases

Releases are automated via GitHub Actions. To create a release:

```sh
git tag v0.X.0
git push --tags
```

This builds Tauri desktop installers for Linux, macOS, and Windows and creates a draft GitHub Release.

## License

By contributing, you agree that your contributions will be dual-licensed under MIT and Apache 2.0.
