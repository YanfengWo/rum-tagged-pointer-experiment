# Brief 215 — Evaluator hold correctness: Unevaluated restore, HoldAll Sequence splice, ReleaseHold contract

Fix brief — unit-9 batch 1 (safety, L). Source: `docs/dev/usability-pass/
kernel-b-findings.md` (headline + gaps 2/3/5 + the hold-design appendix) —
read it FIRST; its Evidence E2/E4/E5 transcripts are the before-state and its
`file:line` citations are the seams. All three changes below move a WL name's
behavior TOWARD documented WL semantics (the sanctioned enhancement class);
none adds an option or a new symbol.

This batch touches the EVALUATOR CORE — argument planning and no-match
reconstruction — so it carries the mandatory core-language microbench A/B
gate and a zero-default-path-cost rule. The three changes share one A/B run.

## Change (a) — `Unevaluated` wrapper restore on the true no-consumer path

Today (audit E4, reproduced at the gate):

```
f[Unevaluated[1 + 1]]                    → f[2]        (WL: f[Unevaluated[1 + 1]])
q[x_Integer] := hit[x];
q[Unevaluated[1 + 1]]                    → hit[2]      (WL: q[Unevaluated[1 + 1]] — the
                                                        raw Plus[1,1] does NOT match
                                                        x_Integer, and losing the wrapper
                                                        must never CREATE a match)
```

The seam: ordinary-position planning strips the wrapper and records
`should_evaluate: false` (`eval_call_prepare.rs:50-55`); when every dispatch
stage misses, `resume_call_with_dispatch_plan` reconstructs the call solely
from prepared arguments (`eval_core/call.rs:97-101`) — no provenance bit
survives to restore the wrapper, and the next fixed-point pass evaluates the
now-bare content.

Required behavior:

- A stripped `Unevaluated` wrapper is RESTORED in the reconstructed call when
  NO stage consumed the argument (undefined head, or definitions present but
  none matched). All consuming paths keep today's behavior byte-identical:
  builtins (`Length[Unevaluated[1 + 1]]` → `2`), held heads
  (`Hold[Unevaluated[1 + 1]]` → `Hold[Plus[1, 1]]`), matching downvalues
  (`g[x_] := HoldComplete[x]` → `HoldComplete[Plus[1, 1]]`), explicit
  `Function`, upvalues, and the standalone unary `Unevaluated[e]` builtin
  call.
- **Fixed-point stability is mandatory**: the restored call must be
  byte-identical to the input so the evaluator's change-detection terminates.
  Test that `f[Unevaluated[1 + 1]]` is a stable fixed point (evaluate the
  result again; assert identity, no loop, no message).
- Matching must run against the RAW content (it already does); the fix is
  ONLY at reconstruction. Do not touch how consumers receive the argument.
- **Zero default-path cost**: record provenance ONLY when a wrapper is
  actually stripped (the strip site already branches on the wrapper — set a
  plan flag there); the restore check runs only inside no-match
  reconstruction. No allocation, no extra branch on calls that contain no
  `Unevaluated`. State in the report where the flag lives and why the
  wrapper-free path pays nothing.
- The `Unevaluated[Sequence[…]]` composition edge (E4:
  `f[Unevaluated[Sequence[a, b]]]` → `f[a, b]` today): the WL surface here
  is NOT confidently documented — do not guess WL parity. Pick the tightest
  self-consistent rule compatible with (a)+(b) (acceptable: a wrapper whose
  content was spliced away has no single position to restore — provenance is
  dropped, the call keeps the spliced shape), DOCUMENT it in a test with a
  comment naming it as rum's pinned contract, and list it in the report as a
  pinned-not-WL row.

## Change (b) — ordinary `HoldAll` must allow raw `Sequence` splicing

Today (audit E5): `SetAttributes[h, HoldAll]; h[Sequence[a, b]]` →
`h[Sequence[a, b]]` — the planner hard-wires `allow_sequence_splice: false`
for every held argument (`eval_call_prepare.rs:18-47`). WL: `Sequence`
objects are flattened in ALL functions except those with `SequenceHold` or
`HoldAllComplete`; holding evaluation and holding sequence-splicing are
independent axes.

Required behavior:

- `h[Sequence[a, b]]` (HoldAll `h`) → `h[a, b]` — spliced RAW (arguments stay
  unevaluated). Same for `Hold[Sequence[a, b]]` → `Hold[a, b]` and
  `Reap`-class `HoldFirst` positions.
