# brief-348 — rhs `Condition` guards, step 1: the substrate and `DownValues`

## Goal

Make `f[x_] := rhs /; test` work for `DownValues`. Today the guard is stored
as part of the body and never evaluated:

```text
b[x_] := x^2 /; x > 0
b[3]   ->  9 /; True      (must be 9)
b[-3]  ->  9 /; False     (must be b[-3])
```

Build the shared guard substrate this and the later lanes will use, then wire
it through the complete `DownValue` chain: installation, identity, ordering,
application, clearing, and inspection. **Only `DownValues`.** `UpValues`,
`SubValues`, `DefaultValues`, `OwnValues`, generic rewrite rules,
`ReplacePart`, string rules, and the exception-handler scan are later briefs —
leave them exactly as they are, including their current wrong behavior.

## Context pointers

- Read `AGENTS.md` and `docs/guide/patterns-and-rules.md` first.
- Parsing and lowering are already correct: `lhs := rhs /; test` lowers to
  `RuleDelayed[lhs, Condition[rhs, test]]` (`lowering/binary.rs:64-67`,
  right-associative per `rum-syntax/src/ast.rs:383-400`). The omission is
  entirely downstream.
- `crates/rum-runtime/src/runtime/model/definitions.rs:71-80` — `DownValue`
  has no guard field; the whole `Condition` lands in `rhs`.
- `crates/rum-runtime/src/runtime/definitions/definition_tags.rs:206-219` —
  installation compiles the lhs matcher and stores the rhs whole; `:302-325`
  `replace_or_insert_down_value` and `equivalent_down_value_index` decide
  identity and slot.
- `crates/rum-runtime/src/runtime/definitions/definition_analysis_helpers.rs:487-495`
  — `lhs_patterns_equivalent` / `canonicalize_pattern_names`, today's
  lhs-only identity.
- `crates/rum-runtime/src/runtime/definitions/application/down_values.rs:302-329`
  — where the matched rhs is substituted and evaluated.
- **The working model to copy**: the pattern-position guard,
  `pattern_matching/core.rs:498` (`MatcherNode::Condition`) and `:125-137`
  (substitute bindings, evaluate, accept only exact `True`). Read it before
  designing anything; a second, divergent guard mechanism would be worse than
  the bug.
- Acceptance is mechanical: the named tests and A/B rows decide.

## Deliverable

### The substrate

- A private module holding: split a stored rhs into `(body, ordered guards)`,
  rebuild the `Condition` form from them, and evaluate a guard sequence to
  accept/decline. Nested guards flatten to one ordered sequence evaluated
  left to right — both the parser's right-nested
  `Condition[body, Condition[a, b]]` and an explicit
  `Condition[Condition[body, a], b]` normalize to body plus `[a, b]`.
- **Only exact `System\`True` accepts**; every other returned value declines.
  This is the pattern-side rule at `core.rs:133-137` — reuse its evaluator
  rather than writing a second one, and match its behavior for a guard that
  errors, returns unevaluated, or suspends. Report what that behavior is.
- A guard is evaluated with the matched bindings substituted, in the same
  `OptionValueScope` as the body it guards.

### `DownValues`

- `DownValue` carries the body and an ordered guard sequence; an empty
  sequence is the unguarded case and must be indistinguishable from today.
- `SetDelayed` splits the raw rhs. `Set` evaluates the complete rhs **first**,
  then splits — the immediate-rhs contract is unchanged.
- Identity is lhs **plus** the ordered guard sequence, alpha-canonicalized
  jointly (collect the renaming from the lhs, apply that one map to both).
  An empty sequence is distinct from every non-empty one, so
  `Condition[body, True]` is a separate candidate from an unguarded rule.
  Same lhs and same guards is still a redefinition and keeps its slot.
  Specificity ordering stays lhs-only; equal-specificity new entries append.
- On an lhs match, evaluate the guards in order; all accepting selects the
  rule, any declining moves to the next candidate exactly as an lhs mismatch
  does.
- `Unset` / clearing stay lhs selectors: they remove every guard variant.
- `DownValues[f]` and `Definition[f]` rebuild the `Condition` form, so
  inspection output is unchanged for unguarded rules and shows the guard for
  guarded ones.

### Tests

Assert exact resulting values or `DownValues` strings, not just absence of
error:

- `b[x_] := x^2 /; x > 0` — `b[3]` is `9`, `b[-3]` is `b[-3]`, messages empty.
- Coexistence and order: `f[x_] := "positive" /; x > 0` then
  `f[x_] := "other"` — `f[3]` is `"positive"`, `f[-3]` is `"other"`, and
  `DownValues[f]` has **two** entries.
- Redefinition: installing the same lhs with the same guard replaces in place.
- Nested guards, both spellings, and a two-guard rule where the first accepts
  and the second declines.
- Non-boolean guard results: a symbol, an integer, an unevaluated call — each
  declines.
- A guard that errors, and a guard that suspends (`InputString` inside the
  guard) — matching the pattern-side behavior you found.
