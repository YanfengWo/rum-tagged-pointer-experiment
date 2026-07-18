# brief-267 — Permutations (eager) + LazyPermutations (batch 3c)

Implementation brief. Sixth brief of the (c) lazy-sequences line. Read
`docs/dev/lazy-sequences-design.md`, the batch-3b code
(`crates/rum-stdlib/src/lazy.rs`), and the eager combinatorics style
(`crates/rum-stdlib/src/structure_ops/combinatorial_lists.rs`,
`eval_subsets`) before touching anything.

## Goal

> **REVISION 1 (director, 2026-07-18):** Codex correctly stop-lossed on a
> scope omission — `catalog/structure.rs` registers via
> `structure_ops::eval_X`, so a new `eval_permutations` in
> `combinatorial_lists.rs` must be re-exported from
> `structure_ops/mod.rs` (line ~30, the `pub(crate) use
> combinatorial_lists::{…}` list). That file (and the same re-export for
> the shared `next_permutation` helper if `lazy.rs` reaches it that way)
> is now IN SCOPE for MECHANICAL re-export additions only. Nothing else in
> `mod.rs` changes.

Add the pair (user ruling 2026-07-18 — rum has no eager `Permutations`
today): an eager `Permutations[list]` and a lazy `LazyPermutations[list]`,
sharing ONE order so `Normal[LazyPermutations[l]] === Permutations[l]`.
Both are `Pure`, `Protected`. The lazy one is a source `PullStepper` with
a next-permutation cursor; the eager one enforces the generated-element +
memory limits like `Subsets`. No default/hot-path change.

## Order semantics (fix exactly this — folk scope)

`Permutations[list]` = all **DISTINCT** permutations of the multiset, in
**rum canonical lexicographic order**: sort the elements with
`compare_expr_canonical` (the comparator `Sort` uses — exported from
`kernel_api`), then enumerate distinct permutations via the standard
next-permutation algorithm (find the largest `i` with `a[i] < a[i+1]`,
the largest `j > i` with `a[j] > a[i]`, swap, reverse the suffix — all
comparisons via `compare_expr_canonical`). This yields distinct
permutations (duplicates handled) and terminates at the descending
arrangement.

- `Permutations[{a, b, c}]` → `{{a,b,c},{a,c,b},{b,a,c},{b,c,a},{c,a,b},
  {c,b,a}}` (6, lex).
- `Permutations[{1, 1, 2}]` → `{{1,1,2},{1,2,1},{2,1,1}}` (3, distinct).
- `Permutations[{}]` → `{{}}`; `Permutations[{a}]` → `{{a}}`.
- This is FOLK order (canonical lex; the first perm is the sorted
  arrangement). It intentionally does NOT reproduce WL's input-first
  ordering — document that in the one-line summary. The `Permutations[l,
  {n}]` fixed-length form is OUT of scope (note it deferred).

## Eager `Permutations[list]`

- Add `eval_permutations` in `combinatorial_lists.rs`, mirroring
  `eval_subsets`'s shape (1 arg; the arg must be a `Call` list, else the
  same style of `invalid_*` fail-closed).
- Compute the distinct-permutation count exactly as a `BigInt`
  (`n! / ∏ (multiplicity_k!)`) and enforce the SAME guards `eval_subsets`
  uses: `enforce_generated_element_limit_for_big_count` +
  `enforce_active_memory_limit_for_projected_bytes` — so a huge `n!` fails
  closed BEFORE materializing (do not enumerate first).
- Sort a working copy with `compare_expr_canonical`, enumerate via
  next-permutation, collect into a `List` (preserve the original element
  Exprs, e.g. keep `a`,`b`,`c` — only the ORDER is canonicalized).

## `LazyPermutations[list]`

- Public builtin `LazyPermutations[list]` (1 arg; the arg a `List`, else
  inert + `argform`; wrong arity → `argx`): returns
  `LazyList[LazyPermutations[list]]` (descriptor holds the ORIGINAL list).
- Descriptor node `LazyPermutations[list]`; source stepper (charges the
  ceiling per element like the other sources). `stepper_from_descriptor`
  extracts the list items, **sorts a copy with `compare_expr_canonical`**
  (same order as eager), and holds `current: Vec<Expr>` (starts sorted) +
  `exhausted: bool`. `next`: if `exhausted` → `None`; else
  `charge_source_pull()?`, materialize the current permutation as a
  `List`, then `exhausted = !next_permutation(&mut current)` (the same
  algorithm/comparator as eager). Finite (distinct-perm count); huge `n!`
  fails closed at the ceiling; bounded `Take`/`First`/`lz[[i]]` cheap.
- Factor the next-permutation step so eager and lazy call the SAME helper
  (guarantees identical order).

## Scope

