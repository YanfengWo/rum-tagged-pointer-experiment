# brief-343 — make iterator bindings visible to held-lhs normalization

## Goal

`Do[f[i] = i, {i, 1, 3}]` must install `f[1] :> 1`, `f[2] :> 2`, `f[3] :> 3`
instead of the single raw `HoldPattern[f[i]] :> 3` it installs today, and the
same for `Table`, `Sum`, and `Product`. The repair is one consistency fix:
ordinary symbol evaluation resolves a symbol through `self.definitions` and
*then* through `dynamic_own_values`, while the held-lhs normalizer used by
`Set`/`SetDelayed` consults only `self.definitions`. Give the normalizer the
same second step, in the same order.

Nothing else changes. Read-side lookup, hold semantics, the iterator fast
path, and every other binder keep their current behavior.

## Context pointers

- Read `AGENTS.md` (operational notes, Testing Policy hard lines) and
  `docs/guide/scoping-and-contexts.md` before touching anything.
- **The defect, verbatim** (release CLI at `8d2eda2d`):

  ```text
  ClearAll[j, cleanF, dirtyF]
  Do[cleanF[j] = j, {j, 1, 2}]     -> DownValues[cleanF] = {HoldPattern[cleanF[j]] :> 2}
  j = 99
  Do[dirtyF[j] = j, {j, 1, 2}]     -> DownValues[dirtyF] = {HoldPattern[dirtyF[1]] :> 1,
                                                            HoldPattern[dirtyF[2]] :> 2}
  ```

  The pre-existing `j = 99` makes the iterator symbol "dirty", so the iterator
  takes its fallback branch and installs a real `SymbolDefinition` — and the
  unchanged normalizer then sees it. That is the decisive experiment: the
  fallback semantics are already correct, only the clean path is invisible.

- **The mechanism**, confirmed by the director:
  - `crates/rum-runtime/src/runtime/eval_core/symbol.rs:65-91` — ordinary
    symbol evaluation: `own_value`, then `delayed_own_value`, then
    `dynamic_own_value_binding`.
  - `crates/rum-runtime/src/runtime/eval_core/symbol.rs:125-134` —
    `dynamic_own_value_binding(&self) -> Option<Option<Expr>>`: outer `None`
    means no binding; inner `None` means bound with no value.
  - `crates/rum-runtime/src/runtime/definitions/definitions.rs:580-604` —
    `resolve_definition_lhs_head_value`: the same resolution, missing the
    third step. **This is the line to change.**
  - `crates/rum-runtime/src/runtime/definitions/definitions.rs:345-346` —
    where `Set` calls `normalize_definition_lhs_call_heads`.
  - `crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver.rs:252-278`
    — the clean-symbol fast path that pushes the dynamic frame;
    `iteration_aggregate.rs:125-130` — `Sum`/`Product` share it.

- **Exposure is provably small.** `push_dynamic_own_value_binding` has exactly
  three non-test call sites:
  `scoping/lexical/block.rs:67` and `:105` both pass `None`, and
  `iteration/driver.rs:277` passes `Some(value)`. So only eager iterators can
  make the new step resolve anything; uninitialized `Block` keeps binding
  with no value and its held-lhs behavior must stay byte-identical. **Verify
  this inventory yourself** and report it — if you find a fourth site, say so
  before changing anything.

