# Brief 171b — Partition alignment anchoring correction (amends 171 in-tree)

Correction brief. Brief-171's working tree is UNCOMMITTED and stays as
your starting point; five of its six items passed review. Item 3
(`Partition` `{kL, kR}` alignment) implemented an INVERTED anchor: the
current tree pins `Partition[{a,b,c,d}, 2, 2, {1,1}]` →
`{{a},{b,c},{d}}` with padding/raggedness at the FRONT. The WL anchor
convention is the opposite, and the front door of this feature is the
classic ragged-TAIL chunking idiom. Fix ONLY the Partition anchoring
(implementation + its tests + its catalog/LIMITS text); everything
else in the tree stays byte-identical.

## The anchor rule to implement (and document in the catalog text)

For `Partition[list, n, d, {kL, kR}, pad]` on flat lists:

- `kL` anchors the FIRST element of `list` at position `kL` of the
  FIRST block: prepend exactly `kL - 1` fill elements. `kL = 1` means
  NO left fill.
- Blocks of size `n` step by `d`. Emit d-stepped blocks until the LAST
  element of `list` has appeared at position ≥ `kR` in an emitted
  block; do not emit further blocks.
- Right side: with a pad element (or cyclic pad list), incomplete
  emitted blocks are filled to length `n`; with `{}` they stay ragged;
  with the 4-argument form (no pad argument) the list itself continues
  cyclically (WL's default padding is the list treated cyclically).
- Keep the existing folk restrictions from the tree: positive
  `kL`/`kR` ≤ n, `Partition::align` message otherwise; nested/level
  forms stay rejected.

## Canonical cases that MUST hold (pin all three as tests)

```
Partition[{a, b, c, d, e}, 2, 2, {1, 1}, {}]  ->  {{a, b}, {c, d}, {e}}
Partition[{a, b, c, d}, 2, 2, {1, 1}, z]      ->  {{a, b}, {c, d}}
Partition[{a, b, c, d}, 2, 2, {1, 1}]         ->  {{a, b}, {c, d}}
```

(the ragged-tail chunking idiom; no spurious padding when both anchors
are already satisfied; cyclic default likewise). Also pin one
left-fill case, e.g. `Partition[{a, b, c, d}, 2, 2, {2, 1}, z]` →
`{{z, a}, {b, c}, {d, z}}`, and one `kR` case with `d < n` of your
choice — derive it from the anchor rule, document the reading if any
corner stays ambiguous.

## Scope

- MAY modify: `crates/rum-stdlib/src/structure_ops/combinatorial_lists.rs`
  (Partition paths only), the Partition tests in
  `crates/rum-stdlib/src/tests/structure_ops/**`,
  `crates/rum-stdlib/src/catalog/structure.rs` (Partition description),
  `crates/rum-stdlib/src/docgen.rs` (Partition LIMITS note),
  `docs/reference/builtins.md` (regenerated).
- MUST NOT touch: the other five items' code/tests/behavior
  (Subsets/PadLeft/PadRight/Transpose/Reverse/Riffle stay exactly as
  the tree has them), anything outside the files above.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
printf '' | cargo run -q -p rum-cli -- --eval '{Partition[{a, b, c, d, e}, 2, 2, {1, 1}, {}], Partition[{a, b, c, d}, 2, 2, {1, 1}, z], Partition[{a, b, c, d}, 2, 2, {1, 1}], Partition[{a, b, c, d}, 2, 2, {2, 1}, z], Partition[{a, b, c, d}, 2], Partition[{a, b, c, d, e}, 2, 1]}'
git status --short
```

Expected: fmt clean; workspace green; the four alignment outputs match
the canonical cases above; the last two (2- and 3-argument forms)
byte-identical to today.

## Hard rules

- Do NOT `git commit`, branch, push. No network, no new dependencies,
  no new `unsafe`.
- The diff relative to the CURRENT tree touches Partition only.
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format

1. What changed relative to the in-tree state; the final documented
   anchor rule; any corner readings.
2. Verification — all commands verbatim.
3. Stop-loss / open questions (empty if none).