- Files you MAY modify / create:
  - `crates/rum-stdlib/src/structure_ops/combinatorial_lists.rs` —
    `eval_permutations` + the shared `next_permutation` helper (or put the
    helper where both eager and `lazy.rs` can reach it as `pub(crate)`).
  - `crates/rum-stdlib/src/structure_ops/mod.rs` — MECHANICAL: add
    `eval_permutations` (and the shared `next_permutation` helper if
    `lazy.rs` reaches it here) to the `pub(crate) use
    combinatorial_lists::{…}` re-export at ~line 30. No other change to
    this file.
  - `crates/rum-stdlib/src/catalog/structure.rs` — `Permutations` catalog
    entry (`.protected()`; declared message tags).
  - `crates/rum-stdlib/src/lazy.rs` — `eval_lazy_permutations` shim + a
    `Permutations` source `PullStepper` variant + `next` arm +
    `stepper_from_descriptor` case (calling the shared next-permutation +
    `compare_expr_canonical`).
  - `crates/rum-stdlib/src/catalog/lazy.rs` — `LazyPermutations` entry.
  - `crates/rum-runtime/src/runtime/builtin_catalog/stdlib_attributes.rs`
    — mirror rows `Permutations` and `LazyPermutations`, both
    `ATTR_PROTECTED` (alphabetical; MECHANICAL, vocabulary-enforced).
  - Tests: `crates/rum-stdlib/src/tests/lazy.rs` (LazyPermutations + the
    `Normal === eager` identity) and the eager combinatorics test module
    (locate where `Subsets`/`Tuples` are tested; add `Permutations`
    cases). Keep all existing tests green.
- Files/areas you MUST NOT touch:
  - `crates/rum-core/src/expr.rs`; the evaluator/matcher/formatter; the
    eager `Subsets`/`Tuples` impls; the realize/stepper driver shape (add a
    variant, don't reshape); `Part`/kernel selectors.
  - `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**` (except appending to
    your report; reference regen is the director's job), `.git`,
    `Cargo.lock`.
- No default-path cost — new cold builtins only; NO microbench A/B
  required (state this).

## Deliverable

- Eager `Permutations` + `LazyPermutations` sharing one order; catalog +
  mirror registration (vocabulary green; reference regenerates at the
  gate).
- Tests: `Permutations[{a,b,c}]` exact (6, lex); `Permutations[{1,1,2}]`
  distinct (3); `Permutations[{}]` → `{{}}`;
  `Normal[LazyPermutations[l]] === Permutations[l]` over several inputs
  (incl. duplicates and empty); `Take[LazyPermutations[Range[12]], 3]`
  cheap (12! never materialized) = the first three of `Permutations`'
  order; huge eager `Permutations[Range[50]]` fails closed at the
  generated-element limit; huge lazy `Normal` fails closed at the ceiling;
  results opaque `_LazyList`; snapshot round-trip re-derives; invalid
  inputs fail closed.

## rum / Rust pitfalls (read twice)

1. Order is the contract: eager and lazy MUST call the same
   next-permutation helper + `compare_expr_canonical`, so
   `Normal[LazyPermutations[l]] === Permutations[l]` holds by
   construction — test it.
2. The lazy cursor is TRANSIENT (sorted `current` in the stepper); the
   descriptor holds only the original list. Snapshot stores the input and
   re-derives — do NOT serialize cursor state.
3. Eager must fail closed on huge `n!` via the count guards BEFORE
   enumerating (mirror `eval_subsets`); the lazy one charges
   `charge_source_pull()` per element so its full `Normal` fails closed at
   the ceiling.
4. Default-path cost: none — new cold builtins. Do not touch the hot path.
5. Suspension-safe: the lazy source does pure index/compare work but still
   charges the ceiling per element; propagate any `EvalError` via `?`.
6. Distinct perms: next-permutation over a canonically-sorted array yields
   DISTINCT multiset permutations and terminates — do not emit `n!`
   duplicates for a multiset.
7. Stop-loss is real: if `Normal[LazyPermutations] === Permutations` will
   not hold after two attempts, STOP and report the order diff.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test --workspace
```

No hot-path change → no microbench A/B (state this).

Live `rum-cli --eval` probes, verbatim:

- `Permutations[{a, b, c}]` → the 6-perm lex list.
- `Permutations[{1, 1, 2}]` → `List[List[1,1,2],List[1,2,1],List[2,1,1]]`.
- `Normal[LazyPermutations[{a, b, c}]] === Permutations[{a, b, c}]` → `True`.
- `Normal[LazyPermutations[{1, 1, 2}]] === Permutations[{1, 1, 2}]` → `True`.
- `Take[LazyPermutations[Range[12]], 3]` → cheap; the first three perms.
- `Permutations[{}]` → `List[List[]]`.
- A huge-input fail-closed probe (eager `Permutations[Range[50]]` and lazy
  `Normal[LazyPermutations[Range[50]]]`) each returning promptly, not
  hanging.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in
  the working tree; the director gates and commits.
- Do NOT modify files outside Scope; if you must, STOP and report.
- Touch nothing outside this repo; no network; only file edits within
  Scope, `cargo`, release `rum-cli`, `python3`.
- Testing Policy is a hard line: compare lazy against rum's OWN eager
  `Permutations` only; no external-engine comparison / oracle.
- No new `unsafe` (rum-core / rum-runtime are zero-unsafe). If convinced it
  is unavoidable, STOP and report.
- Minimal diff: no drive-by reformatting or renames outside the task.
- Paste real verbatim verification output; if a run did not reach green,
  say so — never fake it.

## Report format (final message)

1. What changed — per file, line ranges, one-line why (call out the
   shared next-permutation helper).
2. Verification — each command with verbatim output (test counts, probes,
   the `Normal === eager` identities).
3. Verdict — confirm distinct-perm order, eager/lazy identity, both
   fail-closed paths, opacity, snapshot round-trip; prior tests green.
4. Stop-loss / open questions (empty if none).
