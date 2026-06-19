# 2026-06-19 — Autonomous harness bootstrap

## Symptom
personafix is "unusable" despite a solid (~121-test) rules engine. Goal: stand up a
looping agent harness that can iterate the app to SR4 Chummer parity autonomously
(within guardrails).

## Root cause of "unusable" (corrected during exploration)
Two earlier assumptions were wrong and worth recording so we don't re-debug them:

1. **The `todo!()` stubs in `crates/data/src/sqlite.rs` are NOT the blocker.** That
   `SqliteGameDataRepository` impl of `GameDataRepository` is **dead code** — the desktop
   app reimplements its own game-data queries in `apps/desktop/src-tauri/src/commands.rs`
   (`query_skills_db`, `query_qualities_db`, `query_weapons_db`, `query_augmentations_db`),
   which are fully implemented and unit-tested with seeded in-memory data
   (`commands.rs` tests around lines 1516–1638). Nothing calls the trait impl.
2. **The actual blocker is the absence of `game_data.db`.** The migrate crate works but
   needs `vendor/` Chummer XML repos that have never been run. Given a populated DB, the
   query path already functions. So usability is closer than "65%" implied — the gating
   item is data, then UI breadth, then conflict reporting.

A second real gap: `SR4Rules::validate_creation` (`crates/core/src/rules/sr4.rs:63`)
does not detect quality incompatibilities, essence overage, or magic+resonance — i.e. the
"identify and report conflicts" feature the user explicitly wants does not exist yet.

## Fix (this session = scaffolding only)
Built the harness foundations, not the features:
- `docs/sr4-parity-checklist.md` — the machine-checkable Definition of Done (loop's stop
  condition), keyed to a verification ladder L0–L4.
- `docs/harness/backlog.md` — ordered, dependency-aware work queue with per-item gates.
- `crates/core/tests/conflict_detection.rs` — TDD seed: three tests that encode the
  conflict-detection spec and **fail for the right reason** (validate_creation returns
  `[]`). Kept `#[ignore]` with loud reasons so `make test` stays green; verified red via
  `cargo test -p personafix-core -- --ignored`.
- `docs/harness/driver.workflow.js` — the deterministic driver (review copy, NOT run).
  Encodes: select ready item → TDD implement → park-after-2 → adversarial anti-spec-gaming
  review → semver bump + commit. Stops on human-checkpoint items (e.g. data licensing).

## Key design note
The harness's first job is to build its own *oracle*. `cargo test` proves the rules engine
but says nothing about the running app — an autonomous loop with only L1 would "ship a
blank window confidently." The ladder (L2 integration, L3a data smoke, L3b UI smoke) is
what makes the loop's green light trustworthy. Build the oracle before trusting the loop.

## Follow-up
- Licensing of ChummerGenSR4 data is a HUMAN CHECKPOINT (backlog P1-1) — the loop must
  stop and ask, not guess. Keep `vendor/` and `game_data.db` git-ignored regardless.
- Next concrete step (with operator go-ahead): backlog P4-1..3 (conflict detection) is the
  cleanest first real loop iteration — pure-core, no data dependency, un-ignores the seed.
