# Release Transformation Plan — transparent main-branch release shape

Status: ACTIVE (started 2026-07-04). Coordination doc for briefs 124-130 plus the
maintainer's own batches. This file is the authoritative per-file inventory; each
brief's MAY-modify list derives from it.

## Decision

The earlier plan (a scrubbed, fresh-history `release-staging/` export) is DROPPED.
Instead, `main` itself is transformed into release shape and the full history is
kept. Rationale: a fresh-history repo reads as concealment and has no protective
effect anyway; a visible correction is the stronger, honest posture. The repo has
no remote; the endpoint of this line is a clean tree + a bundle-mode cold backup.
Publication mechanics (hosting, repo name, first tag) are a separate later decision.

Verified before adopting this plan (2026-07-04): **no WolframKernel-generated
content was ever committed** — `WLBuiltin/` was always gitignored, difftest oracle
outputs were transient by design, a full-history filename audit found no oracle
dumps, and the curated difftest cases are hand-written `input ||| expected` pairs.

## Policy (the new testing doctrine)

Tests verify exactly two things from now on:

1. rum's behavior matches **its own documented design** (the language guide +
   generated builtin reference become the normative contract);
2. **performance A/B between rum builds** during optimization work.

Consequences:

- Differential testing against WolframKernel, the mma-hol snapshot/acceptance
  pipeline, and compatibility matrices/reports are all removed and will not
  return. This becomes a hard line in `AGENTS.md` / `CLAUDE.md`.
- Existing in-crate unit/regression tests are KEPT — they already assert rum's
  behavior. Only provenance-flavored names/comments are reworded (brief-127).
- The removed assets go to the private archive
  `/Users/fft/Developments/Archive/rum-history/` (mirrors repo paths;
  `MANIFEST.md` there records origin, removal commit, nature, recycling leads).
  The public source of truth remains git history; the archive is a convenience
  mirror only.

## Sequence

| Step | Owner | Content |
|---|---|---|
| M0 | Claude | Commit this plan; start MANIFEST bookkeeping (WLBuiltin already archived) |
| brief-124 | Codex | Remove `crates/rum-difftest` + `scripts/export-wl-builtins.wls`; archive copies; workspace/`.gitignore` cleanup |
| brief-125 | Codex | Archive WK/mma-hol scripts; strip the `wolfram` engine from both microbench scripts (keep rum lane + labels as the A/B tool) |
| brief-126 | Codex | Archive closed-line docs + `docs/dev/codex/brief-001..123` + `docs/dev/kernel-api-audit-2026-07-03.md` per the inventory below |
| M1 | Claude | Rewrite DISCLAIMER (user-reviewed), README, AGENTS/CLAUDE hard lines, testing-strategy, TODO refresh |
| brief-127 | Codex | Wording scan (brief-096 method): crates/ test renames + comments/strings; kept docs reframing per replacement guidance |
| brief-128 | Codex | Builtin reference generator from catalog `BuiltinSpec` (+ optional limits field, drift test) |
| brief-129 | Codex | Hand-written language guide under `docs/guide/` |
| brief-130 | Codex | rustdoc pass (rum-host focus) + per-crate Cargo metadata + top-level housekeeping |
| M2 | Claude | Final sweep + gate; archive briefs 124-130; fill MANIFEST hashes; bundle cold backup |

Gates per brief: `cargo fmt --check`, `cargo test --workspace`, `rg` residue
sweeps named in the brief, scope check against this inventory. No perf gates
(nothing touches evaluation paths); brief-125 sanity-runs the rum-only microbench.

## Inventory — scripts/

