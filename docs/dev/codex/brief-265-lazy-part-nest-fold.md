# brief-265 — lazy sequences, batch 3a (Part seam + NestList + FoldList)

Implementation brief. Fourth brief of the (c) lazy-sequences line. Read
`docs/dev/lazy-sequences-design.md` and the batch-2 code
(`crates/rum-stdlib/src/lazy.rs`) before touching anything.

## Goal

Three additions that all fit the existing machinery:

1. **`Part` kernel re-dispatch seam** — `lz[[i]]` (single positive
   integer) on a `LazyList` forces the i-th element, closing the batch-1
   deferral. The kernel `eval_part` cannot call the stdlib pull
   interpreter (reverse crate dependency), so it emits a symbolic
   re-dispatch `First[LazyDrop[lz, i-1]]` and evaluates it.
2. **`LazyNestList[f, x0]`** — an infinite source stream `x0, f[x0],
   f[f[x0]], …` (a new source stepper).
3. **`LazyFoldList[f, init, lz]`** — a lazy `FoldList`: `init, f[init,x1],
   f[f[init,x1],x2], …` (a new transformer stepper over an inner
   `LazyList`).

Opaque `LazyList` results, `Pure`, snapshot round-trips, ceiling/cancel
via the source pulls (batch-2a mechanism, unchanged). All batch-1/2 lazy
tests stay green. NO default-path cost beyond one head-check in
`eval_part`.

## Part 1 — the `Part` kernel re-dispatch seam

- Kernel `eval_part`: `crates/rum-runtime/src/runtime/kernel_data/
  structure_data/selectors.rs` (`eval_part`, ~76).
- Insert a branch **right after the empty-args check (~line 88), BEFORE
  `fast_exact_integer_part_path`**: if `args.len() == 2`, `args[0]` is a
  `Call` whose head is the System symbol `LazyList`, and `args[1]` is a
  **positive Integer `i`**, then re-dispatch:

  ```
  self.evaluate( First[ LazyDrop[ args[0], i-1 ] ] )
  ```

  i.e. build `Expr::call(system First, vec![Expr::call(system LazyDrop,
  vec![args[0].clone(), Expr::integer(i-1)])])` and return
  `self.evaluate(that)`. `LazyDrop`/`First` are the stdlib builtins; the
  evaluator dispatches them (drop `i-1`, force the first) — no Rust call
  into stdlib, so no crate cycle. For `i == 1` this is
  `First[LazyDrop[lz, 0]]` → the first element.
- Fire ONLY for a single positive-integer spec. Everything else — a
  non-`LazyList` first arg, a span (`i ;; j`), a from-the-end index
  (`-1`), a multi-level spec (`[[i, j]]`), a non-positive integer — falls
  through to the EXISTING code path unchanged (which leaves `Part[lz, …]`
  inert / emits the existing message). Do NOT attempt from-the-end or
  span indexing on a lazy stream.
- Check the head via the kernel's own System-symbol-name predicate (the
  one `kernel_api::is_system_symbol_name` wraps); do NOT add a dependency
  on any stdlib helper.
- The default `Part` path (Part on a List/Association) must pay at most
  one head comparison; the existing behavior is byte-unchanged.

## Part 2 — `LazyNestList` (new source stepper)

- Public builtin `LazyNestList[f, x0]` (2 args; `Protected`, `Pure`, args
  evaluate): returns `LazyList[LazyNestList[f, x0]]`. `f` is the function,
  `x0` the (already-evaluated) initial value. `argx` on wrong arity.
- Descriptor node `LazyNestList[f, x0]`; stepper arm (a SOURCE, charges
  the ceiling per element like `Range`): holds `current` (init `x0`) and
  `f`. `next` = `charge_source_pull()?`; clone `current` as the yield
  value `v`; advance `current = session.evaluate(f[v])?`; return
  `Some(v)`. Yields `x0, f[x0], f[f[x0]], …` (infinite; a nested-eval
  error in `f` propagates via `?`).

## Part 3 — `LazyFoldList` (new transformer stepper)

- Public builtin `LazyFoldList[f, init, lz]` (3 args; `Protected`,
  `Pure`): `lz` must be a `LazyList` (else inert + `argform`); `f`/`init`
  evaluate. Returns `LazyList[LazyFoldList[f, init, <inner desc of lz>]]`.
  `argx` on wrong arity.
- Descriptor node `LazyFoldList[f, init, inner]`; stepper arm (a
  TRANSFORMER over an inner stepper): holds `f`, `accumulator` (init), an
  `emitted_init` flag, and the inner stepper. `next`:
  - if `!emitted_init`: set it, return `Some(accumulator.clone())`
    (yields `init` first, exactly like `FoldList`).
  - else pull inner; `None` → `None`; `Some(x)` →
    `accumulator = session.evaluate(f[accumulator, x])?`; return
    `Some(accumulator.clone())`.
  - Length = 1 + inner length (finite inner → finite; infinite → infinite,
    bounded by realizer/ceiling).

## Scope

