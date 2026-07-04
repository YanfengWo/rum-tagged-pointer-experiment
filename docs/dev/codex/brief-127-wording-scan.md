# Brief 127 — Wording scan: crates/ + three light docs

Implementation brief. Fourth step of `docs/dev/release-transformation-plan.md`
(read its Policy + code-inventory sections first). This is a RENAME/REWORD
pass with **zero behavior change**: no assertion, expected value, logic, or
public API changes of any kind.

## Goal

Remaining external-engine wording inside `crates/` and three kept docs is
gone: test functions and one test file are renamed to describe the behavior
they assert; comments, doc-strings, and catalog descriptions state behavior
as `rum`'s own documented contract instead of citing an external engine as
authority. The whitelisted ecosystem identifiers stay. `cargo test
--workspace` remains green with the same test count (only names change).

## Context pointers

- Policy: `AGENTS.md` §Testing Policy; `DISCLAIMER.md` (committed versions).
- The full mention inventory (2026-07-04, post-brief-126): 25 files under
  `crates/` with 1-4 mentions each, plus `docs/runtime-evaluation-audit.md`,
  `docs/host-embedding-and-hardening.md`, `docs/host-adapter-plan.md`.
  Find them with:
  `rg -in 'wolfram|mathematica|wstp|mma[-_]hol' crates/ docs/runtime-evaluation-audit.md docs/host-embedding-and-hardening.md docs/host-adapter-plan.md`

## Scope

- MAY modify: any file under `crates/` matched by the rg above (renames of
  test fns/modules included — e.g. `crates/rum-stdlib/src/tests/mma_hol_cold_lane.rs`
  and its `tests.rs` module declaration), plus exactly these docs:
  `docs/runtime-evaluation-audit.md`, `docs/host-embedding-and-hardening.md`,
  `docs/host-adapter-plan.md`.
- MUST NOT touch: `ARCHITECTURE.md`, `docs/v0.1-acceptance-boundary.md`
  (Claude rewrites those next), `README.md`, `PROGRESS.md`, `TODO.md`,
  `DISCLAIMER.md`, `AGENTS.md`, `CLAUDE.md`, `ISSUE-sandbox-profile-fs-builtins.md`
  (removed later by brief-130), anything under `docs/dev/`, `scripts/`,
  `Cargo.*`, `.git`.

## Rewording rules

1. **Test fn / file renames**: a name like
   `factorization_builtins_match_wolfram_kernel_integer_anchors` becomes a
   behavior-descriptive name (`factorization_builtins_integer_anchors`).
   Rename `mma_hol_cold_lane.rs` to a behavior-descriptive module name (its
   tests cover exact aggregation shapes, denominator/GCD-LCM reducers, and
   unicode identifiers — e.g. `exact_reduction_shapes.rs`), updating the
   module declaration. Do not change what any test asserts.
2. **Comments / doc-strings**: where a comment cites an external engine as
   the authority ("matches Wolfram...", "WK returns..."), restate the fact as
   rum's contract ("`FullForm` holds its argument", "the documented behavior
   is ..."). Where a comment only explains intent, drop the engine reference.
3. **Catalog descriptions** (`catalog/*.rs`, `builtin_catalog*.rs`): neutral
   phrasing; describe what the builtin does in rum.
4. **WHITELIST — keep verbatim**: the Jupyter ecosystem identifiers
   `"codemirror_mode":"mathematica"` / `codemirror_mode, "mathematica"`,
   `"pygments_lexer":"mma"`, and the `.wl` file extension — both where set in
   src and where asserted in tests. These are third-party ecosystem mode
   names, covered by `DISCLAIMER.md`.
5. **Docs (the three listed)**: reframe "the intended Mathematica /
   Wolfram-style core language"-type phrasing as "rum's documented core
   language"; a reference to a removed doc (support matrix, mma-hol lanes,
   snapshot pipeline) is rewritten to point at `docs/testing-strategy.md` or
   the forthcoming guide/reference, or dropped if the sentence no longer
   serves; a workload example that names mma-hol becomes a generic "a large
   downstream package/workload". Keep each doc's technical content intact —
   this is reframing, not rewriting.
6. If a comment's claim cannot be reworded without DECIDING semantics (i.e.
   the code's intent is unclear without the external citation), leave it
   unchanged and REPORT it — do not decide.

## Deliverable

- The renames/rewording, uncommitted in the working tree.
- Verbatim verification output.
- A short table in the report: file → what changed (renamed / comments / n/a).

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | tail -30
rg -in 'wolfram|mathematica|wstp|mma[-_]hol' crates/ || echo CRATES-CLEAN
# expected residue: ONLY the whitelisted jupyter identifier lines — list them
rg -in 'wolfram|mathematica|wstp|mma[-_]hol' docs/runtime-evaluation-audit.md docs/host-embedding-and-hardening.md docs/host-adapter-plan.md || echo DOCS-CLEAN
git status --short
```

Test totals must match the pre-change totals (renames only — if a count
changes, something was dropped; STOP and fix or report).

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. No network.
- ZERO behavior change: no assertion/expected-value/logic edits; renames and
  natural-language text only.
- Minimal diff: no drive-by reformatting; do not "improve" unrelated comments.
- Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format (your final message)

1. What changed — per file, one line (renamed X→Y / reworded N comments).
2. Verification — each command with verbatim output, incl. the whitelisted
   residue list.
3. Rule-6 leftovers — comments left unchanged because rewording would decide
   semantics (empty if none).
4. Stop-loss / open questions (empty if none).
