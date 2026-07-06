# Brief 152 — Close the stack-guard coverage gaps (the two brief-151 overflows)

Implementation brief with a mandatory diagnostic Phase A (the brief-131
shape). Two reproducible native stack overflows escape the eval-loop stack
guard; both must become recoverable outcomes. Hot-path discipline applies:
if the fix touches eval/dispatch code, the microbench A/B gate is
mandatory.

## The two bugs (from brief-151; ignored stubs document them)

1. `Fold[Plus, 0, Map[Length, GatherBy[Select[Range[50000], Function[EvenQ[#]]], Function[Mod[#, 10]]]]]`
   → process abort by native stack overflow (a LEGAL workload, no guard
   trip). Stub: `bug_list_pipeline_range_50000_overflows_native_stack`
   in `crates/rum-stdlib/src/tests/stress_corpus.rs`.
2. ~100-500 generated definitions
   (`SetAttributes[snapN, Orderless]; snapN[x_] := x + N`) →
   `snapshot_to_bytes` → fresh session → `restore_from_bytes` → abort.
   Stub: `bug_snapshot_restore_500_generated_definitions_overflows_native_stack`
   in `crates/rum-host/tests/stress.rs`.

## Phase A — diagnose before changing anything (paste evidence)

For EACH bug:
- Minimize: bisect the pipeline stages / definition counts to the smallest
  reproducer and the exact failing stage (e.g. is it `GatherBy` alone at
  50k? `Map`? the composite? — is it `snapshot_to_bytes` or
  `restore_from_bytes`, and does the overflow scale with definition count
  or with image size?).
- Identify the recursing function(s): run the minimized reproducer in a
  scratch build under `scripts/run-bounded.py` and capture the crash
  backtrace (macOS crash report or `lldb --batch -o run -o bt`); name the
  frames that repeat. State per-frame native cost if measurable (the
  brief-131 address-delta probe pattern).
- State the mechanism in one sentence each: which recursion never passes
  the eval-loop poll point (and therefore never sees the
  `native_stack_limit` check).

## Phase B — fix per findings

Preferred designs, in order:
1. **Extend the existing headroom check into the offending recursion**:
   the brief-131 stack-base/headroom machinery should be callable from
   kernel-internal recursion sites (per-element helper recursion,
   serializer/parser recursion); insert an amortized check (every N
   iterations/frames) that surfaces the SAME recoverable
   `EvalError::StackLimitExceeded` → `TerminatedEvaluation["StackLimit"]`
   path. No new limit categories.
2. Where the recursion is COLD and simple (e.g. a serializer walking
   definitions), an iterative rewrite is acceptable instead — but recall
   the brief-056/057 lesson: iterative rewrites of hot drop/eval paths
   regressed 20-50% and were rejected; do NOT attempt an iterative `Drop`
   or eval-loop restructure here.
3. The snapshot bug may alternatively be a SHAPE problem (e.g. the restore
   image parsed/evaluated as one deeply nested expression) — if so, fixing
   the image shape (flat statement sequence) is the right fix; the
   round-trip contract (restored definitions evaluate identically) must
   hold and be asserted.

Then: convert both `bug_*` stubs into REAL tests — bug 1's pipeline runs
to its exact expected value (compute it independently in the test); bug 2's
500-definition snapshot round-trip succeeds with a sampled-definition
equivalence check. Un-`#[ignore]` them if they fit the ≤5s budget, else
keep ignored-but-real with doc comments.

## Scope

- MAY modify: `crates/rum-runtime/src/**` (the recursion sites Phase A
  names, the stack-guard helper's visibility for kernel-internal use,
  snapshot/persist image shape), `crates/rum-stdlib/src/**` (builtin
  helper recursion sites, the stress stubs), `crates/rum-host/**` (stress
  stubs; snapshot plumbing only if Phase A points there), guide files ONLY
  if a documented contract changes (state it).
- MUST NOT touch: matcher internals beyond a named recursion site,
  `Cargo.*`, other docs.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
cargo run -q -p rum-cli -- --eval 'Fold[Plus, 0, Map[Length, GatherBy[Select[Range[50000], Function[EvenQ[#]]], Function[Mod[#, 10]]]]]'   # exact value, no crash
# the converted stress tests, run explicitly:
scripts/run-bounded.py --timeout 600 -- cargo test -p rum-stdlib stress_corpus -- --include-ignored
scripts/run-bounded.py --timeout 600 -- cargo test -p rum-host --test stress -- --include-ignored
```

Microbench A/B (mandatory if ANY eval/dispatch/builtin-helper hot path was
touched — Phase A will tell; when in doubt, run it):

```
bash scripts/core-language-microbench.sh --output-dir target/brief-152-base   # on BASE
bash scripts/core-language-microbench.sh --output-dir target/brief-152-new
# paste per-label rum_seconds for both; any hot-label regression is a defect
```

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. No network, no
  new dependencies, **no new `unsafe`**.
- Amortized checks only — no per-element unconditional cost on hot paths
  (the default-path rule); the A/B gate is the proof.
- Remove all Phase-A instrumentation before finishing.
- Stop-loss: same failure twice, or Phase A contradicts every preferred
  design → STOP and report with the evidence.
- Paste real verbatim verification output; never claim green without it.

## Report format (your final message)

1. Phase A findings per bug — minimized reproducer, recursing frames, the
   one-sentence mechanism.
2. What changed — per file, with the chosen design per bug.
3. Verification — all commands verbatim (incl. the A/B table if run).
4. Stop-loss / open questions (empty if none).
