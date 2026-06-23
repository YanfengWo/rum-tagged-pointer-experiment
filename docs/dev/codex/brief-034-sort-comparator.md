# brief-034: implement 2-arg `Sort[list, p]` — the ordering-function form

## Goal

rum's `Sort` only implements the 1-argument form. The 2-argument
`Sort[list, p]` (custom ordering function) is a STUB: `eval_sort`'s
`[expr, _ordering]` branch ignores the comparator (`_ordering`) and returns the
call UNCHANGED/inert (`crates/rum-runtime/src/runtime/structure_ops/ordering_sets.rs:20-26`).
So `Sort[{3,1,2}, Greater]` stays `Sort[{3,1,2}, Greater]` instead of `{3,2,1}`.
This is the current correctness-mainline blocker at `stdlib/Real/Seq.wl`:
`auto/RealArith.wl:743` `varKeysSorted[vs_Association] := Sort[Keys[vs],
Order[#1,#2] === 1 &]` stays inert, then a downstream `Map[g, <inert Sort>]`
distributes `g` over the inert `Sort`'s two arguments — producing
`g[Keys[vs]]` and `g[cmp]`, i.e. `summandTerm[vs[{var[e,real]}], ...]` and
`summandTerm[vs[cmp], ...]`, both `Missing["KeyAbsent", ...]` — and the real
linear-form term is built malformed, raising `SPEC: term type does not match
binder` (diagnosed by isolating `Sort[{3,1,2}, Order[#1,#2]===1 &]` -> inert in
rum vs `{1,2,3}` in WK). When done, `Sort[list, p]` evaluates the ordering
function and returns the sorted list. (Unlike Numerator/Max/Min, this is a
present-but-incomplete builtin, not a missing one — a symbol-axis scan does not
catch it.)

## Context pointers

- Read `AGENTS.md` and PROGRESS `2026-06-23`.
- **The bug:** `eval_sort` (`structure_ops/ordering_sets.rs:13`). The 2-arg arm
  (`:20-26`) returns the inert call. Replace it with a real evaluation of the
  ordering function.
- **The template to MIRROR:** `eval_sort_by` + `resume_sort_by_from_index`
  (`ordering_sets.rs:31` and `:159-218`). That is rum's suspension-safe pattern
  for applying a user function during a sort: walk items by index, evaluate
  `function[item]` via `self.evaluate(...)` + `self.suspend_with_frame(...)`,
  accumulate into a `Vec<SortByEntry>`, and when the index runs out do the PURE
  `sorted.sort_by(|a,b| compare(...).then(a.index.cmp(&b.index)))` (stable) and
  rebuild the call. You will mirror this shape but evaluate the comparator over
  PAIRS, not a key over each element.
- `compare_structural_order` / `compare_expr_canonical`
  (`expr_runtime_helpers.rs:52`) is the 1-arg canonical comparator (not needed
  for the comparator path, but shows the pure-sort tail).
- rum's `Order[a,b]` builtin already works (`Order[3/2,2]` -> `1`), so once the
  comparator is actually evaluated, the `Order[#1,#2]===1 &` case yields
  canonical order correctly.

## WolframKernel semantics to match

`Sort[list, p]`: stable-sort `list` using ordering function `p`; element `a`
sorts before `b` when `p[a, b]` evaluates to `True`. Verified:
```
Sort[{3,1,2}, Greater]              == {3, 2, 1}
Sort[{3,1,2}, #1 > #2 &]            == {3, 2, 1}
Sort[{3,1,2}, Order[#1,#2] === 1 &] == {1, 2, 3}   (canonical asc)
Sort[{c,a,b}, Order[#1,#2] === 1 &] == {a, b, c}
```
- The list head is preserved (mma-hol always passes a `List`, but keep the
  `expr_head` like SortBy does).
- Only the 2-arg `Sort[list, p]` form is in scope (there is no `Sort[p]`
  operator form — leave the existing 1-arg `Sort[list]` branch untouched).

## Required approach (suspension-safe, general)

Do NOT special-case `Order[#1,#2]===1 &`; implement the general ordering
function. A comparator sort needs pairwise comparisons, so:

1. If the list expr is an `Atom`, keep the existing
   `invalid_ordering_normal_call("Sort", args, 1)`. Otherwise extract the head +
   element vector (like SortBy reads `expr_head`/`expr_args`).
2. **Resumable phase (mirror `resume_sort_by_from_index`):** evaluate
   `p[items[i], items[j]]` for the ordered pairs you need, ONE per
   `self.evaluate(...)` + `self.suspend_with_frame(...)` step, resuming into a
   continuation that stores the boolean (`result === True` -> `a` before `b`)
   and advances. Precomputing all `i != j` pairs (`n*(n-1)`, O(n²)) into a
   relation table is the simplest correct fit for the linear resumable pattern
   — mma-hol's sorted lists are tiny, so O(n²) is fine (note the cost; a smarter
   merge-sort-with-suspension is a later optimization, NOT required here).
