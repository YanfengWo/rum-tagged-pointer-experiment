# brief-262 — lazy pure sequences, batch 1 (mechanism spine)

Implementation brief. First brief of the (c) lazy-sequences line. Read
`docs/dev/lazy-sequences-design.md` in full before touching anything.

## Goal

When done: rum has an opaque, pure, immutable lazy-sequence value
`LazyList[<descriptor>]` (a reserved inert head), a single generator
`LazyRange` producing infinite arithmetic streams, a Rust pull
interpreter that realizes a descriptor under the session iteration
ceiling + cancellation, and four realizers taught to force a `LazyList`:
`Normal`, `Take`, `First`, `Part`. `LazyList` is opaque (does not match
`_List`, does not auto-force), all-`Pure`, and round-trips through
session snapshot/restore re-deriving identically. This is the extensible
spine; transformers and the broad set come in later batches. NO change
to the default evaluation path or to list operations other than the four
named realizers.

## Context pointers

- Read `AGENTS.md` (operational notes + Testing Policy hard lines) and
  `docs/dev/lazy-sequences-design.md` (the canonical design; §2
  representation, §3 surface, §4 opaque semantics, §5 safety) first.
- Registration model for a NEW ordinary stdlib builtin (NOT control
  family): a family module under `crates/rum-stdlib/src/` wired in
  `lib.rs`, a `catalog/<family>.rs` entry wired in `catalog/mod.rs`
  (each family is one `mod X;` line), plus the attribute mirror in
  `crates/rum-runtime/src/runtime/builtin_catalog/stdlib_attributes.rs`
  (alphabetical `Spec::new(...)` rows; the vocabulary test
  `catalog_attributes_match_kernel_vocabulary` in `rum-stdlib/src/tests.rs`
  enforces catalog↔mirror agreement). Precedent to copy: the
  `ReplaceAllUnheld` builtin — `rum-stdlib/src/rewrite_builtins.rs` +
  `catalog/rewrite.rs`.
- A purely inert head (like `Exception`/`UpTo`) needs ONLY a catalog
  `BuiltinSpec::new(name, summary).protected()` entry + an attribute
  mirror row — no shim, no eval function. `LazyList` is such a head.
- Realizer seams to extend (locate the exact function; add a `LazyList`
  branch at the top that does NOT perturb the existing List path):
  - `Normal` → `crates/rum-stdlib/src/associations/basic.rs` (`eval_normal`, ~373).
  - `Take` → `crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs` (`eval_take`, ~17).
  - `First` → `crates/rum-stdlib/src/structure_ops/surface.rs` (`eval_first`, ~85).
  - `Part` → locate the `Part` implementation (structure_ops; the `[[i]]`
    selector). If `Part` on an opaque head is intrusive to reach cleanly,
    STOP and report the seam rather than forcing it.
- The iteration ceiling + cancellation: find how `Do`/`Table`/`NestWhile`
  bound their loops and check cancellation (the per-session iteration
  budget + the cancel/interrupt check). The realize loop MUST use the
  same mechanism. Do not invent a new limit.
- Acceptance is mechanical: the named tests + probes decide, not prose.

## Representation (implement exactly this)

- Value: `LazyList[<descriptor>]`. Head `LazyList`, attributes
  `{HoldAllComplete, Protected}` — content never self-evaluates, so the
  value is inert (a normal form; `LazyList` has no downvalues). One
  argument: the descriptor.
- Descriptor for batch 1 (the ONLY node this batch): `LazyRange[start, step]`
  — head `LazyRange`, args are already-evaluated Integer/Rational. This
  is the internal descriptor form; it appears ONLY inside a `LazyList`
  wrapper (shielded by `HoldAllComplete`, never re-evaluated).
- The public builtin `LazyRange` (SAME head, disambiguated purely by
  position — top-level call = builtin, inside `LazyList` = descriptor)
  evaluates its args and returns the wrapped value:
  - `LazyRange[]` → `LazyList[LazyRange[1, 1]]`
  - `LazyRange[start]` → `LazyList[LazyRange[start, 1]]`
  - `LazyRange[start, step]` → `LazyList[LazyRange[start, step]]`
  - All three are INFINITE (stream-first-element oriented; `start` is the
    first element — intentionally NOT `Range`'s max-bounded semantics).
    `start`/`step` must be Integer or Rational; anything else → the
    builtin's argument message, inert return (fail-closed, no partial
    value). `LazyRange` attributes: `{Protected}` (args evaluate; NOT
    hold — the held-body generator `LazyTable` is a later batch).
  - Bounded/finite `LazyRange` arg shapes and full `Range`-shape parity
    are DEFERRED to the close-out batch; do not add them here.
