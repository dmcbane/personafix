# SR5 Parity Checklist — Definition of Done

This file tracks SR5-specific parity against the Chummer5a reference implementation.
SR4 items that carry forward unchanged are omitted; see `docs/sr4-parity-checklist.md`
for the baseline. Status follows the same convention:

- **L0** static: `cargo build`, `cargo clippy -- -D warnings`, `cargo fmt --check`, `npx tsc --noEmit`
- **L1** unit (Rust, pure): `cargo test -p personafix-core`
- **L2** integration (Rust, temp SQLite): `cargo test -p personafix-desktop`
- **L3a** data smoke: assert SR5 query layers return non-empty, well-formed rows
- **L4** human: run `make dev`, confirm the SR5 flow is usable

Status legend: `[ ]` todo · `[~]` in progress · `[x]` done · `[!]` parked

---

## Character Creation — Priority Table

- [x] Priority table data correct (metatype/attribute/magic/skill/resources rows) — *L1: sr5_priority.rs tests*
- [x] Priority selection UI — five categories, five levels, no repeats — *P6-1 done*
- [x] Starting attribute points from priority (attributes column) shown and enforced — *P6-3 done*
- [x] Starting skill points from priority (skill column) shown and enforced — *P6-1 done*
- [x] Starting resources (nuyen) from priority stored and tracked — *P6-1 done*
- [x] Special attribute pool (Magic/Resonance/Edge) from metatype priority — *P6-3 done*
- [x] Metatype racial limits applied per priority table — *SR5Rules implemented*
- [x] Magic/Resonance starting rating from magic_or_resonance priority column — *P6-2 done*

## Character Creation — Awakened Subtypes

- [x] Magician: buys spells at creation, casts spells in play — *P6-2 done*
- [x] Adept: power points = Magic rating; buys adept powers — *P6-5 done*
- [x] Mystic Adept: both spells and adept powers; power points must be explicitly allocated — *P6-5 done*
- [x] Technomancer: buys complex forms; fading instead of drain — *P8-1 done*
- [ ] Tradition selection for Magicians (Hermetic / Shaman / etc.) — no separate tradition panel yet
- [ ] Mentor Spirit selection (quality-like, adds bonuses) — *not implemented*

## Contacts

- [x] SR5 contact pool = Charisma × 3 (not 1 karma/point like SR4) — *P6-4 done*
- [x] Free pool distributed freely between connection and loyalty — *P6-4 done*
- [x] Overflow beyond free pool costs 1 karma each — *P6-4 done*

## Skills

- [x] Active skills from SR5 data (different list from SR4) — *L3a non-empty*
- [x] Skill groups (linked skills buy as a block) — *group_json stored; SR5Rules*
- [ ] Knowledge skills — not tracked (no panel)
- [ ] Language skills — not tracked (no panel)

## Qualities

- [x] SR5 qualities loaded from data — *L3a non-empty*
- [x] Positive/negative quality costs in karma (not BP like SR4) — *SR5Rules*
- [x] Quality conflict detection (incompatible_with) — *P4-1 done*
- [x] Essence overage error (augs > 6.00) — *P4-2 done*
- [x] Magic + Resonance mutual exclusion — *P4-3 done*

## Augmentations

- [x] SR5 cyberware/bioware list loaded from data — *L3a non-empty*
- [x] Essence cost graded (Standard 1×, Alpha 0.8×, Beta 0.7×, Delta 0.5×, Used 1.25×) — *engine done*
- [x] Bioware costs half essence vs. cyberware — *Chummer data pre-halves bioware essence (Cat's Eyes `<ess>0.1</ess>` = effective cost); grade-multiplier-only engine is correct. See `docs/dev-journal/2026-06-20-bioware-essence-halving.md`.*
- [ ] Grade availability restrictions — *not enforced*

## Weapons / Armor / Gear

- [x] SR5 weapons loaded from data — *L3a non-empty*
- [x] SR5 armor loaded from data — *L3a non-empty*
- [ ] Armor stacking rules (only highest rating counts, others provide half) — *not enforced*
- [ ] Availability and restricted/forbidden items — *not tracked*
- [ ] Ammo tracking — *not implemented*

## Complex Forms (Technomancer)

- [x] 38 SR5 complex forms seeded from complexforms.xml — *P8-1 done*
- [x] Fading value displayed; Resonance cap enforced in builder — *P8-1 done*
- [ ] Compiling/decompiling sprites — *not implemented*

## Career Play (SR5-specific)

- [x] Karma costs match SR5 table (attribute × 5, active skill × 2) — *SR5Rules done*
- [ ] Initiation / Submersion — *not implemented*
- [ ] Metamagic / Echo — *not implemented*
- [ ] Contact loyalty improvement (karma cost) — *not tracked*

## Character Sheet

- [x] All equipment visible in saved character sheet — *P9-1 done*
- [x] Dice roller with skill pool shortcuts — *P10-1 done*
- [ ] Print/export character sheet — *P10-4 todo*

---

**Done = every checked item above verified, SR5 creation flow tested end to end, and
you have run `make dev` and agreed the SR5 flow is a usable Chummer5a replacement (L4).**