- UNCHANGED: `hc[Sequence[a, b]]` (`HoldAllComplete`) and
  `sh[Sequence[a, b]]` (`SequenceHold`) stay nested; ordinary evaluated
  splicing (`f[Sequence[a, b]]` → `f[a, b]`) stays; the `Evaluate` pierce
  (`h[Evaluate[Sequence[a, b]]]` → `h[a, b]`) stays; `Nothing`'s List-only
  removal path is untouched.
- The gate for blocking is exactly `sequence_hold()` (`SequenceHold` |
  `HoldAllComplete`) — which `metadata.rs:223-229` already implements; the
  fix is to stop conflating it with the hold plan in the planner.

## Change (c) — `ReleaseHold`: add `HoldForm`, pin ONE layer

Today (audit E5, reproduced at the gate):

```
ReleaseHold[HoldForm[1 + 1]]     → HoldForm[Plus[1, 1]]   (WL: 2 — HoldForm is a
                                                            documented ReleaseHold target)
ReleaseHold[Hold[Hold[1 + 1]]]   → 2                       (WL: Hold[1 + 1] — ONE layer
                                                            per call)
```

The seam: `hold_thread.rs:17-30` LOOPS `release_hold_once` over consecutive
recognized wrappers, and the wrapper-name list
(`substitution/function_and_subst_helpers.rs:34-60`) omits `HoldForm`.

Required behavior:

- The recognized wrapper set becomes `Hold`, `HoldForm`, `HoldComplete`,
  `HoldPattern` (+ the attribute-hold heads the current list already names —
  do not silently drop existing members).
- Exactly ONE wrapper layer is removed per call at the positions rum strips
  today: `ReleaseHold[Hold[Hold[1 + 1]]]` → `Hold[1 + 1]`;
  `ReleaseHold[Hold[1 + 1]]` → `2` (the unwrapped content then evaluates
  normally); `ReleaseHold[HoldPattern[1 + 1]]` → `2` and
  `ReleaseHold[1 + 1]` → `2` unchanged.
- **Do NOT expand traversal scope.** WL's `ReleaseHold` also strips holds at
  deeper positions (`ReleaseHold[g[Hold[1 + 2]]]` → `g[3]` in WL). Whether
  rum does anything for non-top positions is UNAUDITED: probe
  `ReleaseHold[g[Hold[1 + 1]]]` (and one nested-list case), REPORT the
  current behavior verbatim as a boundary row for triage, and leave that
  behavior unchanged in this batch.
- Update the `ReleaseHold` registration description
  (`builtin_catalog/scoping.rs`) to name the wrapper set + one-layer
  contract; if docgen has an existing Limits note for it, MERGE (brief-198
  lesson: the LIMITS table is keyed by name and a blind add OVERWRITES).
  Regenerate the reference.

## Method — survey before coding

1. **Test survey first.** Grep the workspace tests for assertions that
   encode the OLD behaviors: `Sequence` staying nested under a `HoldAll`
   head, `ReleaseHold` multi-layer stripping or `HoldForm` non-release, and
   `Unevaluated` results shaped like today's `f[2]`. List every hit in the
   report with its verdict (must-update vs untouched). The MAY list below
   pre-includes the holding/control test families for exactly this reason
   (units 4/5 gate lesson: a behavior brief's MAY list spans every test file
   asserting the old behavior, across crates).
2. Implement (b) first (smallest seam), then (c), then (a) — each with its
   tests green before the next, so a regression is attributable.
3. New tests: at minimum, the pinned-contract probe list below plus the
   fixed-point stability test and one test per change asserting the
   UNCHANGED neighbors (`HoldAllComplete`/`SequenceHold` nesting, consuming
   `Unevaluated` paths, `ReleaseHold[HoldPattern[…]]`).

## Pinned-contract probe list (run before/after; paste both, verbatim)

