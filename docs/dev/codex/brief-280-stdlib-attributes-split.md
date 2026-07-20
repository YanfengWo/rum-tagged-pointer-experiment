# Brief 280 — split `stdlib_attributes.rs` along its name ordering

`crates/rum-runtime/src/runtime/builtin_catalog/stdlib_attributes.rs` is
30,747 bytes — inside AGENTS.md's `20k`–`35k` split-plan range and
still growing (it was ~28k when accepted as debt on 2026-07-18; briefs
262-279 pushed it here). User ruling 2026-07-20: clear it now rather
than wait for it to cross `35k`.

## Why the earlier "no family boundary" ruling does not block this

The 2026-07-18 decision to accept this file as debt reasoned that it is
a `binary_search` over ONE sorted const with no family structure to
split on. That is correct — and it is exactly why the split axis must
be the **name ordering itself**, not a family taxonomy. The table's only
structure is its ASCII sort order, so chunk it along that.

Two facts make this cheap:

- `stdlib_builtin_attribute_specs()` (line 678) already merges
  `STDLIB_NON_EMPTY_ATTRIBUTE_SPECS` with `STDLIB_EMPTY_ATTRIBUTE_NAMES`
  into a `Vec` and calls `sort_unstable_by` at line 693. It does not
  depend on the const being one array — only on the contents.
- `default_stdlib_attribute_set_for_symbol()` (line 699) is the only
  place that binary-searches the const directly, and it already has the
  lookup key (`symbol.name()`) in hand before searching.

## Target shape

Keep `stdlib_attributes.rs` as the parent module (Rust `foo.rs` +
`foo/` pattern, as `pattern_matching/sequence.rs` + `sequence/` and
`builtin_catalog/namespace/mod.rs` already do in this repo — follow
whichever of those two idioms fits; parent-file + child-dir is
preferred here since the parent keeps real logic).

- **Parent `stdlib_attributes.rs` keeps**: the 23 `ATTR_*`
  `AttributeSet` constants (lines 7-95), `STDLIB_EMPTY_ATTRIBUTE_NAMES`
  (3 entries, line 619 — do NOT split it), the `const fn` sortedness
  helpers, ALL compile-time assertions, and both public functions.
- **New children** hold only the spec data, split from
  `STDLIB_NON_EMPTY_ATTRIBUTE_SPECS` by first byte of `name`, in ASCII
  order. Measured distribution of its 460 entries gives these four
  roughly-balanced chunks:

  | file | byte range of first char | entries |
  |---|---|---|
  | `stdlib_attributes/specs_a_e.rs` | `< b'F'` | 123 |
  | `stdlib_attributes/specs_f_m.rs` | `b'F'..=b'M'` | 130 |
  | `stdlib_attributes/specs_n_r.rs` | `b'N'..=b'R'` | 84 |
  | `stdlib_attributes/specs_s_z.rs` | `>= b'S'` | 123 |

  Note the first chunk is `< b'F'`, not `b'A'..=b'E'`: names beginning
  with `$` (ASCII `0x24`) sort below `A`, and although none are in this
  array today, `$`-prefixed system symbols exist elsewhere. The last
  chunk is open-ended for the same reason. **Every possible name must
  land in exactly one chunk.**

  Each child declares one `pub(super) const SPECS: &[Spec] = &[…];`
  (name it consistently across the four) and starts with `use super::*;`
  so the `ATTR_*` constants and the `Spec` alias resolve. Move the
  entries VERBATIM — do not reformat, reorder, or re-derive any
  attribute set.

## Lookup (must stay const-time; no `OnceLock` on this path)

`default_stdlib_attribute_set_for_symbol` dispatches on the first byte,
then binary-searches that one chunk:

```rust
const fn chunk_for_name(name: &str) -> &'static [Spec] {
    match name.as_bytes() {
        [] => &[],
        [first, ..] if *first < b'F' => specs_a_e::SPECS,
        [first, ..] if *first <= b'M' => specs_f_m::SPECS,
        [first, ..] if *first <= b'R' => specs_n_r::SPECS,
        _ => specs_s_z::SPECS,
    }
}
```

(Shape it however compiles cleanly — `const fn` is preferred but not
required. What matters: one chunk selected by first byte, then ONE
`binary_search_by` over it, and **no `OnceLock` / lazy init / allocation
introduced on this path.** It is reached from `attribute_set_for_symbol`
on matcher-hot paths.)

`stdlib_builtin_attribute_specs()` changes only by extending from the
four chunks instead of one array, before the existing sort. Its output
must be **identical**, including order.

