.PHONY: dev build test test-all test-core test-desktop test-migrate lint fmt clean install migrate help check-deps setup

DESKTOP_DIR = apps/desktop
REQUIRED_TOOLS = cargo rustc node npm

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-15s\033[0m %s\n", $$1, $$2}'

check-deps: ## Verify required toolchains (cargo, rustc, node, npm) are installed
	@missing=""; \
	for tool in $(REQUIRED_TOOLS); do \
		if ! command -v $$tool >/dev/null 2>&1; then \
			missing="$$missing $$tool"; \
		fi; \
	done; \
	if [ -n "$$missing" ]; then \
		echo "Error: missing required tools:$$missing" >&2; \
		echo "" >&2; \
		echo "Run 'make setup' to install, or install manually:" >&2; \
		echo "  Rust (cargo, rustc): https://rustup.rs/" >&2; \
		echo "  Node.js (node, npm): https://nodejs.org/ (or use nvm/fnm)" >&2; \
		exit 1; \
	fi

setup: ## Install missing toolchains (Rust via rustup; Node instructions printed)
	@if ! command -v cargo >/dev/null 2>&1 || ! command -v rustc >/dev/null 2>&1; then \
		echo "Installing Rust via rustup..."; \
		curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable; \
		echo "Rust installed. Restart your shell or run: source \$$HOME/.cargo/env"; \
	else \
		echo "Rust already installed: $$(rustc --version)"; \
	fi
	@if ! command -v node >/dev/null 2>&1 || ! command -v npm >/dev/null 2>&1; then \
		echo ""; \
		echo "Node.js / npm is not installed. Install it via one of:"; \
		echo "  - nvm:   https://github.com/nvm-sh/nvm (recommended)"; \
		echo "  - fnm:   https://github.com/Schniz/fnm"; \
		echo "  - apt:   sudo apt-get install -y nodejs npm"; \
		echo "  - brew:  brew install node"; \
		exit 1; \
	else \
		echo "Node.js already installed: $$(node --version), npm $$(npm --version)"; \
	fi

install: check-deps ## Install all dependencies (Rust + Node)
	cd $(DESKTOP_DIR) && npm install

dev: check-deps ## Launch the desktop app in development mode
	cd $(DESKTOP_DIR) && npm run tauri dev

build: check-deps ## Build production desktop installers (AppImage, deb, dmg, msi)
	cd $(DESKTOP_DIR) && npm run tauri build

test: check-deps ## Run all Rust tests (excludes migration tests that need vendor/ data)
	cargo test

test-all: check-deps ## Run ALL tests including migration tests (requires vendor/ repos)
	cargo test -- --include-ignored

test-core: check-deps ## Run core crate tests only
	cargo test -p personafix-core

test-desktop: check-deps ## Run desktop IPC command tests only
	cargo test -p personafix-desktop

test-migrate: check-deps ## Run migration XML parser tests (requires vendor/ repos)
	cargo test -p personafix-migrate -- --ignored

lint: check-deps ## Run clippy and TypeScript type checking
	cargo clippy -- -D warnings
	cd $(DESKTOP_DIR) && npx tsc --noEmit

fmt: check-deps ## Format all Rust and check formatting
	cargo fmt --all

fmt-check: check-deps ## Check formatting without modifying files
	cargo fmt --all --check

migrate: check-deps ## Run the data migration tool (requires vendor/ repos)
	cargo run --bin personafix-migrate -- \
		--sr5-path vendor/chummer5a/Chummer/data \
		--sr4-path vendor/chummer-sr4/bin/data \
		--output game_data.db

clean: ## Clean all build artifacts
	cargo clean
	rm -rf $(DESKTOP_DIR)/dist $(DESKTOP_DIR)/node_modules
