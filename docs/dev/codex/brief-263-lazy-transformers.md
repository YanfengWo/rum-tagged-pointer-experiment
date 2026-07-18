# brief-263 — lazy sequences, batch 2a (transformers + Rest)

Implementation brief. Second brief of the (c) lazy-sequences line. Read
`docs/dev/lazy-sequences-design.md` and the batch-1 code
(`crates/rum-stdlib/src/lazy.rs`) before touching anything.

> Scope note: `LazyTable` (a held-body generator, different mechanics) is
> split out to batch 2b (a separate brief) so its hold/iterator-scoping
> seam does not stall the four homogeneous transformers here.

## Goal

When done: rum has four lazy transformers — `LazyMap`, `LazySelect`,
`LazyTake`, `LazyDrop` — each producing a `LazyList` (stays lazy,
composable), plus the `Rest` realizer taught to force a `LazyList`. To
compose filtering correctly, the batch-1 pull interpreter is refactored
from "collect a Vec up to budget" into a **per-element stepper** that
yields one element at a time under the iteration ceiling + cancellation.
All batch-1 lazy tests stay green. NO change to the default evaluation
path beyond one `is_lazy_list` head-check added to `eval_rest`.

## Context pointers

- Read `AGENTS.md` (Testing Policy hard lines) and
  `docs/dev/lazy-sequences-design.md` (§2 representation, §3 surface, §4
  opaque, §5 safety) first.
- Batch-1 interpreter to refactor: `crates/rum-stdlib/src/lazy.rs`
  (`realize`/`pull_descriptor`/`pull_lazy_range`, `is_lazy_list`,
  `lazy_list_descriptor`, `realize_all`/`realize_at_most`). Keep the
  public helper signatures (`is_lazy_list`, `realize_all`,
  `realize_at_most`) — the realizers in
  `associations/basic.rs`/`structure_ops/*` call them.
- Applying a function to an element (reuse pattern): stdlib evaluates
  `session.evaluate(Expr::call(f.clone(), vec![element]))`
  (`functional_ops/iteration.rs`). The interpreter (in rum-stdlib, which
  depends on rum-runtime) uses the same `session.evaluate`; a nested
  `EvalError` (Cancelled / IterationWorkLimit / ExceptionThrown / Thrown)
  propagates via `?` — DO propagate, do not swallow.
