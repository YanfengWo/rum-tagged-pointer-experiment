# Brief 120 — slimming coda: the four 30k+ split-plan files (assess-first)

## Goal

Split — or explicitly assess-and-keep — the four largest remaining
split-plan-band files, per the JUST-REVISED policy
(`docs/code-organization-guidelines.md`, commit 4c88a44 — read the revised
§File Size And Splitting Policy, §Directory Fan-Out, §Splitting Is
Organization, and brief-114's principles which the new section codifies):

1. `crates/rum-runtime/src/runtime/control_data/branching_loops/iteration.rs` (34,310)
2. `crates/rum-runtime/src/runtime/model.rs` (33,896)
3. `crates/rum-runtime/src/runtime/scoping/lexical.rs` (32,024)
4. `crates/rum-runtime/src/runtime/eval_core.rs` (31,579)

IMPORTANT — this brief practices the new policy's flexibility: for EACH file,
first perform the assessment (does it still hold ONE coherent
responsibility?). If a file's cohesion assessment concludes that splitting
would fragment a genuinely single responsibility, KEEP it and record the
assessment (why, and what boundary would be used if it grows) — that is a
valid, first-class outcome. Do not force a split to hit a number. Remember
the in-file-test rule: extract any `#[cfg(test)]` module to a sibling
`tests.rs` FIRST and re-measure before judging.

Zero behavior change; workspace green (1301/0); **microbench A/B required**
(all four files are eval-path or eval-adjacent; 115/116/118-style: base
before edits, rebuild release rum-cli before the post run, one rerun
allowed; judged labels: the full do-*/table-*/downvalue-*/module-local/
block-*/if-true set + table-iterator-plus as the standing sentinel).

## Context pointers

- `iteration.rs`: Do/Table/Sum/Product iterator machinery (spec parsing,
  the iterator Block-wrapper drive loop, aggregation). Plausible axis:
  iterator-spec parsing vs the drive/resume loop vs aggregation builders.
  HOT (the fast path restored in brief-113 lives here — its
  attribute-equality check and surroundings byte-stable).
- `model.rs`: the Session struct + config/limits/trace/message/activation
  types. Plausible axis: session-state struct vs config types vs
  trace/message types. COLD-ish (types), but everything includes it —
  stable/churn separation is the payoff here.
- `scoping/lexical.rs`: lexical binding/substitution for Module/With.
  Plausible axis: binding construction vs substitution application vs
  escape analysis. HOT.
- `eval_core.rs`: the evaluator main loop. THE most likely
  assess-and-keep candidate — if the loop plus its direct orchestration is
  one responsibility, say so and keep it. If there are genuinely separable
  domains (e.g. symbol evaluation vs call orchestration), split only along
  those.
- All standing rules: split map first (or assessment instead), verbatim
  bodies, minimal visibility growth, facade keeps callers unchanged,
  sibling DAG, per-file ≤20k for whatever you do split (cohesion exceptions
  justified inline).

## Scope

- MAY modify: the four files (→ directories where split; in-place where
  kept), their in-file test extractions, `use` lines inside new directories,
  parent mod declarations only if syntax demands.
- MUST NOT touch: any caller outside, any behavior, `Cargo.toml`/
  `Cargo.lock`, any `mma-hol` clone, `docs/**` except your report, `.git`.

## Verification (MANDATORY)

```
bash scripts/core-language-microbench.sh --engine rum --output-dir /tmp/cdx_brief_120_micro_base   # BEFORE any edit
# ... work ...
cargo fmt --check
cargo test --workspace
cargo build --release -p rum-cli
bash scripts/core-language-microbench.sh --engine rum --output-dir /tmp/cdx_brief_120_micro_new
# per-file outcome evidence: for split files the size table; for kept files the assessment
find crates/rum-runtime/src/runtime -name "*.rs" | while read f; do c=$(wc -c < "$f"); if [ $c -gt 20000 ]; then echo "$c $f"; fi; done | sort -rn
git status --short -- crates/
```

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Do NOT modify files outside Scope; STOP and report if you believe you must.
- Nothing outside this repo, no network; only Scope edits, `cargo`, the
  microbench script, `grep`, `python3`. Never edit any `mma-hol` file.
  **No new `unsafe`.**
- Probe budget: two A/B runs + one rerun.
- Stop-loss: sentinel or judged-label consistent regression, callers
  needing changes, same compile error twice → STOP and report.

## Report format

1. PER-FILE VERDICT FIRST: split (with map) or kept (with assessment) —
   four entries.
2. What changed — inventory; untouched-callers evidence.
3. Verification — verbatim outputs incl. the 20k+ audit and A/B table with
   sentinel commentary.
4. Verdict — how the revised policy performed in practice; anything for the
   directory fan-out batch (next: runtime/ has 75 direct children).
5. Stop-loss / open questions.
