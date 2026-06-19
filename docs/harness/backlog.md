# Harness Backlog — ordered work queue

The autonomous driver reads this file top-to-bottom, picks the first `todo` item whose
`deps` are all `done`, runs the per-item micro-cycle (TDD → implement → verify →
adversarial review → commit), then updates the item's status here. State lives on disk so
any run can be killed and resumed (see `driver.workflow.js`).

**Per-item micro-cycle** (driver-enforced):
1. mark `doing`
2. write/enable the failing test named in `accept` (must fail first — TDD)
3. implement until the named gates are green (L0+L1, plus L2/L3 where stated)
4. adversarial review: a second agent checks the diff for spec-gaming — weakened asserts,
   new `todo!()`/`unwrap()`/`unimplemented!()`, swallowed errors (your CLAUDE.md rules)
5. green & clean → bump version + changelog (semver), commit + push, mark `done`
6. red after **2** attempts → mark `parked`, journal why, continue (your chosen guardrail)

Status: `todo` · `doing` · `done` · `parked`

---

## P1 — Game data (unblocks everything UI)

- id: P1-1  status: todo  deps: []
  desc: Locate ChummerGenSR4 data repo; confirm license/redistribution. Document findings.
  accept: a note in `docs/dev-journal/` records the source + license decision; `vendor/`
          and `game_data.db` added to `.gitignore`.
  note: HUMAN CHECKPOINT — licensing is a judgment call; loop should stop and ask if unsure.

- id: P1-2  status: todo  deps: [P1-1]
  desc: Clone vendor data into `vendor/`, run `make migrate` to produce `game_data.db`.
  accept: L3a — `game_data.db` exists and the data-smoke test (P0-5) returns non-empty
          SR4 skills + qualities.

- id: P1-2b status: todo  deps: [P1-1]
  desc: FALLBACK if P1-2 blocked — hand-author a small SR4 seed dataset (5 metatypes,
        ~40 skills, ~30 common qualities, a few weapons/armor/augs) as a migration or fixture.
  accept: L3a green using the seed.

## P0 — Oracle scaffolding (mostly done in bootstrap)

- id: P0-5  status: todo  deps: []
  desc: Add the L3a data-smoke test: given a `game_data.db` path (env var or fixture),
        assert `query_skills_db`/`query_qualities_db` return non-empty, well-formed rows.
        Mark `#[ignore]` with a loud reason until P1 provides the DB.
  accept: test compiles; ignored with documented reason; un-ignored it would exercise the
          real query path end to end.

## P4 — Conflict detection (your explicit goal; pure-core, no data dependency)

- id: P4-1  status: done   deps: []
  desc: Detect mutually-incompatible qualities in `SR4Rules::validate_creation`, using the
        existing `Quality::incompatible_with` field. Report as `Warning` (don't block).
  accept: remove `#[ignore]` from `incompatible_qualities_are_reported_as_warning`
          (`crates/core/tests/conflict_detection.rs`) and make it pass — gate L1.

- id: P4-2  status: todo  deps: []
  desc: Detect essence overage (augmentations whose graded essence cost drives essence
        <= 0) in `validate_creation`. Reuse `calculate_essence`. Report as `Error`.
  accept: un-ignore `essence_overage_is_reported_as_error` and make it pass — gate L1.

- id: P4-3  status: todo  deps: []
  desc: Detect magic + resonance set simultaneously in `validate_creation`. Report `Error`.
  accept: un-ignore `magic_and_resonance_together_is_reported_as_error` and pass — gate L1.

- id: P4-4  status: todo  deps: [P4-1, P4-2, P4-3]
  desc: Surface conflicts in the builder UI — severity-colored, non-blocking banner.
  accept: L3b smoke shows a warning/error rendered for a conflicting draft + L4.

## P2 — Data layer hygiene

- id: P2-1  status: todo  deps: [P1-2]
  desc: Either delete the dead `SqliteGameDataRepository` `todo!()` stubs in
        `crates/data/src/sqlite.rs`, or implement them to match the desktop query layer.
  accept: no `todo!()` remain in `crates/data`; L0+L1 green. (Low priority — not on app path.)

## P3 — Builder UI completeness (SR4)

- id: P3-1  status: todo  deps: [P1-2]
  desc: Wire Skills + Qualities panels to real `game_data.db` (replace any seeded data).
  accept: L3b smoke: open builder, skill/quality lists are non-empty + L4.

- id: P3-2  status: todo  deps: [P1-2]
  desc: Add Augmentations panel (essence cost + grade), mirroring `QualityPanel`.
  accept: L3b smoke + L4.

- id: P3-3  status: todo  deps: [P1-2]
  desc: Add Gear/Weapons/Armor panel.
  accept: L3b smoke + L4.

- id: P3-4  status: todo  deps: []
  desc: Add Contacts panel (connection/loyalty, 1 BP per point).
  accept: L3b smoke + L4.

- id: P3-5  status: todo  deps: [P1-2]
  desc: Add Magic panel (spells + adept powers).
  accept: L3b smoke + L4.

- id: P3-6  status: todo  deps: []
  desc: Character list / load screen (uses existing `list_characters` + `get_character`).
  accept: L3b smoke: create two characters, reopen one + L4.

## P5 — Career play

- id: P5-1  status: todo  deps: [P3-6]
  desc: UI to apply ledger events (karma/nuyen received/spent) via `apply_event`.
  accept: L2 round-trip already exists; add L3b smoke + L4.

- id: P5-2  status: todo  deps: [P5-1]
  desc: UI for skill/attribute improvement (show karma cost from rules engine).
  accept: L3b smoke + L4.

- id: P5-3  status: todo  deps: [P5-1]
  desc: Career timeline / ledger view (uses `get_ledger`).
  accept: L3b smoke + L4.
