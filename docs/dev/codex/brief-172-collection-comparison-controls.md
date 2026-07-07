# Brief 172 — Unit-2 fixes 3: collection comparison controls

Implementation brief. Unit-2 fix batch 3 of the Usability Pass
(tracker `docs/dev/usability-pass.md`, unit-2 triage item 3; evidence
E4 in `docs/dev/usability-pass/structure-functional-findings.md`).
All opt-in: every currently-accepted call stays byte-identical;
currently-rejected documented forms gain meaning. No call-planning
work (brief-175 already landed the operator forms — the two-argument
data forms added here must not collide with any operator arity; check
against the matcher in `functional_data.rs` before wiring each one).

## Changes

1. **`DeleteDuplicates[list, test]`** (WL-documented): custom equality
   predicate; first-kept-wins order as the default form. Folk note for
   the catalog description and docgen LIMITS: the predicate form is
   O(n²) pairwise (WL's is too) — document, don't optimize.
2. **`Tally[list, test]`** (WL-documented): custom equality; the
   representative element is the FIRST of each class.
3. **Set operations `SameTest`** (WL-documented option):
   `Union[lists…, SameTest -> f]`, `Intersection[…, SameTest -> f]`,
   `Complement[…, SameTest -> f]`. Today a trailing rule is consumed
   as a same-head input and emits `heads` (E4) — after the change a
   trailing `SameTest -> f` (or option list) parses as the option.
   Register the default (`SameTest -> Automatic`) in
   `options/defaults.rs`. Output ordering under a custom test:
   keep-first-occurrence in the sorted scan order you inherit from the
   default path; document the choice.
4. **`ReverseSort[list, p]`** (WL-documented): custom ordering
   predicate, mirroring however `Sort[list, p]` handles `p` today
   (check `Sort`'s existing two-argument contract FIRST and mirror it
   exactly, reversed).
5. **`SortBy[list, f, p]`** (WL-documented): three-argument ordering —
   sort by `f`-values under predicate `p` (ties broken as today's
   `SortBy` breaks them; document).
6. **`GatherBy[list, {f1, f2, …}]`** (WL-documented): hierarchical
   grouping — group by `f1`, then within each group by `f2`, etc.
   Today the function list is applied as ONE callable, silently
   producing singleton groups (E4) — a silent wrong answer; this
   replaces it with the documented meaning.
7. **`MaxBy[list, f, n]` / `MinBy[list, f, n]`**: top-/bottom-k by
   `f`-value, returned in the order WL documents (MaxBy: decreasing
   f-order among the selected; verify against your knowledge and
   document what you implement; ties keep list order). The 2-argument
   forms byte-identical.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
printf '' | cargo run -q -p rum-cli -- --eval '{DeleteDuplicates[{1, -1, 2}, SameQ[Abs[#1], Abs[#2]] &], Tally[{1, -1, 2}, SameQ[Abs[#1], Abs[#2]] &], Union[{1, -1}, SameTest -> (SameQ[Abs[#1], Abs[#2]] &)], Intersection[{1, 2}, {-1}, SameTest -> (SameQ[Abs[#1], Abs[#2]] &)], Complement[{1, 2}, {-1}, SameTest -> (SameQ[Abs[#1], Abs[#2]] &)], ReverseSort[{3, 1, 2}, Less], SortBy[{3, 1, 2}, Identity, Greater], GatherBy[{1, 2, 3, 4}, {OddQ, Mod[#, 3] &}], MaxBy[{1, -3, 2}, Abs, 2], MinBy[{1, -3, 2}, Abs, 2]}'
# defaults byte-identical:
printf '' | cargo run -q -p rum-cli -- --eval '{DeleteDuplicates[{1, 1, 2}], Tally[{a, a, b}], Union[{2, 1}, {1, 3}], ReverseSort[{3, 1, 2}], SortBy[{3, 1, 2}, Identity], GatherBy[{1, 2, 3}, OddQ], MaxBy[{1, -3}, Abs], Options[Union]}'
git status --short
```

Expected: fmt clean; workspace green (report totals); probe 1 all
evaluate with no messages (paste outputs — review evidence; GatherBy's
hierarchical case must show nested grouping, not singletons); probe 2
byte-identical to today except `Options[Union]` now shows the
registered `SameTest` default.

## Scope

- MAY modify: `crates/rum-stdlib/src/structure_ops/**`
  (dedup/collection_stats/ordering_sets), the catalog descriptions in
  `crates/rum-stdlib/src/catalog/structure.rs`,
  `crates/rum-stdlib/src/docgen.rs` (LIMITS),
  `crates/rum-runtime/src/runtime/options/defaults.rs` (SameTest
  registrations), `crates/rum-stdlib/src/tests/**`,
  `crates/rum-stdlib/src/tests.rs`, `docs/reference/builtins.md`
  (regenerated if descriptions changed).
- MUST NOT touch: `Sort`'s existing behavior, the operator-form
  matcher in `functional_data.rs`, other catalogs, the guide,
  `TODO.md`, the tracker, findings files.

## Hard rules

- Do NOT `git commit`, branch, push. No network, no new dependencies,
  no new `unsafe`.
- Every currently-accepted call byte-identical; changes only give
  meaning to currently-rejected forms (plus the GatherBy function-list
  case, whose current silent-singleton output is the documented bug
  being fixed — pin the new hierarchical contract with tests).
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format

1. What changed — per file, per item; the documented orderings chosen
   (set-ops under custom test, MaxBy/MinBy top-k, SortBy ties).
2. Verification — all commands verbatim.
3. Stop-loss / open questions (empty if none).