- `LazyRange` head is `Capability::Pure`.

## Pull interpreter (the extensible core)

- One Rust function that, given a descriptor, yields elements on demand,
  driven by a caller-supplied budget (max elements to pull, or "until
  exhausted"), checking the session cancellation flag each step and
  charging the session iteration ceiling. Batch 1 has ONE arm:
  `LazyRange[start, step]` yields `start, start+step, …` forever.
  Structure it as a `match` on the descriptor head so batch 2 adds arms
  (`LazyMap`, `LazySelect`, …) without reshaping the driver.
- Realizers call this interpreter:
  - `Normal[LazyList[d]]` → pull until exhausted, collect into a `List`.
    An infinite descriptor hits the iteration ceiling → return the same
    `TerminatedEvaluation[...]` shape `Do`/`Table` produce on ceiling
    (fail-closed — MUST NOT hang).
  - `Take[LazyList[d], n]` (n a non-negative Integer) → pull n, return a
    `List`. `Take[LazyList[d], UpTo[n]]` → pull at most n (UpTo landed in
    brief-254; reuse its parse). n greater than a finite stream's length
    is not reachable in batch 1 (all sources infinite); keep the general
    "pull up to n, stop early if exhausted" shape anyway.
  - `First[LazyList[d]]` → pull 1, return the element; empty → `First`'s
    existing empty message.
  - `Part[LazyList[d], i]` (i a positive Integer, 1-based) → pull i,
    return the i-th; out of range on a finite stream → `Part`'s existing
    range message. Negative/other Part specs on a `LazyList` → fail
    closed with the existing Part message (do NOT try to support
    from-the-end indexing on a lazy stream).
- Every realizer branch is entered ONLY when the argument's head is
  `LazyList`; the existing List/Association code path is byte-unchanged.

## Opaque semantics (mostly free — pin with tests)

- `LazyList[...]` head is `LazyList`, so `MatchQ[lz, _List]` is already
  `False` and `MatchQ[lz, _LazyList]` already `True` — no matcher change;
  add tests pinning both, plus that a plain list operation NOT in the
  realizer set (e.g. `Length`, `Map`, `Total`) does NOT force it (stays
  symbolic / inert on the `LazyList` head).
- Formatting: default rendering `LazyList[LazyRange[1, 1]]` is acceptable
  for this batch (honest, shows the descriptor, never enumerates
  elements). Do NOT add a custom formatter here. Add a test asserting the
  rendered form is the `LazyList[...]` descriptor string and does not
  enumerate stream elements.
- Structural equality is automatic (two `LazyList`s equal iff descriptors
  equal); add one test.

## Scope

- Files you MAY modify / create:
  - NEW `crates/rum-stdlib/src/lazy.rs` — the `LazyRange` builtin shim +
    the pull interpreter + realize helpers the realizers call
    (`pub(crate)` so `structure_ops`/`associations` can call them).
  - `crates/rum-stdlib/src/lib.rs` — `mod lazy;` wiring (MECHANICAL).
  - NEW `crates/rum-stdlib/src/catalog/lazy.rs` — catalog entries for
    `LazyList` (inert, `.protected()`) and `LazyRange`.
  - `crates/rum-stdlib/src/catalog/mod.rs` — `mod lazy;` + include its
    specs in the aggregation exactly as sibling families do (MECHANICAL).
  - `crates/rum-runtime/src/runtime/builtin_catalog/stdlib_attributes.rs`
    — mirror rows: `LazyList` = `{HoldAllComplete, Protected}`,
    `LazyRange` = `{Protected}` (alphabetical position; MECHANICAL —
    the vocabulary test enforces it).
  - `crates/rum-stdlib/src/associations/basic.rs` (`eval_normal`),
    `crates/rum-stdlib/src/structure_ops/surface.rs` (`eval_first`),
    `crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs`
    (`eval_take`), and the `Part` implementation — each gains ONE
    `LazyList` branch.
  - Tests: a new `crates/rum-stdlib/src/tests/lazy.rs` (or the closest
    existing tests module pattern) + wiring; and ONE snapshot round-trip
    test in the runtime tests where session snapshot/restore is already
    tested (locate the existing snapshot round-trip test module and add a
    `LazyList` case).
- Files/areas you MUST NOT touch:
  - `crates/rum-core/src/expr.rs` `Expr`/`Symbol` representation or any
    serde/public shape — the value is a plain `Call`, no core change.
  - The evaluator dispatch / eval hot path, the matcher, the formatter
    internals, any builtin other than the four named realizers +
    `LazyRange`.
  - `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**` (except appending to
    your own report), `.git`, `Cargo.lock`.
- No unconditional default-path cost. The realizer `LazyList` branch must
  be a single cheap head check that returns early only when the head is
  `LazyList`; the existing path pays at most one head comparison.

## Deliverable

- `LazyList` opaque value + `LazyRange` (three infinite forms) + the pull
  interpreter (one arm, extensible) + `Normal`/`Take`(+`UpTo`)/`First`/
  `Part` realizer branches.
- Catalog + attribute-mirror registration (reference regenerates cleanly;
  vocabulary test green).
- Tests: opaque semantics (matches `_LazyList` not `_List`; non-realizer
  ops do not force), realizer correctness (`Take`/`First`/`Part` on an
  infinite `LazyRange`), `Normal`-on-infinite fail-closed
  (`TerminatedEvaluation`, bounded, does not hang), rendering, structural
  equality, snapshot round-trip re-derive.
- If any structure-op microbench label (Take/First/Part/Normal neighbors)
  exists, the A/B rows before/after (these realizers are touched).

## rum / Rust pitfalls (read twice)

1. Timing numbers come from `scripts/core-language-microbench.sh` only;
   profiling env OFF; an A/B is base-build vs candidate-build per label.
2. A probe is the label's EXACT statement — no sink `Set`/`Print` added.
3. Default-path cost is unconditional cost: the `LazyList` branch is a
   single head check; the List path must be unchanged. No `Mutex`,
   allocation, or per-call heap touch added to the existing realizer
   path.
4. Evaluator flows are suspension-safe: if the realize loop reuses a
   suspend/resume-bearing iteration primitive, preserve its continuation
   shape — do not collapse a `suspend_with_frame*` into an eager call.
5. Hot paths avoid owned-tree clones: `Expr`/`SharedExpr`/`SharedArgs`
   are `Arc`-shared; build the `LazyList`/`LazyRange` values with
   `Expr::call` and borrow-traverse descriptors; no deep clone per pulled
   element.
6. One variable per measurement; rerun the same label + same-tier
   neighbors.
7. Stop-loss is real: if `Part`-on-opaque or the ceiling reuse fights you
   twice for the same reason, STOP and report the seam — do not keep
   forcing it.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test --workspace
```

Then, because the four realizers (`eval_normal`/`eval_take`/`eval_first`/
`Part`) are touched, run the microbench A/B and paste the structure-op /
realizer label rows + their same-tier neighbors:

```
bash scripts/core-language-microbench.sh --output-dir <base-dir>   # on BASE (pre-change)
bash scripts/core-language-microbench.sh --output-dir <new-dir>    # on the change
```

Also paste, as live `rum-cli` probes, the verbatim output of:

- `MatchQ[LazyRange[1, 1], _List]` → `False`;
  `MatchQ[LazyRange[1, 1], _LazyList]` → `True`.
- `Take[LazyRange[10, 5], 3]` → `{10, 15, 20}`.
- `First[LazyRange[7, 2]]` → `7`; `LazyRange[1, 1][[4]]` → `4`.
- `Normal[LazyRange[1, 1]]` → a `TerminatedEvaluation[...]` (fail-closed,
  returns promptly).
- The render of `LazyRange[1, 1]` (shows `LazyList[LazyRange[1, 1]]`, no
  elements).

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes
  in the working tree; the director gates and commits.
- Do NOT modify files outside Scope; if you believe you must, STOP and
  write it in the report.
- Touch nothing outside this repo; no network; only file edits within
  Scope, `cargo`, release `rum-cli`, the microbench scripts, and
  `python3` for parsing.
- Testing Policy is a hard line: rum's own documented behavior only; no
  external-engine comparison, oracle, or compatibility matrix.
- No new `unsafe` (rum-core / rum-runtime are zero-unsafe). If convinced
  it is unavoidable, STOP and report.
- Minimal diff: no drive-by reformatting or renames outside the task.
- Paste real verbatim verification output; if a run did not reach green,
  say so — never fake it.

## Report format (final message)

1. What changed — per file, line ranges, one-line why.
2. Verification — each command with verbatim output (test counts,
   per-label A/B rows, the live probes above).
3. Verdict — realizer before/after rows + any neighbor checked; confirm
   opaque semantics and the fail-closed infinite `Normal`.
4. Stop-loss / open questions (empty if none) — in particular the `Part`
   seam if it was awkward.
