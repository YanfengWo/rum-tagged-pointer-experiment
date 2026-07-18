# brief-264 — lazy sequences, batch 2b (LazyTable)

Implementation brief. Third brief of the (c) lazy-sequences line. Read
`docs/dev/lazy-sequences-design.md` and the batch-2a code
(`crates/rum-stdlib/src/lazy.rs`) before touching anything.

## Goal

When done: rum has `LazyTable[body, iterspec]` — a lazy `Table` over an
arithmetic iterator — implemented as a **pure desugaring** onto the
batch-2a primitives, with NO new stepper arm and NO kernel seam. `body`
is held until realization; each element evaluates `body` with the
iterator bound. Opaque `LazyList` result, `Pure`, snapshot round-trips.

## Key design (desugaring — the whole mechanism)

`LazyTable` needs no new interpreter code. The iterator binding is exactly
what the batch-2a `LazyMap` stepper already does (it evaluates
`session.evaluate(f[element])`, and `f = Function[i, body]` binds `i`
lexically). So `eval_lazy_table` desugars:

```
LazyTable[body, {i, imin, imax, step}]
  =>  LazyList[ LazyTake[
        LazyMap[ Function[i, body], LazyRange[imin, step] ],
        count ] ]
```

- The descriptor uses the SAME inert heads the batch-2a stepper already
  interprets (`LazyTake`/`LazyMap`/`LazyRange` + `Function`), so
  `stepper_from_descriptor` handles it with zero changes.
- `body` stays held (LazyTable is `HoldAll`; `Function` holds its body),
  so the inner `Function[i, body]` never evaluates until the Map stepper
  applies it per element. Folk-scope lexical binding via `Function` — do
  NOT reach for the kernel `Table`/`Do` iteration machinery (it is
  `pub(in crate::runtime)`, unreachable from stdlib, and not needed).

## Iterator spec + count

`eval_lazy_table` is `HoldAll`: args arrive unevaluated. Parse the spec
form, EVALUATE the bound expressions (via `session.evaluate`) to exact
rationals, keep `i` (the iterator symbol) and `body` HELD.

- Accepted forms (arithmetic only this batch):
  - `{i, imax}` → imin = 1, step = 1
  - `{i, imin, imax}` → step = 1
  - `{i, imin, imax, step}`
- `count` = number of terms:
  - step > 0: `count = max(0, floor((imax - imin)/step) + 1)`
  - step < 0: symmetric (`floor((imin - imax)/(-step)) + 1`, clamped ≥ 0)
  - An EMPTY range (e.g. `{i, 5, 1}`) is VALID → count 0 → realizes to
    `List[]` (NOT an error — matches `Table`).
- Fail closed (inert unevaluated + a declared message, no partial value)
  when: the spec is not one of the accepted forms; `i` is not a symbol;
  a bound does not evaluate to an Integer/Rational; or `step == 0`.
- The `{i, {list}}` explicit-enumeration form is OUT of scope this batch
  (it needs a non-arithmetic source; note it deferred, do not add it).

## Scope

- Files you MAY modify / create:
  - `crates/rum-stdlib/src/lazy.rs` — `eval_lazy_table` shim (desugaring +
    spec parse + count). No stepper change is expected; if you think one
    is needed, STOP and report (the desugaring should make it unnecessary).
  - `crates/rum-stdlib/src/catalog/lazy.rs` — `LazyTable` catalog entry:
    `BuiltinSpec::ordinary(...)` with attributes `{HoldAll}`, `.protected()`,
    and declared message tags for whatever it emits (`argx`/`argform`).
  - `crates/rum-runtime/src/runtime/builtin_catalog/stdlib_attributes.rs`
    — mirror row `Spec::new("LazyTable", ATTR_HOLD_ALL_PROTECTED)`
    (`ATTR_HOLD_ALL_PROTECTED` already exists; alphabetical; MECHANICAL,
    vocabulary-enforced).
  - `crates/rum-stdlib/src/tests/lazy.rs` — new tests (keep existing green).