## Compile-time guarantees must not weaken

Today one `const _: () = assert!(attribute_specs_strictly_sorted(...))`
gives strict global ASCII sortedness, which also forbids duplicates.
That assertion was added deliberately (commit `8dcc80e`, after a
mis-sorted insert bug) and **this brief must not trade it down.**

Replace it with assertions of equal strength:

1. Each chunk is strictly ASCII-sorted — reuse the existing
   `attribute_specs_strictly_sorted` on each of the four.
2. **Each chunk's every entry actually falls in that chunk's byte
   range.** Write a new `const fn` (e.g.
   `attribute_specs_within_byte_range(specs: &[Spec], lo: u8, hi: u8) -> bool`)
   and assert it per chunk with that chunk's bounds.

(1) + (2) together are strictly equivalent to today's single global
assertion: the ranges are disjoint and cover all bytes, so a duplicate
name cannot span two chunks, and concatenating the chunks in file order
reproduces the original globally sorted array. State that argument in a
comment above the assertions, replacing the current lines 621-625
comment (keep its substance — that an out-of-place insert silently
breaks lookups).

Also keep `STDLIB_EMPTY_ATTRIBUTE_NAMES`'s existing assertion untouched.

## Behavior must be provably unchanged

This is a MECHANICAL split. Prove it, don't assert it:

1. BEFORE any edit, on `HEAD`, add a temporary test that prints every
   `(name, attributes)` pair from `stdlib_builtin_attribute_specs()` in
   iteration order, run it, and save the output to
   `/tmp/brief280_specs_base.txt`.
2. After the split, run the same test and save to
   `/tmp/brief280_specs_cand.txt`.
3. `diff` them — it MUST be empty. Paste the diff command and its
   (empty) output.
4. Delete the temporary test before finishing. It must not appear in
   the final diff.

Additionally the generated builtin reference must not drift (there is a
reference drift test in the suite) and
`crates/rum-stdlib/src/tests.rs`'s vocabulary test — which already
asserts no duplicate names in `stdlib_builtin_attribute_specs()` — must
stay green. Do not modify either test.

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/builtin_catalog/stdlib_attributes.rs`;
  - NEW `crates/rum-runtime/src/runtime/builtin_catalog/stdlib_attributes/*.rs`
    (the four chunk files; add a `mod.rs` only if you choose that idiom);
  - `crates/rum-runtime/src/runtime/builtin_catalog.rs` — MECHANICAL
    ONLY, and only if the module declaration at line 12 needs adjusting.
- Files/areas you MUST NOT touch: `crates/rum-stdlib/**` (including its
  tests), `kernel_api.rs`, `docs/reference/**`, `TODO.md`,
  `PROGRESS.md`, `AGENTS.md`, `.git`, `Cargo.lock`. If you believe you
  must, STOP and report.
- Do NOT change any symbol's attributes. Not one. If you notice
  something that looks wrong in the data, report it — do not fix it
  here.
- No new `unsafe`.

## Stop-loss

If the byte-range chunk dispatch cannot be made to compile as a
`const fn`, fall back to a plain `fn` and say so — that is not a
stop-loss. Genuine stop-loss triggers:
- the before/after spec dump diff is non-empty and you cannot make it
  empty without changing data;
- preserving compile-time strength as described proves impossible;
- the split requires touching a MUST-NOT file.

Then revert every change and report. (`git checkout --` fails under the
sandbox with `Unable to create '.git/index.lock': Operation not
permitted` — restore by scoped patch instead and say so.)

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test --workspace
```

Baseline is `1654 passed / 0 failed` on `ae331b6`; state your totals.
Report the final byte size of every file you created or modified
(`ls -l`), and confirm each is comfortably under `20k`.

No microbench A/B is required: this is a data-layout move with no
algorithmic change, and the lookup keeps the same shape (one
binary_search over a const, now over a shorter one). **If you introduce
anything lazy or allocating on the `default_stdlib_attribute_set_for_symbol`
path, that assumption breaks — in that case stop and report instead.**

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes
  in the working tree; Claude runs the authoritative gate and commits.
- Do NOT modify files outside Scope. If you believe you must, STOP and
  write that in the report instead.

## Report format

1. What you changed, file by file, with `ls -l` sizes.
2. Verbatim verification output (fmt, test totals, the empty spec-dump
   diff).
3. The compile-time-guarantee argument as you implemented it.
4. Stop-loss / open questions.
