.PHONY: dev build test lint fmt clean install migrate help

DESKTOP_DIR = apps/desktop

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-15s\033[0m %s\n", $$1, $$2}'

install: ## Install all dependencies (Rust + Node)
	cd $(DESKTOP_DIR) && npm install

dev: ## Launch the desktop app in development mode
	cd $(DESKTOP_DIR) && npm run tauri dev

build: ## Build production desktop installers (AppImage, deb, dmg, msi)
	cd $(DESKTOP_DIR) && npm run tauri build

test: ## Run all Rust tests
	cargo test

test-core: ## Run core crate tests only
	cargo test -p personafix-core

test-desktop: ## Run desktop IPC command tests only
	cargo test -p personafix-desktop

lint: ## Run clippy and TypeScript type checking
	cargo clippy -- -D warnings
	cd $(DESKTOP_DIR) && npx tsc --noEmit

fmt: ## Format all Rust and check formatting
	cargo fmt --all

fmt-check: ## Check formatting without modifying files
	cargo fmt --all --check

migrate: ## Run the data migration tool (requires vendor/ repos)
	cargo run --bin personafix-migrate -- \
		--sr5-path vendor/chummer5a/Chummer/data \
		--sr4-path vendor/chummer-sr4/bin/data \
		--output game_data.db

clean: ## Clean all build artifacts
	cargo clean
	rm -rf $(DESKTOP_DIR)/dist $(DESKTOP_DIR)/node_modules
