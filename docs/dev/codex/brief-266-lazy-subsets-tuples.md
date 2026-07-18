# brief-266 — lazy sequences, batch 3b (LazySubsets + LazyTuples)

Implementation brief. Fifth brief of the (c) lazy-sequences line. Read
`docs/dev/lazy-sequences-design.md` and the batch-2a/3a code
(`crates/rum-stdlib/src/lazy.rs`) before touching anything.

> `LazyPermutations` is NOT in this brief — rum has no eager
> `Permutations` builtin (it stays symbolic), so a lazy one has no order
> reference; that is a separate director/user decision.

## Goal

Two lazy combinatorial ENUMERATION sources that match the order of rum's
existing eager `Subsets` / `Tuples` exactly, so `Normal[LazySubsets[l]]
=== Subsets[l]` and `Normal[LazyTuples[…]] === Tuples[…]`. Each is a new
source `PullStepper` variant holding a TRANSIENT enumeration cursor; the
descriptor holds only the finite input (serializable, snapshot
re-derives). Finite spaces exhaust naturally; huge spaces fail closed at
the iteration ceiling (each yielded element charges a source pull, the
batch-2a mechanism, unchanged). All prior lazy tests stay green. No
default-path change.

## Enumeration orders (must match the eager builtins)

- `Subsets[{a,b,c}]` → `{{}, {a}, {b}, {c}, {a,b}, {a,c}, {b,c}, {a,b,c}}`
  — by subset SIZE ascending (0..n), within a size by index-lexicographic
  (a "next k-combination" cursor). The lazy source enumerates size k =
  0,1,…,n; for each k, all C(n,k) combinations in lex order; then k+1.
- `Tuples[{a,b}, 2]` → `{{a,a},{a,b},{b,a},{b,b}}` and
  `Tuples[{{1,2},{3,4}}]` → `{{1,3},{1,4},{2,3},{2,4}}` — mixed-radix,
  LAST index varies fastest. The lazy source enumerates a mixed-radix
  counter over the inner-list lengths.
- Verify these against the eager builtin at build time (rum-vs-rum, our
  own documented behavior — NOT an external oracle): the acceptance test
  is `Normal[LazySubsets[l]] === Subsets[l]` etc. over several inputs
  including edges.

## LazySubsets

- Public builtin `LazySubsets[list]` (1 arg; `Protected`, `Pure`): `list`
  must be a `List` (else inert + `argform`). Returns
  `LazyList[LazySubsets[list]]`. (The `Subsets[list, spec]` size-spec form
  is OUT of scope — note it deferred.)
- Descriptor node `LazySubsets[list]`; source stepper holds the elements
  + a cursor `(k, combo: Vec<usize>)` starting at `k = 0` (the empty
  subset). `next`: `charge_source_pull()?`; materialize the current subset
  (`combo` indices → elements as a `List`); advance the cursor
  (next lex k-combination; when a size is exhausted, `k += 1` and reset to
  the first k-combination; when `k > n`, the stream is exhausted → after
  yielding the last, subsequent `next` returns `None`). Total 2^n.

## LazyTuples

- Public builtin, two forms, normalized to one descriptor:
  - `LazyTuples[list, n]` (list a `List`, n a non-negative Integer) →
    n copies of `list`.
  - `LazyTuples[{l1, …, lk}]` (a `List` whose every element is a `List`)
    → use directly.
  - Wrong shape (non-list, negative n, a non-list inner element) → inert
    + `argform`; wrong arity → `argx`.
  - Descriptor node `LazyTuples[<listOfLists>]` (uniform).
- Source stepper holds the inner lists + a mixed-radix cursor (a
  `Vec<usize>` of digits, one per inner list). `next`:
  `charge_source_pull()?`; materialize the current tuple (digit i →
  `listOfLists[i][digit_i]`); increment the cursor (last digit fastest,
  carry); exhausted when the carry overflows the first digit. Edge:
  `n = 0` / an empty `listOfLists` → exactly ONE empty tuple `{}` then
  done (matches `Tuples[l, 0]` → `{{}}`); any inner list empty → product
  0 → empty stream `{}`.

## Scope

- Files you MAY modify / create:
  - `crates/rum-stdlib/src/lazy.rs` — `eval_lazy_subsets` +
    `eval_lazy_tuples` shims; two new `PullStepper` source variants
    (`Subsets`, `Tuples`) + `next` arms + `stepper_from_descriptor` cases.
  - `crates/rum-stdlib/src/catalog/lazy.rs` — catalog entries
    (`.protected()`, declared message tags).
  - `crates/rum-runtime/src/runtime/builtin_catalog/stdlib_attributes.rs`
    — two `Spec::new("Lazy…", ATTR_PROTECTED)` mirror rows (alphabetical;
    MECHANICAL, vocabulary-enforced).
  - `crates/rum-stdlib/src/tests/lazy.rs` — new tests (keep existing green).