- A guard with a side effect (`Print`) — the effect happens; guards run user
  code by design.
- `Set` with a guarded rhs: the rhs evaluates first, then splits.
- `Unset[b[x_]]` removes all guard variants.
- `DownValues` round-trip: the rebuilt form re-installs to the same behavior.
- Unguarded behavior unchanged — the existing suites are the assertion.

### A/B

Base versus candidate rows for at least:

```
downvalue-exact-hit      downvalue-exact-miss
downvalue-pattern-hit    downvalue-pattern-miss
downvalue-exact-miss-repeat  downvalue-pattern-miss-repeat
matcher-single-tiny      matcher-single-wide-small   matcher-single-wide-large
set-ownvalue             symbol-ownvalue-read
```

Add stable labels for a guarded hit and a guarded decline-then-fallback.

## Scope

- Files you MAY modify:
  - a new guard module under `crates/rum-runtime/src/runtime/definitions/`
    and its `mod.rs` registration
  - `crates/rum-runtime/src/runtime/model/definitions.rs`
  - `crates/rum-runtime/src/runtime/definitions/definition_tags.rs`
  - `crates/rum-runtime/src/runtime/definitions/definition_analysis_helpers.rs`
  - `crates/rum-runtime/src/runtime/definitions/definition_clear.rs`
  - `crates/rum-runtime/src/runtime/definitions/application/down_values.rs`
  - `crates/rum-runtime/src/runtime/definitions/definition_catalog/` entries
    needed for `DownValues`/`Definition` reconstruction
  - runtime tests under `crates/rum-runtime/src/runtime/tests/`
  - `scripts/core-language-microbench.sh` for the two new labels
- **Mechanical ripple is in scope but must be reported.** Adding a field to
  `DownValue` will break exhaustive literals, equality, inspection, package
  save, GC traversal, and memory accounting. Fixing those compile sites is
  permitted; for each file you touch outside the list above, say in your
  report why it was mechanically forced. If a site needs a real decision
  rather than a mechanical fix, STOP and report instead of deciding it.
- **The general accept/stop criterion** — use this, not the file list, to
  judge whether a change is in bounds: *any behavior change that occurs only
  when a guarded definition exists is legitimate; any behavior change
  observable with no guarded definition present is a stop.*
- Files/areas you MUST NOT touch: `UpValues`, `SubValues`, `DefaultValues`,
  `OwnValues`, generic rewrite rules, `ReplacePart`, string rules, exception
  handlers, `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**`, `.git`,
  `Cargo.lock`, `Expr`/`Symbol` representation.

## rum / Rust pitfalls (read twice)

1. **The unguarded path must cost nothing.** An empty guard sequence must not
   add a per-candidate branch, allocation, or clone to rule selection — the
   hottest path in the evaluator. Prefer keeping the unguarded matcher shape
   byte-for-byte and letting guarded entries alone take a slower lane. If your
   design cannot avoid a per-candidate check, STOP and report.
2. **Guards run user code**, so guard evaluation must be suspension-safe:
   preserve the `suspend_with_frame*` continuation shape, and make sure a
   suspension inside a guard resumes into the same candidate cursor rather
   than restarting the scan or skipping candidates.
3. **Reuse the pattern-side guard evaluator.** Two guard mechanisms that drift
   apart is a worse outcome than the current bug.
4. **Identity is syntactic after alpha-canonicalization.** Do not simplify
   `True`, reorder guards, or use semantic equivalence.
5. **Do not change what a bare `Condition` expression evaluates to.**
6. **Counters before timings** in the A/B, and name the untouched control
   labels you judged against — never the suite median.
7. **The microbench runs to `--scale 10`** and fails loudly above it
   (`d9c713e1`); a missing or header-only `timings.tsv` means the run failed.
8. **Stop-loss is real.** If the same step fails twice for the same reason, or
   an unguarded label regresses and one focused attempt does not recover it,
   STOP and report.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test -p rum-runtime
cargo test -p rum-stdlib
cargo test --workspace
```

Then the A/B: build base and candidate release binaries, run the SAME
microbench script against both, best-of-2, and paste the per-label rows.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; the director runs the authoritative gate and commits.
- Touch nothing outside this repo; no network.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only; no third-party engine comparison anywhere.
- **No new `unsafe`** — `rum-core` and `rum-runtime` are zero-`unsafe`.
- No new dependencies.
- Bounded runs: `scripts/run-bounded.py --timeout SECONDS -- <command>`, with
  the bound derived from a measured run of the same shape.
- Minimal diff: no drive-by reformatting or renames.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. What changed — per file, with line ranges and a one-line why, with the
   mechanically-forced files listed separately and justified.
2. The guard evaluator's behavior for error / unevaluated / suspending
   guards, and where you found the pattern-side rule you matched.
3. Verification — each command with verbatim output.
4. The exact-value test matrix (guarded hit, decline-and-fallback,
   coexistence, nested, non-boolean, `Set`, `Unset`, round-trip).
5. The A/B — counters first, then base/candidate rows with the control labels
   named.
6. Stop-loss / open questions.
