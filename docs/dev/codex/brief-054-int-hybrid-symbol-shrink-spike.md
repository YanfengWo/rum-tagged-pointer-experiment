# brief-054: DESIGN SPIKE — `Atom::Integer` small-int hybrid + `Symbol` identity niche-pack (Expr 40→32). Do NOT commit.

## Goal

The FINAL safe-Rust per-node Expr-compression step (Phase 3 rungs 2+3, combined
because neither alone moves the metric — `Integer` and `Symbol` are BOTH 32 B and
co-cap the enum). Two changes, measured together, target `size_of::<Expr>()` **40 →
32** and the dense-list `retained` ratio **~5.3x → ~4.1x**:

- **Rung 2** — `Atom::Integer(BigInt)` → `Atom::Integer(IntValue)` where `IntValue`
  is a small-int/bigint hybrid (`Small(i64)` inline, `Big(Box<BigInt>)` for the rest).
  Small integers stop heap-allocating a `BigInt` digit buffer (the per-element alloc
  for dense integer data) and the variant caps at 16 B.
- **Rung 3** — `Symbol.identity: Option<u64>` → `Option<NonZeroU64>` (niche-packs
  16 B → 8 B, shrinking `Symbol` 32 → 24).

After both, the `Atom` cap is `String`=24 → `Atom` 24+8(tag) = 32 → `Expr` 32.
**This is a SPIKE: implement, MEASURE all axes, REPORT — do NOT `git commit`.**
Claude gates (incl. an mma-hol arith/rat corpus re-run) and decides keep/revert.
Pre-diagnosed sizes (measured): `Expr`=40, `Atom`=40, `BigInt`=32, `Symbol`=32,
`Rational`=64 (boxed in rung 1). Predictions are NOT conclusions — the built+measured
numbers decide.

## Design — pin this exactly (the correctness contract matters)

### Rung 2: `IntValue` (in `crates/rum-core/src/expr.rs`)

```rust
pub enum IntValue { Small(i64), Big(Box<BigInt>) }
```

INVARIANT (the whole correctness story rests on it): **`Big` NEVER holds a value that
fits in `i64`.** Every construction normalizes. With the invariant, two `IntValue`s
are equal iff their variants+payloads are equal, so derived `PartialEq/Eq/Hash/Ord`
are correct AND fast — but you MUST enforce normalization at every construction.

- `IntValue::from_bigint(b: BigInt) -> IntValue`: `i64::try_from(&b)` → `Small`, else
  `Big(Box::new(b))`. The ONLY way to build a `Big`.
- `IntValue::from_i64(n: i64) -> IntValue` = `Small(n)`.
- `to_bigint(&self) -> BigInt`: `Small(n)` → `BigInt::from(n)`; `Big(b)` → `(**b).clone()`.
- `as_i64(&self) -> Option<i64>`, `is_zero`, `sign`, etc. as needed by call sites.
- `Display` matches BigInt's; `Serialize/Deserialize` must round-trip to the SAME
  shape the old `Integer(BigInt)` produced if any serialized form is compared (the
  text snapshots re-parse, so prefer serializing as the integer value, not the enum —
  verify difftest/host serialization still matches). If a custom serde impl is needed
  to preserve the wire/text shape, write it; do not silently change the serialized
  form.
- `Ord`/`PartialOrd`: MUST order by numeric value. With the invariant, `Small < Big`
  in magnitude is NOT guaranteed by variant order (a `Big` could be very negative).
  So implement `Ord` by comparing `to_bigint()` OR by a correct value comparison
  (e.g. `Small` vs `Small` direct; any `Big` involved → compare via `to_bigint`).
  Do NOT derive `Ord` if variant order != value order. (Hash with the invariant: a
  value has exactly one representation, so hashing the variant+payload is consistent
  with value-`Eq` — but document this.)

`Atom::Integer(IntValue)`. Keep `Expr::integer(value: BigInt)` SIGNATURE unchanged
(it calls `IntValue::from_bigint` internally) so the ~190 `Expr::integer(BigInt::...)`
call sites do not change. Optionally add `Expr::small_integer(i64)`.

The ~55 `Atom::Integer(value)` match sites currently treat `value` as `&BigInt`
(mostly `value.clone()` then BigInt ops). Change each to obtain a `BigInt` via
`value.to_bigint()` where it needs one (arithmetic stays correctness-identical — it
still computes on `BigInt`; only STORAGE changes). Do NOT attempt an i64 fast-path
arithmetic in this spike (overflow/division edge cases = separate risk); going through
`BigInt` for the operation and normalizing the RESULT via `Expr::integer` is the
low-risk path that still wins the storage alloc + the size shrink.

### Rung 3: `Symbol.identity: Option<NonZeroU64>`

- Change the field type; `with_identity(.., identity: u64)` must convert
  (`NonZeroU64::new(identity).expect("symbol identity must be non-zero")` OR change the
  param to `NonZeroU64`). Audit the `with_identity` callers (`resolution.rs:145`,
  `binding_substitution.rs:527`, and the gensym allocator in `scoping/unique.rs`) +
  the identity-generating counter: ensure it starts at 1 and never yields 0. If any
  site legitimately uses identity 0, STOP and report (do not paper over it).