- Files/areas you MUST NOT touch:
  - `crates/rum-core/src/expr.rs`; the evaluator/matcher/formatter; the
    eager `Subsets`/`Tuples` implementations; any builtin other than the
    two new lazy ones; the realize/stepper driver shape (add variants,
    don't reshape); `Part`/kernel selectors.
  - `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**` (except appending to
    your report; reference regen is the director's job), `.git`,
    `Cargo.lock`.
- No default-path cost — these are new cold builtins; nothing on the hot
  path changes, so NO microbench A/B is required (state this).

## Deliverable

- `LazySubsets` + `LazyTuples` (builtins + source stepper variants);
  catalog + mirror registration (vocabulary green; reference regenerates
  at the gate).
- Tests (the key ones assert order-identity against the eager builtin):
  - `Normal[LazySubsets[{a, b, c}]] === Subsets[{a, b, c}]`; also `{}` and
    a singleton; `Take[LazySubsets[Range[40]], 3]` is cheap (proves
    laziness — the 2^40 space is never materialized) and equals the first
    three of `Subsets`' order (`{}, {1}, {2}`).
  - `Normal[LazyTuples[{a, b}, 2]] === Tuples[{a, b}, 2]`;
    `Normal[LazyTuples[{{1, 2}, {3, 4}}]] === Tuples[{{1, 2}, {3, 4}}]`;
    `LazyTuples[l, 0]` → one empty tuple; an inner empty list → empty.
  - `Normal` on a huge space fails closed at the ceiling (bounded, no
    hang); results are opaque `_LazyList`; snapshot round-trip of each
    descriptor re-derives; invalid inputs fail closed with the declared
    message.

## rum / Rust pitfalls (read twice)

1. Order is the contract: test `Normal[Lazy*[input]] === *[input]` against
   the eager builtin (rum's own behavior — allowed; NOT an external
   engine). If your cursor's order disagrees with the eager builtin, the
   cursor is wrong — fix the cursor, do not change the eager builtin.
2. The cursor is TRANSIENT (in the stepper); the descriptor holds only the
   input. Re-realizing enumerates from the start; snapshot stores the
   input and re-derives — do NOT try to serialize cursor state.
3. Default-path cost: none — new cold builtins. Do not touch the hot path.
4. Evaluator flows are suspension-safe: realization goes through the
   existing stepper driver (propagates `EvalError` via `?`); these sources
   do not call `session.evaluate` per element (pure index/materialize
   work), but STILL charge `charge_source_pull()` per yielded element so
   huge spaces fail closed.
5. Hot paths avoid owned-tree clones: build tuples/subsets with
   `Expr::call(system List, …)`; `Arc`-clone elements; do not deep-clone
   the whole input per element.
6. Exact work only; reuse `expr_as_non_negative_usize` for `n`.
7. Stop-loss is real: if a cursor's order cannot be made to match the
   eager builtin after two attempts, STOP and report with the diff
   between your order and the eager order.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test --workspace
```

No hot-path change → no microbench A/B (state this).

Live `rum-cli --eval` probes, verbatim:

- `Normal[LazySubsets[{a, b, c}]]` (equals `Subsets[{a, b, c}]`).
- `Take[LazySubsets[Range[40]], 3]` → `List[List[], List[1], List[2]]`
  (cheap — 2^40 never materialized).
- `Normal[LazyTuples[{a, b}, 2]]` → `List[List[a, a], List[a, b], List[b,
  a], List[b, b]]`.
- `Normal[LazyTuples[{{1, 2}, {3, 4}}]]` → `List[List[1, 3], List[1, 4],
  List[2, 3], List[2, 4]]`.
- `Normal[LazyTuples[{a, b}, 0]]` → `List[List[]]`.
- `MatchQ[LazySubsets[{1, 2}], _LazyList]` → `True`.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in
  the working tree; the director gates and commits.
- Do NOT modify files outside Scope; if you must, STOP and report.
- Touch nothing outside this repo; no network; only file edits within
  Scope, `cargo`, release `rum-cli`, `python3`.
- Testing Policy is a hard line: compare against rum's OWN eager builtins
  only; no external-engine comparison / oracle / compatibility matrix.
- No new `unsafe` (rum-core / rum-runtime are zero-unsafe). If convinced it
  is unavoidable, STOP and report.
- Minimal diff: no drive-by reformatting or renames outside the task.
- Paste real verbatim verification output; if a run did not reach green,
  say so — never fake it.

## Report format (final message)

1. What changed — per file, line ranges, one-line why (call out the two
   cursor algorithms).
2. Verification — each command with verbatim output (test counts, probes,
   the order-identity assertions).
3. Verdict — confirm order matches eager `Subsets`/`Tuples`, laziness
   (Range[40] cheap), fail-closed huge space, opacity, snapshot
   round-trip; prior lazy tests still green.
4. Stop-loss / open questions (empty if none).
