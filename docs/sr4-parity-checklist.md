# SR4 Parity Checklist — Definition of Done

This is the **termination condition** for the autonomous harness. The loop stops when
every item here is satisfied at its stated gate, and you have signed off the L4 (human)
checks per phase. Each item names the *verification rung* that proves it (see the
verification ladder in the harness plan / `docs/harness/`):

- **L0** static: `cargo build`, `cargo clippy -- -D warnings`, `cargo fmt --check`, `npx tsc --noEmit`
- **L1** unit (Rust, pure): `cargo test -p personafix-core`
- **L2** integration (Rust, temp SQLite, no GUI): `cargo test -p personafix-desktop`
- **L3a** data smoke: build/load a real `game_data.db`, assert query layer returns non-empty rows
- **L3b** UI smoke: `tauri-driver`/Playwright drives the app (added later)
- **L4** human: you run `make dev` and confirm it *feels* usable

Status legend: `[ ]` todo · `[~]` in progress · `[x]` done · `[!]` parked (see journal)

---

## Phase 0 — Oracle (harness can trust its own green light)
- [x] Parity checklist exists (this file) — *gate: file present*
- [x] Backlog exists and is ordered (`docs/harness/backlog.md`)
- [x] Conflict-detection TDD seed exists and fails for the right reason
      (`crates/core/tests/conflict_detection.rs`) — *gate: L1 red under `--ignored`*
- [x] Driver workflow script exists for review (`docs/harness/driver.workflow.js`)
- [x] L3a data-smoke test scaffold exists (ignored until P1 produces a DB)

## Phase 1 — Game data (THE blocker to usability)
- [ ] ChummerGenSR4 data source located; redistribution stance confirmed
      (Chummer is GPL but data encodes copyrighted SR content → keep `vendor/` and
      `game_data.db` git-ignored; do **not** commit/redistribute)
- [ ] `vendor/` populated; `make migrate` produces a `game_data.db` — *gate: L3a green*
- [ ] Fallback if blocked: hand-authored SR4 seed (metatypes, ~40 skills, common
      qualities/gear) so the app is usable end to end — *gate: L3a green*

## Phase 2 — Data layer hygiene
- [ ] Desktop query layer verified against a real DB (skills/qualities/weapons/augs
      return rows) — *gate: L2 + L3a*. NOTE: the desktop `query_*_db` functions in
      `commands.rs` are already implemented and tested; the `todo!()` stubs in
      `crates/data/src/sqlite.rs` (`SqliteGameDataRepository`) are **dead code** not on
      the app's path — either delete them or implement for consistency (low priority).

## Phase 3 — Builder UI completeness (SR4)
- [ ] Skills panel populated from `game_data.db` (not just seeded test data) — *L3b + L4*
- [ ] Qualities panel populated from `game_data.db` — *L3b + L4*
- [ ] Augmentations panel (essence cost shown, grade selectable) — *L3b + L4*
- [ ] Gear/weapons/armor panel — *L3b + L4*
- [ ] Contacts panel (connection/loyalty, 1 BP per point) — *L3b + L4*
- [ ] Magic panel (spells/adept powers) — *L3b + L4*
- [ ] Character list / load screen (open an existing character) — *L3b + L4*

## Phase 4 — Conflict detection & reporting (your explicit goal)
- [x] P4-1: incompatible qualities reported as **Warning** — *gate: L1 (un-ignore seed)*
- [x] P4-2: essence overage (augs exceed 6.00) reported as **Error** — *gate: L1*
- [x] P4-3: magic + resonance simultaneously reported as **Error** — *gate: L1*
- [ ] Conflicts surfaced in the builder UI (severity-colored, non-blocking) — *L3b + L4*

## Phase 5 — Career play
- [ ] UI to apply ledger events (karma/nuyen received/spent) — *L2 round-trip + L4*
- [ ] UI for skill/attribute improvement (karma cost shown) — *L2 + L4*
- [ ] Career timeline / ledger view — *L3b + L4*

## Phase 6 — Later editions (only after SR4 sign-off)
- [ ] SR5 priority flow verified end to end against `game_data.db`
- [ ] SR5 parity checklist drafted (separate file)

---

**Done = every box above checked at its gate, and you have run `make dev` and agreed the
SR4 flow is a usable Chummer replacement (the final L4).** The loop never checks its own
L4 box.