- Files you MAY modify / create:
  - `crates/rum-runtime/src/runtime/kernel_data/structure_data/selectors.rs`
    — the `eval_part` `LazyList` branch (Part 1 ONLY; do not touch other
    selectors).
  - `crates/rum-stdlib/src/lazy.rs` — `eval_lazy_nest_list` +
    `eval_lazy_fold_list` shims; two new `PullStepper` variants
    (`NestList`, `FoldList`) + their `next` arms + `stepper_from_descriptor`
    cases.
  - `crates/rum-stdlib/src/catalog/lazy.rs` — catalog entries for
    `LazyNestList`, `LazyFoldList` (`.protected()`, declared message tags).
  - `crates/rum-runtime/src/runtime/builtin_catalog/stdlib_attributes.rs`
    — two `Spec::new("Lazy…", ATTR_PROTECTED)` mirror rows (alphabetical;
    MECHANICAL, vocabulary-enforced).
  - `crates/rum-stdlib/src/tests/lazy.rs` — new tests (keep existing green).
  - A kernel-side `Part`-on-`LazyList` test — put it where kernel `Part`
    tests already live (locate; if none is a clean fit, a stdlib-level
    `rum-cli`-style probe test in `tests/lazy.rs` is acceptable).
- Files/areas you MUST NOT touch:
  - `crates/rum-core/src/expr.rs`; the evaluator dispatch loop; the
    matcher/formatter; any selector other than `eval_part`; any builtin
    other than the two new lazy builtins; the batch-2a realize/stepper
    driver shape (add variants, don't reshape).
  - `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**` (except appending to
    your report; reference regen is the director's job), `.git`,
    `Cargo.lock`.
- No unconditional default-path cost: the `eval_part` `LazyList` branch is
  one head check + arity/positive-int check, entered only for a `LazyList`
  first arg; the List/Association path is unchanged.

## Deliverable

- `eval_part` `LazyList` re-dispatch; `LazyNestList` source +
  `LazyFoldList` transformer (builtins + stepper arms); catalog + mirror
  registration (vocabulary green; reference regenerates at the gate).
- Tests: `lz[[i]]` forces the i-th element; non-integer / from-end / span
  / multi-level Part on a `LazyList` fall through unchanged;
  `LazyNestList` doubling stream; `LazyNestList` infinite `Normal`
  fail-closed; `LazyFoldList` running sums (init first); `LazyFoldList`
  over a finite inner realizes fully; nested `f` error propagation;
  snapshot round-trip of a `LazyNestList` and a `LazyFoldList` descriptor;
  opacity of both results.

## rum / Rust pitfalls (read twice)

1. Timing numbers come from `scripts/core-language-microbench.sh`; env
   OFF; A/B is base vs candidate per label.
2. A probe is the label's EXACT statement — no sink added.
3. Default-path cost: the `eval_part` branch is one head check; the
   existing Part path is unchanged. No `Mutex`/allocation on the existing
   path.
4. Evaluator flows are suspension-safe: the `Part` re-dispatch uses
   `self.evaluate`, and the NestList/FoldList arms call `session.evaluate`
   per pulled element — propagate every `EvalError` via `?`; do not
   swallow cancel/exception/Thrown.
5. Hot paths avoid owned-tree clones: build re-dispatch/descriptors with
   `Expr::call`; `Arc`-clone `f`/`init`/accumulator; do not deep-clone the
   descriptor per pulled element.
6. One variable per measurement; rerun the same label + same-tier
   neighbors.
7. Stop-loss is real: if the `Part` re-dispatch fights the evaluator, or a
   stepper arm fights the driver, twice for the same reason, STOP and
   report — do not keep tuning.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test --workspace
```

Because `eval_part` is touched, run the microbench A/B and paste the
`part-list` row + its same-tier neighbors:

```
bash scripts/core-language-microbench.sh --output-dir <base-dir>   # BASE
bash scripts/core-language-microbench.sh --output-dir <new-dir>    # change
```

Live `rum-cli --eval` probes, verbatim:

- `LazyRange[10, 5][[3]]` → `20`.
- `LazyRange[1, 1][[1]]` → `1`.
- `Normal[LazyTake[LazyNestList[Function[x, 2 x], 1], 5]]` →
  `List[1, 2, 4, 8, 16]`.
- `Normal[LazyFoldList[Plus, 0, LazyTake[LazyRange[1, 1], 4]]]` →
  `List[0, 1, 3, 6, 10]`.
- `MatchQ[LazyNestList[Function[x, x + 1], 0], _LazyList]` → `True`.
- A finite Part-out-of-range or a span `lz[[1 ;; 2]]` probe showing the
  spec falls through unchanged (inert / existing message), NOT a crash.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in
  the working tree; the director gates and commits.
- Do NOT modify files outside Scope; if you must, STOP and report.
- Touch nothing outside this repo; no network; only file edits within
  Scope, `cargo`, release `rum-cli`, the microbench scripts, `python3`.
- Testing Policy is a hard line: rum's own documented behavior only.
- No new `unsafe` (rum-core / rum-runtime are zero-unsafe). If convinced it
  is unavoidable, STOP and report.
- Minimal diff: no drive-by reformatting or renames outside the task.
- Paste real verbatim verification output; if a run did not reach green,
  say so — never fake it.

## Report format (final message)

1. What changed — per file, line ranges, one-line why (call out the Part
   re-dispatch and the two new stepper arms).
2. Verification — each command with verbatim output (test counts, the
   part-list A/B rows, the probes).
3. Verdict — confirm `lz[[i]]`, non-integer specs fall through,
   NestList/FoldList correctness + fail-closed, nested-error propagation,
   batch-1/2 tests still green.
4. Stop-loss / open questions (empty if none).
