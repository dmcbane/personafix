// personafix autonomous driver — REVIEW COPY (not run automatically).
//
// This is the deterministic loop that drives personafix toward SR4 parity. It is a
// Claude Code *Workflow* script: deterministic control flow (the while-loop, the
// stuck-counter, the commit gate) lives here in plain JS; every step that needs judgment
// is delegated to a subagent via agent(). The script itself has no filesystem access —
// it reads/writes the backlog THROUGH agents.
//
// To run it (token-heavy; spawns many agents): the operator invokes the Workflow tool
// with {scriptPath: "docs/harness/driver.workflow.js"} AFTER explicitly opting in.
// It is intentionally NOT launched as part of bootstrap so you can read it first.
//
// Guardrails encoded below: TDD (test before code), park-after-2-failures, adversarial
// anti-spec-gaming review before any commit, and a hard stop on human-checkpoint items.

export const meta = {
  name: 'personafix-driver',
  description: 'Iterate personafix toward SR4 parity: pick backlog item, TDD, verify, review, commit',
  phases: [
    { title: 'Select' },
    { title: 'Implement' },
    { title: 'Verify' },
    { title: 'Commit' },
  ],
}

// ---- schemas (force structured returns; the model retries on mismatch) ----

const NEXT_ITEM = {
  type: 'object',
  required: ['hasItem'],
  properties: {
    hasItem: { type: 'boolean' },
    id: { type: 'string' },
    desc: { type: 'string' },
    accept: { type: 'string' },
    humanCheckpoint: { type: 'boolean', description: 'true if item is marked HUMAN CHECKPOINT' },
  },
}

const WORK_RESULT = {
  type: 'object',
  required: ['green', 'summary'],
  properties: {
    green: { type: 'boolean', description: 'true only if L0+L1 (and any named L2/L3 gate) all pass' },
    summary: { type: 'string' },
    gatesRun: { type: 'string', description: 'which commands were run and their result' },
  },
}

const REVIEW = {
  type: 'object',
  required: ['clean'],
  properties: {
    clean: { type: 'boolean', description: 'false if the diff games the spec' },
    findings: { type: 'array', items: { type: 'string' } },
  },
}

// ---- the loop ----

const MAX_ITEMS = 50           // backstop; real stop is "no ready item"
let dryStop = false

for (let i = 0; i < MAX_ITEMS && !dryStop; i++) {
  // Budget guard: if the operator set a token target, stop before blowing it.
  if (budget.total && budget.remaining() < 80_000) {
    log(`stopping: ~${Math.round(budget.remaining() / 1000)}k tokens left, below per-item reserve`)
    break
  }

  // ---- Select ----
  phase('Select')
  const next = await agent(
    `Read docs/harness/backlog.md. Pick the FIRST item with status 'todo' whose every dep ` +
    `is status 'done'. Return it. If none are ready, return hasItem=false. Do not modify anything.`,
    { label: 'select', phase: 'Select', schema: NEXT_ITEM },
  )

  if (!next || !next.hasItem) {
    log('no ready backlog items — loop complete')
    dryStop = true
    break
  }

  if (next.humanCheckpoint) {
    log(`HUMAN CHECKPOINT reached at ${next.id}: ${next.desc} — stopping for operator review`)
    break
  }

  log(`item ${next.id}: ${next.desc}`)

  // ---- Implement (TDD), with park-after-2 ----
  let work = null
  for (let attempt = 1; attempt <= 2; attempt++) {
    phase('Implement')
    work = await agent(
      `Work backlog item ${next.id}: "${next.desc}".\n` +
      `Acceptance: ${next.accept}\n\n` +
      `Follow strict TDD: first make the named test FAIL (write it or remove its #[ignore]), ` +
      `confirm red, then implement until green. Honor the repo CLAUDE.md: never swallow errors, ` +
      `no test that passes by skipping assertions, no new todo!()/unimplemented!(). ` +
      `Run the gates named in the acceptance criterion (at minimum: cargo build, ` +
      `cargo clippy -- -D warnings, cargo fmt --all --check, cargo test). ` +
      `Report green=true ONLY if every gate passed. Attempt ${attempt} of 2.`,
      { label: `impl:${next.id}#${attempt}`, phase: 'Implement', schema: WORK_RESULT },
    )
    if (work && work.green) break
    log(`attempt ${attempt} not green: ${work ? work.summary : 'agent died'}`)
  }

  if (!work || !work.green) {
    phase('Commit')
    await agent(
      `In docs/harness/backlog.md set item ${next.id} status to 'parked'. Append a short ` +
      `docs/dev-journal/ entry (YYYY-MM-DD-park-${next.id}.md) with symptom + why it's blocked. ` +
      `Do not commit code changes; revert any partial edits to a clean state first.`,
      { label: `park:${next.id}`, phase: 'Commit' },
    )
    continue
  }

  // ---- Verify (adversarial anti-spec-gaming review) ----
  phase('Verify')
  const review = await agent(
    `Adversarially review the working-tree diff for item ${next.id}. Assume the implementer ` +
    `may have gamed the spec. Look for: weakened/removed assertions, tests that assert nothing, ` +
    `new todo!()/unimplemented!()/unwrap() on error paths, errors caught and ignored, or ` +
    `acceptance met only superficially. Return clean=false with findings if ANY apply.`,
    { label: `review:${next.id}`, phase: 'Verify', schema: REVIEW },
  )

  if (!review || !review.clean) {
    phase('Commit')
    await agent(
      `Item ${next.id} failed adversarial review: ${review ? review.findings.join('; ') : 'no verdict'}. ` +
      `Set its backlog status to 'parked' and journal the findings. Revert the diff to clean.`,
      { label: `reject:${next.id}`, phase: 'Commit' },
    )
    continue
  }

  // ---- Commit (semver bump + changelog + mark done) ----
  phase('Commit')
  await agent(
    `Item ${next.id} is green and clean. Bump the project version per semver (patch for a fix, ` +
    `minor for a feature), update CHANGELOG.md, set the backlog item to 'done', tick the matching ` +
    `box in docs/sr4-parity-checklist.md, then commit (conventional message) and push.`,
    { label: `commit:${next.id}`, phase: 'Commit' },
  )
  log(`item ${next.id} done`)
}

return { stopped: dryStop ? 'backlog drained' : 'checkpoint/budget/backstop' }