```
printf '' | cargo run -q -p rum-cli -- --eval 'q[x_Integer] := hit[x]; SetAttributes[h, HoldAll]; SetAttributes[hc, HoldAllComplete]; SetAttributes[sh, SequenceHold]; {f[Unevaluated[1 + 1]], q[Unevaluated[1 + 1]], Length[Unevaluated[1 + 1]], Hold[Unevaluated[1 + 1]], h[Sequence[a, b]], Hold[Sequence[a, b]], hc[Sequence[a, b]], sh[Sequence[a, b]], f[Sequence[a, b]], h[Evaluate[Sequence[a, b]]], ReleaseHold[HoldForm[1 + 1]], ReleaseHold[Hold[Hold[1 + 1]]], ReleaseHold[Hold[1 + 1]], ReleaseHold[HoldPattern[1 + 1]], ReleaseHold[g[Hold[1 + 1]]], f[Unevaluated[Sequence[a, b]]]}'
```

Expected after-state (first 14 positions; 15 is the reported boundary, 16 is
the pinned-not-WL row):
`f[Unevaluated[1 + 1]]`, `q[Unevaluated[1 + 1]]`, `2`, `Hold[Plus[1, 1]]`,
`h[a, b]`, `Hold[a, b]`, `hc[Sequence[a, b]]`, `sh[Sequence[a, b]]`,
`f[a, b]`, `h[a, b]`, `2`, `Hold[Plus[1, 1]]`, `2`, `2`.

## Perf gate (mandatory — evaluator core)

`scripts/core-language-microbench.sh`, rum-vs-rum, saved binaries,
alternating runs, per-label `timings.tsv`, profiling env OFF, machine
otherwise idle. Acceptance: the affected-path labels move within the noise
floor established by the UNAFFECTED control labels (unit-8 batch-4
discipline: if controls move as much as the affected label, that IS the
noise floor — report it, do not "fix" noise). Persist raw output under
`/tmp/` and cite the files in the report.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace        # full gate — evaluator change ripples cross-crate
# the pinned-contract probe list, before AND after
# the A/B protocol output + per-label table
git status --short            # only MAY-listed files
```

## Scope

MAY modify:
- `crates/rum-runtime/src/runtime/eval_call_prepare.rs`
- `crates/rum-runtime/src/runtime/eval_core/` (minimal — the no-match
  reconstruction seam; name every file touched)
- `crates/rum-runtime/src/runtime/hold_thread.rs`
- `crates/rum-runtime/src/runtime/substitution/function_and_subst_helpers.rs`
- `crates/rum-runtime/src/runtime/builtin_adapters.rs` (ONLY the
  `Unevaluated`/`ReleaseHold` adapters if needed)
- `crates/rum-runtime/src/runtime/builtin_catalog/scoping.rs` (description
  text only)
- `crates/rum-stdlib/src/docgen.rs` + `docs/reference/builtins.md`
  (regenerated; Limits MERGE only)
- Tests: `crates/rum-stdlib/src/tests/holding_and_control_*.rs`,
  `crates/rum-runtime/tests/**` equivalents, any test file the survey names
  (list each in the report with the old→new assertion), one new test file
  allowed.

MUST NOT touch: special-form dispatch (`Rule`/`RuleDelayed` stay
evaluator-coded; their attribute sets unchanged), `binding_substitution.rs`
(batch 2's seam), the matcher, `lowering.rs`, scoping implementations,
catalog attribute sets, anything else.

## Hard rules

- Do NOT `git commit`, branch, or push. No network, no new dependencies, no
  new `unsafe`, no new attribute kinds, no runtime capability checks.
- Zero default-path cost for (a) — justify in the report.
- One variable per measurement: implement all three changes, then ONE A/B of
  candidate-vs-baseline binaries; never time with profiling env ON.
- WL comparison from documented knowledge only; where WL is not confidently
  documented (the `Unevaluated[Sequence[…]]` edge, deep-position
  `ReleaseHold`), pin-and-document or report-and-leave — never guess.
- Stop-loss: if the fixed-point stability test cannot be made to pass
  without touching files outside MAY, STOP and report the blocker instead of
  widening scope.

## Report format

1. Per-change summary: seam, mechanism, and the provenance-flag placement
   for (a) with the zero-cost justification.
2. Test survey: every old-behavior assertion found, file:line, verdict,
   old→new.
3. Pinned-contract probe list before/after, verbatim; the two
   reported-boundary rows (`ReleaseHold[g[Hold[…]]]`,
   `f[Unevaluated[Sequence[…]]]`).
4. A/B per-label table + noise-floor verdict + raw-file paths.
5. Verification outputs verbatim (fmt, full workspace test totals, git
   status).
6. Deviations / open questions (empty if none).
