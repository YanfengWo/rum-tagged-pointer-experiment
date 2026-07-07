# Brief 174 — Unit-2 fixes 5 (FINAL): WFR-inspired list additions

Implementation brief. Unit-2 fix batch 5, the last one, closing the
structure+functional unit. Three NEW builtins whose demand is
evidenced by the Wolfram Function Repository survey
(`docs/dev/wfr-core-survey.md`). ABSORPTION IS IDEA-LEVEL: these are
independent folk implementations written from the described behavior
below; no WFR source was read. Data forms only — no operator forms in
this brief (they fold into a later matcher touch; keep this batch
call-planning-free and correctness-only).

## New builtins (add to the `structure` family)

1. **`SelectPositions[list, crit]`** and **`SelectPositions[list, crit, n]`**
   — the position-returning sibling of `Select`. Returns the 1-based
   positions (each as a one-element `{i}` path, matching
   `Position`'s convention — CHECK `Position`'s output shape in this
   codebase first and match it exactly; if `Position` returns bare
   integers for level-1, match THAT) of top-level elements where
   `crit[element]` is `True`; the 3-argument form returns the first
   `n` such positions. Non-list first argument → inert symbolic call.
   Pin the convention you chose with a test and state it in the
   catalog description.
2. **`Duplicates[list]`** — the elements that occur two or more times
   in `list`, each returned ONCE, in order of first appearance. Empty
   list result when there are none. Structural equality (same as
   `DeleteDuplicates`'s default). Non-list argument → inert symbolic
   call.
3. **`BinarySearch[sortedList, val]`** — folk membership search over an
   ascending-sorted list using rum's canonical order comparison.
   Returns the 1-based position of an element equal to `val`, or
   `Missing["NotFound"]` if absent. Contract (document in the catalog
   description AND a docgen LIMITS note): assumes the list is sorted
   ascending; does NOT validate sortedness (that would defeat the
   O(log n) purpose — the WFR precedent documents the same tradeoff);
   on a non-list first argument, inert symbolic call. If `val` is
   incomparable to the list elements under canonical order, return
   `Missing["NotFound"]` (do not error).

## Wiring (each new builtin)

- Implementation in `crates/rum-stdlib/src/structure_ops/` (new
  submodule or an existing fitting file — smallest sensible diff;
  report where).
- Catalog entry in `crates/rum-stdlib/src/catalog/structure.rs` via
  `BuiltinSpec::ordinary` (mirror `Duplicates`-neighbor entries; these
  three take no special attributes).
- **Kernel vocabulary**: add the three names to
  `STDLIB_EMPTY_ATTRIBUTE_NAMES` in
  `crates/rum-runtime/src/runtime/builtin_catalog/stdlib_attributes.rs`
  (the `catalog_attributes_match_kernel_vocabulary` test enforces
  this — the standing gate lesson for new stdlib builtins).
- Register the `install_core` path if BYTE-family-style explicit
  registration is needed (check how the structure family installs;
  most likely the catalog array is already iterated — confirm and
  report).
- Tests + regenerate `docs/reference/builtins.md` (three new entries;
  count rises by 3).

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
printf '' | cargo run -q -p rum-cli -- --eval '{SelectPositions[{1, 2, 3, 4}, EvenQ], SelectPositions[{1, 2, 3, 4, 6}, EvenQ, 2], Duplicates[{1, 2, 1, 3, 2, 2}], Duplicates[{1, 2, 3}], BinarySearch[{1, 3, 5, 7, 9}, 7], BinarySearch[{1, 3, 5, 7, 9}, 4], BinarySearch[{a, b, c}, b]}'
# inert on non-list / symbolic:
printf '' | cargo run -q -p rum-cli -- --eval '{SelectPositions[x, EvenQ], Duplicates[x], BinarySearch[x, 3]}'
git status --short
```

Expected: fmt clean; workspace green (report totals — three new tests
minimum, and the vocabulary-match test still passes); probe 1 gives
the positions (in your pinned convention), `{1, 2}` (the duplicated
values), `{}`, position `4`, `Missing["NotFound"]`, position `2`;
probe 2 keeps all three as inert symbolic calls. Paste all.

## Scope

- MAY modify: `crates/rum-stdlib/src/structure_ops/**`,
  `crates/rum-stdlib/src/structure_ops.rs`,
  `crates/rum-stdlib/src/catalog/structure.rs`,
  `crates/rum-stdlib/src/lib.rs` (only if explicit registration is
  needed), `crates/rum-runtime/src/runtime/builtin_catalog/stdlib_attributes.rs`
  (the three names), `crates/rum-stdlib/src/docgen.rs` (BinarySearch
  LIMITS note), `crates/rum-stdlib/src/tests/**`,
  `crates/rum-stdlib/src/tests.rs`, `docs/reference/builtins.md`
  (regenerated).
- MUST NOT touch: the operator-form matcher in `functional_data.rs`,
  existing builtins' behavior, other catalog families, the guide,
  `TODO.md`, the tracker, findings files, `Cargo.toml` files.

## Hard rules

- Do NOT `git commit`, branch, push. No network, no new dependencies,
  no new `unsafe`. Independent folk implementations only — no external
  source consulted.
- Only additions; no existing behavior changes. The three new names
  must appear in BOTH the catalog and the kernel vocabulary.
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format

1. What changed — per file; the `SelectPositions` position convention
   chosen (matching `Position`); where each implementation lives.
2. Verification — all commands verbatim.
3. Stop-loss / open questions (empty if none).