| Script | Disposition |
|---|---|
| `export-wl-builtins.wls` | ARCHIVE (brief-124; runs WolframKernel) |
| `mma-hol-bounded-smoke.sh` | ARCHIVE (125) |
| `pair-simp-hotspot-probes.sh`, `pair-simp-pair-body-phase-tree.sh`, `pair-simp-repetition-probe.sh` | ARCHIVE (125) |
| `perf-baseline-stage0.sh` | ARCHIVE (125; drives mma-hol) |
| `perf-probes/` (incl. `wolfram/wk-conversion-wrapper-profile.wls`) | ARCHIVE (125) |
| `rebuild-snapshot.sh`, `snapshot-checkpoint-chain.sh`, `snapshot-reliability.sh` | ARCHIVE (125; mma-hol snapshot pipeline) |
| `core-language-microbench.sh`, `core-memory-microbench.sh` | TRANSFORM (125): remove `wolfram`/`both` engines, WK columns/ratios; keep labels, guarded timing, TSV output |
| `dispatch-brief.sh`, `guarded-test-run.sh`, `run-bounded.py`, `sample-self-time.py`, `sample-attribute-callers.py`, `rum-jupyter-*`, `smoke-rum-jupyter.sh`, `write-rum-jupyter-kernelspec.sh` | KEEP (rum-local tooling) |

## Inventory — docs/

ARCHIVE in brief-126 (closed-line plans/audits, regardless of WK content):
`bytecode-vm-closure-plan.md`, `bytecode-vm-design.md`, `bytecode-vm-general-plan.md`,
`bytecode-vm-plan.md`, `bytecode-vm-research-notes.md`, `core-language-support-matrix.md`,
`crate-rebalance-plan.md`, `evaluator-performance-audit.md`,
`jupyter-host-migration-audit.md`, `layer-b-node-rewrite-plan.md`,
`long-tail-gap-audit.md`, `memory-management-plan.md`, `mma-hol-smoke-and-snapshot.md`,
`next-session-handoff.md`, `performance-kernel-rewrite-plan.md`,
`performance-stage0-baseline.md`, `PROGRESS-archive-2026-07-03.md`,
`proptaut-performance-audit.md`, `representation-rewrite-plan.md`,
`wolfram-builtin-inventory.md`, `docs/dev/kernel-api-audit-2026-07-03.md`,
`docs/dev/codex/brief-001-*` … `brief-123-*` (TEMPLATE.md stays; briefs 124-130
are archived at M2 after execution).

KEEP at HEAD (current-truth; wording pass in M1/127):
`ARCHITECTURE.md`, `README.md` (M1 rewrite), `DISCLAIMER.md` (M1 rewrite),
`AGENTS.md`/`CLAUDE.md` (M1), `TODO.md` (M1 refresh), `PROGRESS.md` (append-only log),
`docs/code-organization-guidelines.md`, `docs/formatter-style.md`,
`docs/host-adapter-plan.md`, `docs/host-embedding-and-hardening.md`,
`docs/jupyter-console-manual.md`, `docs/lowering-canonical-forms.md`,
`docs/python-adapter-plan.md`, `docs/runtime-evaluation-audit.md`,
`docs/testing-strategy.md` (M1 rewrite), `docs/v0.1-acceptance-boundary.md`,
`docs/dev/codex/TEMPLATE.md`. New: `docs/guide/` (129), `docs/reference/` (128).

`ISSUE-sandbox-profile-fs-builtins.md`: converted to a TODO entry and removed (130).

## Inventory — code (brief-127 scan)

- Rename `*_match_wolfram_kernel_*`-style test fns; reword comments/strings that
  cite WK observations as authority (the behavior is now rum's own contract).
- Third-party ecosystem identifiers are KEPT as-is (nominative use, one
  DISCLAIMER sentence covers them): jupyter `codemirror_mode: "mathematica"`,
  `pygments_lexer: "mma"`, the `.wl` file extension. An own extension is a later
  product decision, out of scope here.
- Whitelist after 127: Wolfram/Mathematica may appear only in `DISCLAIMER.md`
  and (sparingly, nominative) `README.md` + the kept ecosystem identifiers.

## Open items

- CI workflow: deferred until a remote/hosting exists.
- crates.io publication: undecided; Cargo metadata (130) written to be ready.
- Test-asset recycling from the archive (curated cases → spec regression,
  catalog-drift inventory tool, lane_d baseline): on demand, per the policy
  above — an expected value is recycled only if promoted into rum's own
  reference as a design commitment.
- Post-removal coverage gap: rebuild integration depth rum-natively (llvm-cov
  buildout restart; sandbox/stress corpus aligned with the embedding
  direction) — tracked in TODO.md after M1.