3. **Pure phase:** stable-sort the element indices using the relation table —
   `a` before `b` iff `before[a][b]` (tie-break by original index for
   stability). Build a robust `Ordering`:
   `if before[i][j] { Less } else if before[j][i] { Greater } else { Equal }`,
   so an inconsistent user comparator cannot panic. Rebuild `Expr::call(expr_head,
   reordered_items)`.
4. A non-`True` comparator result (`False`, or anything not `True`) means "not
   before" — do not treat a non-boolean as an error; match WK's "before iff
   `True`".

## Scope

- Files you MAY modify: `crates/rum-runtime/src/runtime/structure_ops/ordering_sets.rs`
  (the `eval_sort` 2-arg arm + new `resume_*` helpers + any small accumulator
  struct like `SortByEntry`), plus a test module under
  `crates/rum-runtime/src/runtime/tests/`.
- MUST NOT touch: any `mma-hol` clone or the v5 checkpoints; `crates/rum-core`
  `Expr`/`Symbol`/serde shape; the 1-arg `Sort` path or `SortBy`; `AGENTS.md`,
  `PROGRESS.md`, `TODO.md`, `docs/**` except this report; `.git`, git config,
  `Cargo.lock`.
- No default-path cost: the comparator path only runs for 2-arg `Sort`.

## rum / Rust pitfalls (read twice)

1. **Suspension-safety is mandatory.** Apply the comparator via
   `self.evaluate(...)` + `self.suspend_with_frame(move |session, result| ...)`
   exactly like `resume_sort_by_from_index` — do NOT collapse it into a single
   eager re-entrant evaluation, and preserve the suspend/resume continuation
   shape. Every captured value the continuation needs must be cloned into the
   closure (see how the template clones `suspended_*`).
2. **Stable.** Equal-by-`p` elements keep input order (tie-break by index), like
   the template's `.then(lhs.index.cmp(&rhs.index))`.
3. **`True` is the System symbol.** Compare the evaluated result against
   `True` the System boolean (mirror however rum tests booleans elsewhere — e.g.
   Select's predicate check), not a string.
4. **Preserve head + arity guards.** Keep the `Atom` -> invalid and the
   `_ => invalid_call_with_arg_range("Sort", args, 1, 2)` arms.
5. **Minimal diff.** Mirror the SortBy machinery; do not refactor the 1-arg path.

## Verification (MANDATORY — run, iterate to green, paste verbatim)

```
cargo fmt --check
cargo test -p rum-runtime
cargo build --release -p rum-cli
BIN=./target/release/rum-cli

$BIN --eval 'Sort[{3, 1, 2}, Greater]'                 # {3, 2, 1}
$BIN --eval 'Sort[{3, 1, 2}, #1 > #2 &]'               # {3, 2, 1}
$BIN --eval 'Sort[{3, 1, 2}, Order[#1, #2] === 1 &]'   # {1, 2, 3}
$BIN --eval 'Sort[{c, a, b}, Order[#1, #2] === 1 &]'   # {a, b, c}
$BIN --eval 'Sort[{3, 1, 2}]'                          # {1, 2, 3}  (1-arg regression)
$BIN --eval 'Sort[Keys[<|c -> 1, a -> 2, b -> 3|>], Order[#1, #2] === 1 &]'  # {a, b, c}
$BIN --eval 'Sort[{1, 3, 2, 3, 1}, Less]'              # {1, 1, 2, 3, 3}
```

Paste verbatim. (The real unblock — re-extend RealArith + extend Seq on the v5
cp34 base, expecting the `SPEC: term type does not match binder` /
`Missing[KeyAbsent]` error to be GONE — is Claude's follow-up gate; you do NOT
run it.)

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; Claude gates and commits.
- NEVER write to/edit any `mma-hol` clone or the v5 checkpoints.
- Do NOT modify files outside Scope. If you think you must, STOP and say so.
- Full machine access, touch NOTHING outside this repo, no network. Commands:
  file edits within Scope, `cargo`, the release `rum-cli`.
- Suspension-safety per pitfall 1 is non-negotiable. Minimal diff. You MUST paste
  real verbatim output; if not green, say so.

## Report format (final message)

1. What changed — the `eval_sort` 2-arg arm + new resume helpers, files + line
   ranges, and a one-line note on the pair-evaluation/suspension structure.
2. Verification — each command, verbatim output (test counts + each eval).
3. Verdict — 2-arg `Sort[list, p]` evaluates the comparator, is stable, the
   canonical-order and Greater/Less cases match WK, 1-arg regression green.
4. Stop-loss / open questions.
