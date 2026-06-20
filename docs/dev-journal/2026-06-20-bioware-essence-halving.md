# Bioware Essence Halving Audit

**Symptom:** `docs/sr5-parity-checklist.md` flagged "Bioware essence cost halving" as
unimplemented, implying the rules engine might double-count bioware essence.

**Root cause (non-issue):** ChummerGenSR4 and Chummer5a bioware XML files store the
*already-halved* effective essence cost directly in the `<ess>` field. For example:
- Cat's Eyes (bioware): `<ess>0.1</ess>` — this is the 0.1E effective cost after halving
  (the "raw" biological cost would be 0.2E, but Chummer pre-computes it)

**Engine behavior:**
- `crates/migrate/` stores `ess` strings verbatim in `game_data.db` (e.g. `"0.1"`).
- `AugmentationPanel.tsx:parseEssenceCost` evaluates the string and converts to
  centessences: `Math.round(0.1 * 100) = 10`.
- `calculate_essence` in `sr4.rs`/`sr5.rs` applies `grade_multiplier` only:
  `adjusted = (base_cost * grade_multiplier) / 100`.
- Result: 10 centessence × Standard (100%) = 10 centessence = 0.1E. Correct.

**Fix:** No code change needed. The grade-multiplier-only engine is correct for both
cyberware and bioware because the data is pre-halved at the source.

**Follow-up:** Update `docs/sr5-parity-checklist.md` to mark bioware essence as verified.