- Truthiness for `LazySelect`: reuse `Select`'s own predicate-truthiness
  decision (locate `Select`'s eval; a result is kept iff it is `True`).
  Do not invent a new truthiness rule.
- `Rest` realizer seam: `crates/rum-stdlib/src/structure_ops/surface.rs`
  (`eval_rest`, ~127) — add the same single `is_lazy_list` head-check
  branch batch-1 added to `eval_first`.
- New builtins register exactly as batch 1 did: shims in `lazy.rs`,
  catalog entries in `catalog/lazy.rs`, attribute-mirror rows in
  `builtin_catalog/stdlib_attributes.rs` (alphabetical; vocabulary test
  `catalog_attributes_match_kernel_vocabulary` enforces catalog↔mirror).
- Acceptance is mechanical: the named tests + probes decide.

## The stepper refactor (the core of this batch)

Replace the batch-1 "collect Vec to budget" realize path with a
**pull-one stepper** over the descriptor tree:

- A stepper exposes `next(&mut self, session, ctx) -> Result<Option<Expr>,
  EvalError>` — `Some(element)`, `None` when the stream is exhausted, or
  an `EvalError` propagated from a nested eval / a limit / cancel. `ctx`
  carries the iteration-work limit, a running "source pulls" counter, and
  the interrupt handle.
- Build a stepper from a descriptor (recursively):
  - `LazyRange[start, step]` — a SOURCE: each `next` charges ONE unit
    against the ceiling counter and checks cancel, yields `current`,
    advances `current += step`. Never exhausts (infinite).
  - `LazyMap[f, inner]` — hold an inner stepper; `next` = pull inner; on
    `Some(x)` return `session.evaluate(f[x])?`; on `None` return `None`.
  - `LazySelect[pred, inner]` — hold an inner stepper; `next` LOOPS:
    pull inner; on `Some(x)` evaluate `pred[x]`, return `x` if truthy,
    else keep pulling; on `None` return `None`. (The inner pulls each
    charge the ceiling via the source, so a predicate that never matches
    hits the limit and fails closed — MUST NOT hang.)
  - `LazyTake[inner, n]` — hold an inner stepper + remaining count `n`;
    `next` = if `remaining == 0` return `None`, else `remaining -= 1` and
    return inner's `next`. This is what makes a FINITE lazy stream.
  - `LazyDrop[inner, k]` — on the first `next`, pull-and-discard `k` from
    inner (each discarded pull charges the ceiling), then delegate to
    inner thereafter.
- **Ceiling / cancel accounting lives in the SOURCE stepper** (one unit
  per element pulled from a `LazyRange`), so every filter/skip that
  over-pulls the source is bounded by `iteration_work_limit` and returns
  the same `TerminatedEvaluation["IterationWorkLimit"]` fail-closed shape.
  Cancel returns `EvalError::Cancelled`. Preserve batch-1's exact
  fail-closed and cancel behavior (its tests must stay green).
- `realize(budget)` drives the stepper: loop pulling `next`, collect into
  a `List`, stop at `budget` (`AtMost(n)`) or `None` (exhausted). Keep
  `realize_all` (UntilExhausted) and `realize_at_most(n)` as the public
  entry points with unchanged signatures.

## Public builtins (each returns a `LazyList`, stays lazy)

All are `Capability::Pure`, `{Protected}` (args evaluate — NOT hold; the
held-body `LazyTable` is batch 2b). Each unwraps the input `LazyList`'s
descriptor and rewraps the composed descriptor; descriptors never nest a
`LazyList` (the wrapper is only at top level).

- `LazyMap[f, lz]` → `LazyList[LazyMap[f, <inner desc of lz>]]`.
- `LazySelect[pred, lz]` → `LazyList[LazySelect[pred, <inner>]]`.
- `LazyTake[lz, n]` (n a non-negative Integer; also accept `UpTo[n]`) →
  `LazyList[LazyTake[<inner>, n]]`.
- `LazyDrop[lz, n]` (n a non-negative Integer) → `LazyList[LazyDrop[<inner>, n]]`.
- In every case, if the stream argument is **not** a `LazyList`, or `n`
  is invalid, the call stays **inert (returns unevaluated)** — do NOT
  auto-lift a plain `List` into a lazy stream (that would blur the opaque
  boundary), and do NOT partially construct. Fail closed.

## Rest realizer (forces the tail, stays lazy)

- `Rest[LazyList[d]]` → `LazyList[LazyDrop[d, 1]]` (a lazy value; the tail
  stays lazy — correct for infinite streams). One `is_lazy_list`
  head-check branch in `eval_rest`, mirroring batch 1's `eval_first`.

## Opaque / composition semantics (pin with tests)

- Transformers compose: `Normal[LazyTake[LazyMap[f, LazyRange[1,1]], 4]]`
  realizes a finite `List`. This is the first FINITE lazy stream — it
  makes `Normal`-on-natural-exhaustion (the batch-1 gap) testable now.
- `LazySelect` filters correctly under a bounded realizer
  (`Take[LazySelect[pred, LazyRange[1,1]], k]` pulls until `k` pass).
- A `LazySelect` whose predicate never matches, realized by `Normal`,
  fails closed at the ceiling (does not hang).
- Nested eval errors inside `f`/`pred` propagate out of the realizer.
- Snapshot round-trip of a COMPOSED descriptor (e.g.
  `LazyTake[LazyMap[f, LazyRange[3,2]], 4]`) re-derives identically.
- Result values of transformers are still opaque `LazyList` (not `_List`,
  no auto-force).

## Scope

- Files you MAY modify / create:
  - `crates/rum-stdlib/src/lazy.rs` — the stepper refactor + the four
    transformer shims + stepper arms.
  - `crates/rum-stdlib/src/catalog/lazy.rs` — catalog entries for the
    four new builtins (`.protected()`, `Capability::Pure`, declared
    message tags for any `argx`/`argform` they emit).
  - `crates/rum-runtime/src/runtime/builtin_catalog/stdlib_attributes.rs`
    — four `Spec::new("Lazy…", ATTR_PROTECTED)` mirror rows (alphabetical;
    MECHANICAL, vocabulary-enforced).
  - `crates/rum-stdlib/src/structure_ops/surface.rs` — `eval_rest`
    `LazyList` branch.
  - `crates/rum-stdlib/src/tests/lazy.rs` — new tests (keep all existing
    ones green).
- Files/areas you MUST NOT touch:
  - `crates/rum-core/src/expr.rs`; the evaluator dispatch / hot path; the
    matcher; the formatter; any builtin other than `eval_rest` + the new
    lazy builtins; `Part` / `kernel_data/structure_data/selectors.rs`
    (still deferred).
  - `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**` (except appending to
    your report; the generated reference is the director's job at the
    gate), `.git`, `Cargo.lock`.
- No unconditional default-path cost: the `eval_rest` `LazyList` branch is
  one head check; the existing list path pays at most one comparison.

## Deliverable

- The stepper interpreter (LazyRange source + LazyMap/LazySelect/LazyTake/
  LazyDrop arms) with ceiling/cancel in the source; all batch-1 lazy
  tests still green.
- `LazyMap`/`LazySelect`/`LazyTake`/`LazyDrop` builtins + `Rest` realizer.
- Catalog + attribute-mirror registration (vocabulary green; reference
  regenerates at the gate).
- Tests: composition to a finite `List` via `Normal`; `LazySelect`
  filtering under a bounded `Take`; never-matching `LazySelect` +
  `Normal` fail-closed at ceiling; `LazyDrop`/`Rest`; nested eval-error
  propagation; snapshot round-trip of a composed descriptor; opacity of
  transformer results.

## rum / Rust pitfalls (read twice)

1. Timing numbers come from `scripts/core-language-microbench.sh` only;
   profiling env OFF; A/B is base-build vs candidate per label.
2. A probe is the label's EXACT statement — no sink added.
3. Default-path cost is unconditional cost: the `eval_rest` branch is one
   head check; the list path is unchanged. No `Mutex`/allocation added to
   the existing path.
4. Evaluator flows are suspension-safe: the interpreter calls
   `session.evaluate` per pulled element — propagate every `EvalError`
   via `?`; do not collapse or swallow suspensions/cancels/exceptions.
5. Hot paths avoid owned-tree clones: `Expr`/`SharedExpr`/`SharedArgs`
   are `Arc`-shared; clone `f`/`pred` with `Arc` semantics, borrow-
   traverse descriptors; do not deep-clone the descriptor per pulled
   element.
6. One variable per measurement; rerun the same label + same-tier
   neighbors.
7. Stop-loss is real: if the stepper refactor or the ceiling accounting
   fights you twice for the same reason, STOP and report — do not keep
   tuning.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test --workspace
```

If any structure-op microbench label routes through `eval_rest`/`eval_first`
(a `rest`/`first` label), run the A/B and paste those rows + same-tier
neighbors:

```
bash scripts/core-language-microbench.sh --output-dir <base-dir>   # BASE
bash scripts/core-language-microbench.sh --output-dir <new-dir>    # change
```

Also paste, as live `rum-cli --eval` probes, the verbatim output of:

- `Normal[LazyTake[LazyRange[1, 1], 5]]` → `List[1, 2, 3, 4, 5]`
  (finite realize via natural exhaustion).
- `Normal[LazyTake[LazyMap[Function[x, x^2], LazyRange[1, 1]], 4]]` →
  `List[1, 4, 9, 16]`.
- `Take[LazySelect[EvenQ, LazyRange[1, 1]], 3]` → `List[2, 4, 6]`.
- `Normal[LazyTake[LazyDrop[LazyRange[1, 1], 3], 3]]` → `List[4, 5, 6]`.
- `First[Rest[LazyRange[10, 5]]]` → `15`.
- `Normal[LazySelect[Function[x, False], LazyRange[1, 1]]]` → a
  `TerminatedEvaluation[...]` (fail-closed, returns promptly).
- `MatchQ[LazyTake[LazyRange[1, 1], 3], _LazyList]` → `True`.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes
  in the working tree; the director gates and commits.
- Do NOT modify files outside Scope; if you must, STOP and report.
- Touch nothing outside this repo; no network; only file edits within
  Scope, `cargo`, release `rum-cli`, the microbench scripts, `python3`.
- Testing Policy is a hard line: rum's own documented behavior only; no
  external-engine comparison / oracle / compatibility matrix.
- No new `unsafe` (rum-core / rum-runtime are zero-unsafe). If convinced
  it is unavoidable, STOP and report.
- Minimal diff: no drive-by reformatting or renames outside the task.
- Paste real verbatim verification output; if a run did not reach green,
  say so — never fake it.

## Report format (final message)

1. What changed — per file, line ranges, one-line why (call out the
   stepper refactor shape).
2. Verification — each command with verbatim output (test counts, probes,
   any A/B rows).
3. Verdict — confirm composition, `LazySelect` filtering + fail-closed,
   nested-error propagation, batch-1 tests still green.
4. Stop-loss / open questions (empty if none).