- Acceptance is mechanical: the named tests and the named A/B rows decide.

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/definitions/definitions.rs`
  - `crates/rum-runtime/src/runtime/tests/definitions_and_rules.rs`
  - `crates/rum-runtime/src/runtime/tests/definitions_and_rules_patterns.rs`
  - `crates/rum-stdlib/src/tests/holding_and_control_part5.rs`
  - `crates/rum-stdlib/src/tests/incidental_session_requests_input_resume.rs`
  - `scripts/core-language-microbench.sh` — only to add a focused permanent
    label if the A/B needs one.
- Files/areas you MUST NOT touch: everything else, including
  `iteration/driver.rs`, `iteration_aggregate.rs`, `eval_core/symbol.rs`,
  `scoping/**`, `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**`, `.git`,
  `Cargo.lock`, and `Expr`/`Symbol` representation.
- **Explicitly out of scope for this brief** (each is a separate defect with
  its own brief coming; do not fix them here, do not add tests asserting their
  current behavior beyond the controls listed below):
  - `AssociateTo`/`KeyDropFrom` writing back to the raw target
    (`ii = 2; AssociateTo[af[ii], "k" -> 7]` installs `af[ii]` while reading
    `af[2]`) — present for plain globals too, unrelated to iterators.
  - The eleven compound assignment heads not evaluating a `Part` selector
    (`kk = 1; lst[[kk]] += 5` reports `Part::pkspec1`) — also present for
    plain globals.
  - `TagSet`/`UpSet` never normalizing lhs arguments — a standing contract
    question, deliberately unchanged.
- Do NOT add unconditional cost to the default evaluation path.

## Deliverable

- `resolve_definition_lhs_head_value` falls back to
  `dynamic_own_value_binding` **only when `self.definitions` yields nothing**,
  preserving today's precedence exactly. An outer `None` (no binding) and an
  inner `None` (bound, no value) both leave the symbol as-is. A value found
  this way goes through the same `normalize_definition_lhs_expr` +
  recursive `resolve_definition_lhs_head_value` treatment as a definitions
  value, and the `visited` cycle guard keeps working unchanged.
- Tests, as an enumerated matrix — this class of change is exactly where a
  partial matrix has hidden a defect before, so sample nothing:
  - `Do`, `Table`, `Sum`, `Product` x `Set`, `SetDelayed`, each asserting the
    exact resulting `DownValues` string. `Set` resolves the lhs key;
    `SetDelayed` resolves the lhs key and leaves the rhs delayed on the
    iterator symbol (`f[1] :> i`), which is the director's ruling.
  - Clean/dirty parity: the same iterator with and without a pre-existing
    value for the iterator symbol must now produce identical `DownValues`.
  - Nested iterators, two iterators sharing a name, and an inner iterator
    shadowing an outer one.
  - Body assigns the iterator symbol (`Do[i = 5; f[i] = 1, {i, 1, 2}]`) — a
    real definition must still win over the dynamic value, matching read-side
    precedence — and body `Clear[i]`.
  - After the loop: the iterator symbol restored exactly (unbound stays
    unbound; a pre-existing value comes back).
  - Controls that must stay byte-identical: plain global symbol, `For`,
    `Module`, `With`, initialized `Block`, **uninitialized `Block`**
    (`Block[{i}, f[i] = 1]` must still store `f[i]`), explicit `Function`,
    pattern-bound definition rhs, `LazyTable`, `list[[i]] = v`, and
    `assoc[i] = v`.
  - Suspension: an `InputString` inside the `Set` rhs within an iterator
    body — resume must not replay and must restore scope exactly.
  - Messages empty on every success case.
  - Convert at least one existing wide-DownValue fixture that works around
    this defect with `Do[With[{k = i}, f[k] = k], …]`
    (`definitions_and_rules_patterns.rs:182-200`) to the direct
    `Do[f[i] = i, …]` idiom, keeping one `With` fixture as a lexical control.
- A/B rows, base versus candidate, for the `Set`/definition labels and the
  iterator labels plus their same-tier neighbors.

## rum / Rust pitfalls (read twice)

1. **Do not change read-side precedence** in `symbol.rs`. The whole point is
   that the lhs normalizer adopts the existing order, not that the order moves.
2. **Inner `None` is not "no value to use" by accident** — it is a real
   binding with no value (uninitialized `Block`). Resolving it to anything
   would change `Block[{i}, f[i] = 1]`, which is correct today.
3. **The `visited` guard is a cycle guard, not a scope marker.** A dynamic
   value that resolves back to its own symbol must terminate exactly as the
   definitions path does. Add a test for it.
4. **This runs on every `Set`.** `dynamic_own_values` is empty outside `Block`
   and iterator bodies, so the added step must be a single empty-vector check
   on the common path — no allocation, no clone, no map lookup when there is
   no binding. If your implementation clones anything on the empty path,
   rework it.
5. **Evaluator flows are suspension-safe.** Do not collapse or reorder any
   `suspend_with_frame*` continuation.
6. **Do not reach for the binder.** A design-only survey proposed mirroring
   iterator values into `self.definitions` instead; the director rejected that
   layer for this step. If you become convinced the normalizer cannot express
   the fix, STOP and report — do not switch layers on your own.
7. **Stop-loss is real.** If a target or same-tier label regresses and one
   focused attempt does not recover it, STOP and report the rows. Do not tune
   repeatedly, and do not "fix" a regression by moving the change elsewhere.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test -p rum-runtime
cargo test -p rum-stdlib
cargo test --workspace
```

Then the A/B gate. Build a base binary from the unmodified tree and a
candidate binary from your change, and run the SAME microbench script against
both, best-of-2:

```
bash scripts/core-language-microbench.sh --output-dir <base-dir>
bash scripts/core-language-microbench.sh --output-dir <new-dir>
```

Paste the per-label `rum_seconds` rows for, at minimum:

```
do-iterator-null
table-iterator-null
table-iterator-symbol
table-iterator-plus
do-small
set-ownvalue
symbol-ownvalue-read
downvalue-exact-hit
downvalue-exact-miss
set-part-fill-small
set-part-fill-large
```

Counters before timings: if you can show with an allocation or a definition
lookup counter that the empty-`dynamic_own_values` path is untouched, report
that first — it is load-immune, and a timing number that disagrees with a flat
counter is noise, not a regression. Name the untouched control labels you
judged against; do not normalize against the suite median.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; the director runs the authoritative gate and commits.
- Do NOT modify files outside Scope. If you believe you must, STOP and write
  that in the report instead.
- Touch nothing outside this repo; no network. Commands are limited to file
  edits within Scope, `cargo`, the release `rum-cli`, the microbench scripts,
  and `python3` for parsing.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only. No black-box comparison against another vendor's engine, no
  oracle harness, no compatibility matrix, no third-party engine output.
- **No new `unsafe`.** `rum-core` and `rum-runtime` are zero-`unsafe`.
- **Every probe carries a liveness assertion.** A probe that measures a
  failing path looks perfectly well-behaved; assert the work happened (exact
  `DownValues`, a count, a printed value), never just an elapsed time.
- **Timeout discipline**: use `scripts/run-bounded.py --timeout SECONDS --
  <command>` for anything that could run long; size the bound from a measured
  run of the same shape and state the bound you chose.
- Minimal diff: no drive-by reformatting, renames, or comment churn.
- Paste real verbatim output. If a run did not reach green, say so explicitly.

## Report format (your final message)

1. What changed — per file, with line ranges and a one-line why.
2. The `push_dynamic_own_value_binding` call-site inventory you verified.
3. Verification — each command with verbatim output (test counts, the full
   matrix's exact `DownValues` assertions).
4. The A/B — counters first, then base/candidate rows for the target and
   neighbor labels, with the control labels you judged against named.
5. Stop-loss / open questions — where you stopped and why (empty if none).