- `identity()` returns `Option<NonZeroU64>` (or `Option<u64>` via `.map(NonZeroU64::get)`
  to minimize downstream churn — pick the lower-churn option and keep callers working).
- The `Hash`/`Ord` impls (expr.rs ~:191/:208) use the `identity` as a `u64` with
  `None → 0`; since real identities are now ≥1, the `None=0` sentinel no longer
  collides — keep that behavior (`None → 0`, `Some(nz) → nz.get()`).

## Scope

- MAY modify: `crates/rum-core/src/expr.rs` (IntValue + Symbol.identity), the ~55
  `Atom::Integer` match sites + `with_identity` callers across `crates/rum-core`,
  `crates/rum-runtime`, `crates/rum-difftest`, and the `expr_memory_proxy_bytes`
  Integer arm in `memory.rs` (so `ByteCount` reflects the new layout: `Small` adds 0
  heap bytes, `Big` adds `size_of::<BigInt>()` + buffer). A throwaway `size_of` test
  is OK if deleted before finishing.
- MUST NOT touch: any `mma-hol` clone; the `Atom::Rational` boxing from rung 1;
  `AGENTS.md`/`PROGRESS.md`/`TODO.md`/`docs/**` except your report; `.git`, `Cargo.lock`.
- SPIKE: do NOT `git commit`.

## Verification (MANDATORY — run all, paste verbatim; this IS the deliverable)

**Axis 0 — correctness.** `cargo fmt --check` && `cargo test --workspace`. MUST be
green; exact integer/rational arithmetic must be byte-identical (the invariant is the
risk — a `Big` holding a small value would break `Eq`/`Hash`). Paste result lines.

**Axis 0b — sizes.** Throwaway test printing `size_of::<Expr>/Atom/Symbol/IntValue>`
(then delete). Expect Expr 40→**32**, Atom→32, Symbol 32→**24**, IntValue→16.

**Axis 1 — memory retained (the metric).**
`bash scripts/core-memory-microbench.sh --engine both --output-dir /tmp/cdx_054_mem`.
Paste the `retain-range-1000/10000/100000` `retained` columns. Expect `rum_retained`
~34000/340000/3.4M and `retained_ratio` ~5.3 → **~4.1x**. (Read `retained` only.)

**Axis 2 — allocation traffic (rung 2's real win).** Measure total allocations for a
dense-integer workload before vs after, using the alloc-count instrument
(`crates/rum-cli/src/alloc_count.rs`, env `RUM_PROFILE_ALLOC` — inspect it and use its
documented invocation; build with whatever feature it needs). Workload:
`Range[100000]`. Expect a LARGE drop in allocation count (≈100000 fewer per-element
`BigInt` allocs). Paste the before/after alloc counts. If the instrument cannot be
invoked cleanly, say so and report what you tried (do not fake it).

**Axis 3 — time (regression check).** Build BEFORE (parent, `git stash push -- crates/`)
and AFTER (working tree) release binaries to `/tmp/rum_base` + `/tmp/rum_new`
(`git stash pop` to restore). Then, interleaved, 5+ runs each, internal AbsoluteTiming
medians:
- integer-arith probe: `N[AbsoluteTiming[Do[s=0; Do[s=s+k, {k,1,2000}], {200}]][[1]]]`
- rational probe: `N[AbsoluteTiming[Do[s=0; Do[s=s+1/k, {k,1,400}], {200}]][[1]]]`
- `bash scripts/core-language-microbench.sh --engine rum --rum-cli /tmp/rum_base ...`
  and `... --rum-cli /tmp/rum_new ...`; paste the hot-label timings base vs new.
Expect neutral or slightly better (smaller Expr). Report the medians + ratios.

## Hard rules

- Do NOT `git commit`/branch/push. Leave uncommitted for Claude's gate.
- Do NOT modify files outside Scope. If a site needs a non-mechanical change beyond
  `to_bigint()`/`Box::new`, note it.
- Full machine access, repo only, no network. Commands: edits, `cargo`, the two
  microbench scripts, the release `rum-cli`, the alloc-count instrument.
- Never edit any `mma-hol` file.
- Stop-loss: if `cargo test --workspace` is red after two focused fix passes for the
  same failure, STOP and report (do NOT loosen a test, do NOT relax the IntValue
  invariant to pass). Same for any identity-0 site.
- Measure honestly: internal AbsoluteTiming only, profiling OFF for timing, one
  variable per run; if an axis is too noisy to call, say so.
- Paste REAL verbatim numbers; never fabricate.

## Report format

1. What changed — per file + line ranges; the `Atom::Integer` match-site count and
   `with_identity` caller count actually touched; whether a custom serde/Ord impl was
   needed and why.
2. Verification — every command + verbatim output for all axes (test lines; the
   size_of lines; the retained rows; the alloc before/after; the timing medians).
3. The verdict — Expr 40→32? retained ratio before/after; alloc-traffic drop;
   time neutral? Plus: did the IntValue invariant hold across the whole test suite?
4. Stop-loss / open questions (empty if none). DECISION IS CLAUDE'S — report only.
