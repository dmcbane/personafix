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
- [x] ChummerGenSR4 data source located; redistribution stance confirmed
      (Chummer is GPL but data encodes copyrighted SR content → keep `vendor/` and
      `game_data.db` git-ignored; do **not** commit/redistribute)
- [x] `vendor/` populated; `make migrate` produces a `game_data.db` — *gate: L3a green*

## Phase 2 — Data layer hygiene
- [x] Desktop query layer verified against a real DB (skills/qualities/weapons/augs
      return rows) — *gate: L2 + L3a*. The dead `todo!()` stubs in
      `crates/data/src/sqlite.rs` deleted (P2-1 done).

## Phase 3 — Builder UI completeness (SR4)
- [x] Skills panel populated from `game_data.db` — *L3a + L4*
- [x] Qualities panel populated from `game_data.db` — *L3a + L4*
- [x] Augmentations panel (essence cost shown, grade selectable) — *P3-2 done*
- [x] Gear/weapons/armor panel — *P3-3 done*
- [x] Contacts panel (connection/loyalty) — *P3-4 done*
- [x] Magic panel (spells + adept powers + complex forms per tradition) — *P3-5, P6-2, P6-5 done*
- [x] Character list / load screen — *P3-6 done*

## Phase 4 — Conflict detection & reporting (your explicit goal)
- [x] P4-1: incompatible qualities reported as **Warning** — *gate: L1 (un-ignore seed)*
- [x] P4-2: essence overage (augs exceed 6.00) reported as **Error** — *gate: L1*
- [x] P4-3: magic + resonance simultaneously reported as **Error** — *gate: L1*
- [x] Conflicts surfaced in the builder UI (severity-colored, non-blocking) — *L4 verified*

## Phase 5 — Career play
- [x] UI to apply ledger events (karma/nuyen received/spent, nuyen spend) — *P5-1 done*
- [x] UI for skill/attribute improvement (karma cost shown) — *P5-2 done*
- [x] Career timeline / ledger view — *P5-3 done*

## Phase 5b — Character sheet completeness
- [x] Full equipment display in saved sheet (qualities, augs, spells/powers/forms,
      contacts, weapons, armor) — *P9-1 done*
- [x] DraftAdeptPower.cost save bug fixed (string → number centessences) — *P9-1 done*

## Phase 6 — Later editions (only after SR4 sign-off)
- [x] SR5 priority flow verified end to end against `game_data.db` — *P6-1–P6-5 done*
- [x] SR5 parity checklist drafted (separate file) — *docs/sr5-parity-checklist.md, P10-2 done*

---

**Done = every box above checked at its gate, and you have run `make dev` and agreed the
SR4 flow is a usable Chummer replacement (the final L4).** The loop never checks its own
L4 box.
