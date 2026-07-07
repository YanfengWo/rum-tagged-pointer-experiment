# Brief 177 — Unit-3 fixes 1: predicate & key-sort comparison controls

Implementation brief. Unit-3 fix batch 1 of the Usability Pass
(tracker `docs/dev/usability-pass.md`, unit-3 triage item 1; evidence
E3/E5/E7 in
`docs/dev/usability-pass/predicates-associations-findings.md`). All
opt-in, pure per-builtin argument parsing — NO call-planning /
operator-form work (that is brief-179). Every currently-accepted call
stays byte-identical; currently-rejected documented forms gain meaning.

## Precedent

`SameTest` already landed for the set operations `Union`/
`Intersection`/`Complement` in brief-172 (see
`crates/rum-stdlib/src/structure_ops/ordering_sets.rs` and the
`options/defaults.rs` registration). REUSE that mechanism and its
equivalence-helper shape here — do not fork a second SameTest
implementation.

## Changes

1. **`SameTest -> SameQ` on the collection predicates**: `DuplicateFreeQ`
   (adds a 2nd arg / option to a 1-arg builtin), `IntersectingQ`,
   `DisjointQ`, `SubsetQ`, `ContainsAll`, `ContainsAny`,
   `ContainsExactly`, `ContainsNone`, `ContainsOnly` (each adds a
   trailing `SameTest -> f` option to its 2-arg data form). Default is
   structural equality (today's behavior), byte-identical when the
   option is absent. Register `SameTest -> SameQ` (or `-> Automatic`,
   matching whatever brief-172 registered — check and be consistent)
   in `options/defaults.rs` for all nine.
   - `DuplicateFreeQ` was exactly-1-arg (E3); the option makes it
     accept `DuplicateFreeQ[list, SameTest -> f]`. The bare
     `DuplicateFreeQ[list]` stays byte-identical.
2. **`OrderedQ[list, p]`** — custom ordering predicate (E5). Today the
   2-arg form is rejected by exact-arity checking. Mirror however
   `Sort[list, p]` / `OrderedQ`'s single-arg canonical check compose
   `p` (check `Sort`'s existing 2-arg contract and reuse its
   comparison path). `OrderedQ[list]` byte-identical. Do NOT add the
   operator form here (that is 179).
3. **`LexicographicOrder[a, b, p]`** — comparator form (E5). Today the
   3-arg form is inert with no diagnostic. Implement the comparator, or
   if the comparator contract is ambiguous from documentation, fail
   closed with a targeted `LexicographicOrder` unsupported-form message
   instead of silent inertness — state which you did.
4. **`KeySortBy[assoc, f, p]`** — 3-arg ordering (E7). Today rejected
   with `argt` (1 or 2 args). Mirror `KeySort[assoc, p]`'s comparator
   path (which already accepts a predicate) applied to the transformed
   keys `f /@ keys`. `KeySortBy[assoc, f]` byte-identical.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
printf '' | cargo run -q -p rum-cli -- --eval '{DuplicateFreeQ[{1, -1}, SameTest -> (SameQ[Abs[#1], Abs[#2]] &)], IntersectingQ[{1}, {-1}, SameTest -> (SameQ[Abs[#1], Abs[#2]] &)], SubsetQ[{1, 2}, {-1}, SameTest -> (SameQ[Abs[#1], Abs[#2]] &)], ContainsAll[{1, 2}, {-1}, SameTest -> (SameQ[Abs[#1], Abs[#2]] &)], OrderedQ[{3, 2, 1}, Greater], LexicographicOrder[{a}, {b}, Greater], KeySortBy[<|"bb" -> 1, "a" -> 2|>, StringLength, Greater], Options[DuplicateFreeQ]}'
# defaults byte-identical:
printf '' | cargo run -q -p rum-cli -- --eval '{DuplicateFreeQ[{1, 1}], IntersectingQ[{1}, {2}], SubsetQ[{1, 2}, {1}], ContainsAll[{1, 2}, {1}], OrderedQ[{1, 2, 3}], KeySortBy[<|"bb" -> 1, "a" -> 2|>, StringLength]}'
git status --short
```

Expected: fmt clean; workspace green (report totals); probe 1 all
evaluate with no messages — the SameTest cases treat 1/-1 as equal
(`DuplicateFreeQ` → `False`, `IntersectingQ` → `True`, `SubsetQ` →
`True`, `ContainsAll` → `True`), `OrderedQ[{3,2,1}, Greater]` → `True`,
`LexicographicOrder` and `KeySortBy` per your contract (paste),
`Options[DuplicateFreeQ]` shows the registered SameTest; probe 2
byte-identical to today (paste).

## Scope

- MAY modify: `crates/rum-stdlib/src/predicates/**` (collections,
  ordering), `crates/rum-stdlib/src/associations/**` (KeySortBy),
  `crates/rum-stdlib/src/catalog/predicates.rs` and
  `crates/rum-stdlib/src/catalog/associations.rs` (descriptions),
  `crates/rum-stdlib/src/docgen.rs` (LIMITS),
  `crates/rum-runtime/src/runtime/options/defaults.rs` (SameTest
  registrations), `crates/rum-stdlib/src/tests/**`,
  `crates/rum-stdlib/src/tests.rs`, `docs/reference/builtins.md`
  (regenerated if descriptions changed).
- MUST NOT touch: the operator-form matcher in `functional_data.rs`,
  `Sort`/`KeySort` existing behavior, other families, the guide,
  `TODO.md`, the tracker, findings files, `Cargo.toml` files.

## Hard rules

- Do NOT `git commit`, branch, push. No network, no new dependencies,
  no new `unsafe`.
- Every currently-accepted call byte-identical; changes only give
  meaning to currently-rejected documented forms plus options
  defaulting to today's behavior. No operator forms (179 owns those).
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format

1. What changed — per file, per item; the LexicographicOrder decision
   (implemented vs message); the SameTest default value used (match
   brief-172).
2. Verification — all commands verbatim.
3. Stop-loss / open questions (empty if none).
