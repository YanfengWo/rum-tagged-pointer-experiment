# Brief 171 — Unit-2 fixes 2: structural spec flexibility

Implementation brief. Unit-2 fix batch 2 of the Usability Pass
(tracker `docs/dev/usability-pass.md`, unit-2 triage item 2; evidence
E3 in `docs/dev/usability-pass/structure-functional-findings.md`).
Six items, all opt-in: every currently-accepted call stays
byte-identical; currently-rejected documented forms gain meaning.
None of this touches call planning — per-builtin argument parsing and
implementations only.

## Changes

1. **`Subsets` size specs — its own diagnostic currently LIES**
   (safety): `Subsets::nninfseq` advertises
   `All | Infinity | nmax | {nmin} | {nmin, nmax} | {nmin, nmax, dn}`
   but the implementation rejects `All`, `Infinity`, and stepped
   ranges (E3;
   `crates/rum-stdlib/src/structure_ops/combinatorial_lists.rs`).
   Implement the advertised forms with the WL-documented meanings
   (`All`/`Infinity` ≡ all lengths; `{k}` exactly k; `{kmin, kmax}`
   inclusive range; nonzero step `dn`, negative steps enumerate
   descending lengths). Subset ordering within each length must stay
   identical to today's.
2. **`PadLeft`/`PadRight` list dimensions**: accept a list of
   machine-size non-negative integers as the target-shape argument for
   rectangular nested lists (`PadLeft[{{1},{2}}, {2, 3}]` →
   `{{0,0,1},{0,0,2}}`; with an explicit pad element likewise). Scalar
   behavior byte-identical. WL's margin argument is OUT of scope this
   batch — it must keep today's rejection, and say so in the report.
3. **`Partition` alignment/padding forms** (folk subset of the WL
   surface, FLAT lists only): add the documented 4th argument
   `{kL, kR}` alignment and 5th argument padding (single element,
   cyclic padding list, or `{}` for ragged no-padding). Nested/level
   forms stay rejected. 2- and 3-argument behavior byte-identical.
   If the WL contract for a corner is ambiguous from documentation,
   choose the conservative reading, TEST it, and document it in the
   catalog description — do not guess silently.
4. **`Transpose` general permutations**: accept any positive
   permutation list matching the tensor rank on rectangular nested
   lists (E3 rejected `{3, 2, 1}` on a 2×2×2). Identity and
   first-two-swap behavior byte-identical; non-rectangular input keeps
   today's behavior.
5. **`Reverse[expr, All]`**: accept `All` as the level spec (reverse
   at every level). Integer/list level specs byte-identical.
6. **`Riffle` positional forms**: `Riffle[list, x, n]` (insert x
   every n elements) and `Riffle[list, x, {imin, imax, dn}]` per the
   WL-documented meanings. Two-argument behavior byte-identical.

Message hygiene rule for ALL items: after the change, every message a
builtin emits must describe forms it actually accepts/rejects — no
lying diagnostics in either direction. Update `docgen.rs` LIMITS notes
where they state the old contracts.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
printf '' | cargo run -q -p rum-cli -- --eval '{Subsets[{a, b, c}, All], Subsets[{a, b, c}, {1, 3, 2}], PadLeft[{{1}, {2}}, {2, 3}], PadRight[{{1}, {2}}, {2, 3}, x], Partition[{a, b, c, d}, 2, 2, {1, 1}, z], Partition[{a, b, c, d, e}, 2, 2, {1, 1}, {}], Transpose[{{{a, b}, {c, d}}, {{e, f}, {g, h}}}, {3, 2, 1}], Reverse[{{a, b}, {c, d}}, All], Riffle[{a, b, c, d}, x, 3]}'
# defaults byte-identical:
printf '' | cargo run -q -p rum-cli -- --eval '{Subsets[{a, b, c}, 2], PadLeft[{1, 2}, 4], Partition[{a, b, c, d}, 2], Partition[{a, b, c, d, e}, 2, 1], Transpose[{{a, b}, {c, d}}], Reverse[{a, b, c}], Riffle[{a, b, c}, x]}'
git status --short
```

Expected: fmt clean; workspace green (report totals); probe 1 all
evaluate (no messages; paste outputs — they are review evidence);
probe 2 byte-identical to today (paste).

## Scope

- MAY modify: `crates/rum-stdlib/src/structure_ops/**`,
  `crates/rum-stdlib/src/structure_ops.rs` (if a root file exists),
  `crates/rum-stdlib/src/catalog/structure.rs` (descriptions),
  `crates/rum-stdlib/src/docgen.rs` (LIMITS notes),
  `crates/rum-stdlib/src/tests/**`, `crates/rum-stdlib/src/tests.rs`,
  `docs/reference/builtins.md` (regenerated if descriptions changed).
- MUST NOT touch: functional_ops, call dispatch/planning anywhere,
  other catalogs, the guide, `TODO.md`, the tracker, findings files.

## Hard rules

- Do NOT `git commit`, branch, push. No network, no new dependencies,
  no new `unsafe`.
- Every currently-accepted call byte-identical; changes only give
  meaning to currently-rejected documented forms. Remaining rejections
  keep accurate messages.
- Minimal diff. Stop-loss: same failure twice → STOP and report; if
  item 3's WL contract proves too intricate for a folk subset, land
  the unambiguous core, message the rest accurately, and report the
  cut.
- Paste real verbatim verification output; never claim green without it.

## Report format

1. What changed — per file, per item; any conservative readings chosen
   for Partition corners; what remains rejected (with its message).
2. Verification — all commands verbatim.
3. Stop-loss / open questions (empty if none).
