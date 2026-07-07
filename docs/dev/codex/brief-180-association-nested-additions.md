# Brief 180 — Unit-3 fixes 4 (FINAL): nested/keyed association additions

Implementation brief. Unit-3 fix batch 4, the last one, closing the
predicates+associations unit. Four NEW association builtins whose
demand is evidenced by the WFR survey
(`docs/dev/wfr-core-survey.md`) and the audit (E10). ABSORPTION IS
IDEA-LEVEL: independent folk implementations from the pinned behavior
below; no third-party source read. Data forms (plus operator forms
that are unambiguous via the existing matcher — see each item). Neutral
module naming — do NOT name any file after an inspiration source
(brief-174 lesson).

## New builtins (add to the `associations` family)

1. **`KeyGroupBy[assoc, f]`** and **`KeyGroupBy[assoc, f, red]`** — the
   key-grouping analog of `GroupBy` (mirror `eval_group_by` in
   `crates/rum-stdlib/src/associations/grouping.rs` for shape and the
   reduction convention). Group the association's entries by `f[key]`;
   each group's value is a sub-association of the original
   `key -> value` entries whose key mapped there. The 3-argument form
   applies `red` to each group exactly as `GroupBy[…, red]` does
   (mirror it — do not invent a different reduction shape). Pin:
   `KeyGroupBy[<|"aa" -> 1, "b" -> 2|>, StringLength]` →
   `<|2 -> <|"aa" -> 1|>, 1 -> <|"b" -> 2|>|>`.
   Also add the operator form `KeyGroupBy[f]` (1-arg — unambiguous
   since the data form needs the association first) via the matcher IF
   it is trivial; otherwise defer and say so (data form is the
   priority).
2. **`AssociationKeyDeflatten[flatAssoc]`** — a flat association whose
   keys are lists becomes a nested association. Pin:
   `AssociationKeyDeflatten[<|{"a", "b"} -> 1, {"a", "c"} -> 2|>]` →
   `<|"a" -> <|"b" -> 1, "c" -> 2|>|>`. A key that is not a list is
   treated as a length-1 path. Deterministic key order = first
   appearance.
3. **`AssociationKeyFlatten[nestedAssoc]`** — the inverse: a nested
   association becomes flat with list keys. Pin:
   `AssociationKeyFlatten[<|"a" -> <|"b" -> 1|>|>]` →
   `<|{"a", "b"} -> 1|>`. Round-trip: deflatten∘flatten and
   flatten∘deflatten are identity on well-formed inputs (pin one
   round-trip test each way). A non-association leaf stops the
   recursion (it is a value).
4. **`MergeNested[{assoc…}, f]`** — recursive `Merge` for nested
   associations (mirror `eval_merge` in
   `crates/rum-stdlib/src/associations/resumable/aggregate.rs` at the
   leaf level: `f` is applied to the LIST of values collected for a
   leaf key, INCLUDING singletons, exactly like `Merge`). Recurse while
   both sides at a key are associations; stop at the shallowest
   non-association. Pin:
   `MergeNested[{<|"db" -> <|"host" -> "a"|>|>, <|"db" -> <|"port" -> 5432|>|>}, Last]`
   → `<|"db" -> <|"host" -> "a", "port" -> 5432|>|>`. And a conflict
   case: `MergeNested[{<|"a" -> 1|>, <|"a" -> 2|>}, Last]` →
   `<|"a" -> 2|>` (matches `Merge` on flat data). Also add the operator
   form `MergeNested[f]` (1-arg, unambiguous — data form needs the list
   first) if trivial via the matcher.

Non-association / malformed inputs stay inert symbolic calls (do not
error), consistent with the family's fail-closed style.

## Wiring (each new builtin)

- Implementations in `crates/rum-stdlib/src/associations/` (a new
  neutrally-named submodule such as `key_reshape.rs` for the
  flatten/deflatten/group trio and `MergeNested` near `Merge`, or
  wherever the diff is smallest — report where; NO inspiration-source
  naming).
- Catalog entries in `crates/rum-stdlib/src/catalog/associations.rs`
  via `BuiltinSpec::ordinary`.
- **Kernel vocabulary**: add the four names to
  `STDLIB_EMPTY_ATTRIBUTE_NAMES` in
  `crates/rum-runtime/src/runtime/builtin_catalog/stdlib_attributes.rs`
  (the `catalog_attributes_match_kernel_vocabulary` gate).
- Any operator forms go in the existing matcher
  (`functional_data.rs`); if you touch the matcher, run the microbench
  A/B (alternating, saved binaries) and report it; if you add NO
  operator forms, say so and skip the A/B.
- Tests + regenerate `docs/reference/builtins.md` (count rises by 4).

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
printf '' | cargo run -q -p rum-cli -- --eval '{KeyGroupBy[<|"aa" -> 1, "b" -> 2|>, StringLength], AssociationKeyDeflatten[<|{"a", "b"} -> 1, {"a", "c"} -> 2|>], AssociationKeyFlatten[<|"a" -> <|"b" -> 1|>|>], MergeNested[{<|"db" -> <|"host" -> "a"|>|>, <|"db" -> <|"port" -> 5432|>|>}, Last], MergeNested[{<|"a" -> 1|>, <|"a" -> 2|>}, Last]}'
# round-trip + inert:
printf '' | cargo run -q -p rum-cli -- --eval '{AssociationKeyDeflatten[AssociationKeyFlatten[<|"a" -> <|"b" -> 1, "c" -> 2|>|>]], KeyGroupBy[x, f], MergeNested[y, Last]}'
git status --short
```

Expected: fmt clean; workspace green (report totals — four new tests
minimum, vocabulary-match test still passes); probe 1 matches the
pinned outputs; probe 2 shows the round-trip identity
`<|"a" -> <|"b" -> 1, "c" -> 2|>|>` and the two non-association calls
inert. Paste all.

## Scope

- MAY modify: `crates/rum-stdlib/src/associations/**`,
  `crates/rum-stdlib/src/catalog/associations.rs`,
  `crates/rum-stdlib/src/lib.rs` (only if explicit registration is
  needed),
  `crates/rum-runtime/src/runtime/builtin_catalog/stdlib_attributes.rs`
  (the four names),
  `crates/rum-runtime/src/runtime/kernel_data/functional_data.rs`
  (only if adding operator forms),
  `crates/rum-stdlib/src/docgen.rs`, `crates/rum-stdlib/src/tests/**`,
  `crates/rum-stdlib/src/tests.rs`, `docs/reference/builtins.md`
  (regenerated).
- MUST NOT touch: existing builtins' behavior (`Merge`/`GroupBy` stay
  as-is), other catalog families, the guide, `TODO.md`, the tracker,
  findings files, `Cargo.toml` files.

## Hard rules

- Do NOT `git commit`, branch, push. No network, no new dependencies,
  no new `unsafe`. Independent folk implementations only.
- Only additions; no existing behavior changes. The four new names
  appear in BOTH the catalog and the kernel vocabulary. No source file
  named after an inspiration source.
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format

1. What changed — per file; where each implementation lives (confirm
   neutral naming); whether operator forms were added (and the A/B if
   so).
2. Verification — all commands verbatim.
3. Stop-loss / open questions (empty if none).