- Files/areas you MUST NOT touch:
  - `crates/rum-core/src/expr.rs`; the evaluator/matcher/formatter; the
    batch-2a `PullStepper` / realize path (reuse it unchanged); any other
    builtin; `Part` / kernel selectors (still deferred).
  - `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**` (except appending to
    your report; reference regen is the director's job), `.git`, `Cargo.lock`.
- No default-path cost (this is a new cold builtin; nothing on the hot
  path changes).

## Deliverable

- `eval_lazy_table` desugaring + `LazyTable` builtin (`HoldAll`, `Pure`,
  `Protected`), registered (catalog + mirror; vocabulary green; reference
  regenerates at the gate).
- Tests: `Normal[LazyTable[i^2, {i, 1, 5}]]` → `List[1, 4, 9, 16, 25]`;
  stepped form `{i, 2, 10, 2}`; two-arg `{i, imax}`; empty range → `List[]`;
  body held until realization (a side-effect probe proving `body` does not
  evaluate at construction); malformed spec / non-rational bound / step 0
  fail closed with the declared message; result is opaque `_LazyList`;
  snapshot round-trip of a `LazyTable` value re-derives.

## rum / Rust pitfalls (read twice)

1. `LazyTable` is `HoldAll`: the shim must evaluate ONLY the bound
   expressions (imin/imax/step) and keep `i` + `body` held. Build
   `Function[i, body]` with the held `body` — do not evaluate `body`.
2. Reuse batch-2a: the desugared descriptor is interpreted by the
   existing stepper; do not duplicate realization logic.
3. Default-path cost: none — a new cold builtin. Do not touch the hot
   path.
4. Evaluator flows are suspension-safe: realization goes through the
   existing stepper, which already propagates `EvalError` via `?`.
5. Hot paths avoid owned-tree clones: build the descriptor with
   `Expr::call`; `Arc`-clone `body`, do not deep-clone.
6. Exact rationals only (no float); reuse `expr_as_exact_rational` /
   `expr_as_non_negative_usize` as batch-2a does.
7. Stop-loss is real: if the desugaring does not compose cleanly with the
   existing stepper, STOP and report rather than adding a stepper arm.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test --workspace
```

No hot-path code changes, so no microbench A/B is required (state this).

Live `rum-cli --eval` probes, verbatim:

- `Normal[LazyTable[i^2, {i, 1, 5}]]` → `List[1, 4, 9, 16, 25]`.
- `Normal[LazyTable[i, {i, 2, 10, 2}]]` → `List[2, 4, 6, 8, 10]`.
- `Normal[LazyTable[i, {i, 3}]]` → `List[1, 2, 3]`.
- `Normal[LazyTable[i, {i, 5, 1}]]` → `List[]` (empty range, no error).
- `Take[LazyTable[i^2, {i, 1, 1000000}], 3]` → `List[1, 4, 9]` (large
  bound, but bounded realize is cheap — the descriptor is lazy).
- `MatchQ[LazyTable[i, {i, 1, 3}], _LazyList]` → `True`.
- A held-body probe of your choice proving `body` does not evaluate until
  realization.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes
  in the working tree; the director gates and commits.
- Do NOT modify files outside Scope; if you must, STOP and report.
- Touch nothing outside this repo; no network; only file edits within
  Scope, `cargo`, release `rum-cli`, `python3`.
- Testing Policy is a hard line: rum's own documented behavior only.
- No new `unsafe`. If convinced it is unavoidable, STOP and report.
- Minimal diff: no drive-by reformatting or renames outside the task.
- Paste real verbatim verification output; if a run did not reach green,
  say so — never fake it.

## Report format (final message)

1. What changed — per file, line ranges, one-line why (confirm no stepper
   change was needed).
2. Verification — each command with verbatim output (test counts, probes).
3. Verdict — confirm held body, empty range, fail-closed spec, opacity,
   snapshot round-trip; batch-2a tests still green.
4. Stop-loss / open questions (empty if none).
