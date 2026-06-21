# Brief 012 — Layer B (args compaction): `SharedArgs` backing = `Arc<[Expr]>`

## Goal

Layer B's flaw-2 fix, low-risk variant. The plan originally framed the
single-allocation node as a 572-site migration of `Expr::Call { head, args }`.
A scope audit found a far less invasive path: the dominant per-node allocation
is the **args**, currently `SharedArgs(Arc<Vec<Expr>>)` = an `Arc` box PLUS a
separate `Vec` heap buffer (two allocations). Swapping the backing to
`Arc<[Expr]>` (a single contiguous allocation holding the refcount header and
the elements inline) cuts that to **one** allocation per node and improves cache
locality, **without changing the `Expr::Call { head, args }` variant shape** —
so none of the ~572 destructure sites change. `SharedArgs` is referenced in only
~55 places; ~837 arg reads go through `as_ref()` / `iter()` / indexing, all of
which work identically on a slice.

This is the WolframKernel-style compact-args representation, scoped to
`rum-core`. The head (`SharedExpr`) compaction and the inline normal-form stamp
(the `34x` lever) are separate later briefs.

## Expected, measurable result (the gate)

- Correctness preserved: `cargo test --workspace` green, and the `DumpSave` /
  `Get` snapshot round-trip unchanged (`Arc<Vec<Expr>>` and `Arc<[Expr]>`
  serialize to the identical sequence wire format).
- The primitive microbench tier does NOT regress (most labels COW-reuse args
  after A2, so expect flat-to-slightly-better; `downvalue-pattern-hit`, which
  rebuilds nodes each iteration, may improve).
- **Bool-stack load** (the node-construction-heavy downstream signal) improves
  or holds versus the post-A2 baseline `0.70s`.

The win does NOT show in the `shared_args_alloc` counter (that counts
`SharedArgs::new` calls, still ~1 per construction); it is the eliminated hidden
`Vec`-buffer allocation + contiguity. Measure it via the Bool-stack load and the
tier ratio, not the counter.

## Honest trade-off (state it in your report)

`into_vec` can no longer unique-unwrap an `Arc<[Expr]>` into a `Vec` (a slice
cannot be moved out of an `Arc`); it must copy (`to_vec`). After A2 the hot tier
shows `into_vec_total = 0`, so this copy lands only on the ~18 cold builtin
callers of `into_vec` (ReplaceAll/Cases/Delete/etc.), which mostly clone already
because their args are shared. Confirm the tier does not regress; if any hot
label regresses because of this, STOP and report.

## Context pointers

- All changes are in `crates/rum-core/src/expr.rs`, the `SharedArgs` block
  (declaration line ~496 and its impls through ~580). Read it fully first.
- `SharedArgs(Arc<Vec<Expr>>)` derives `Clone, Debug, Eq, Hash, Serialize,
  Deserialize`. Keep all of those (they all hold for `Arc<[Expr]>`).
- Audit facts you are relying on: nothing mutates a `SharedArgs` in place
  (no `push`/`extend`/resize through `DerefMut`); `cargo build --workspace` is
  the proof — if a hidden mutator exists it will fail to compile, and you must
  STOP and report rather than re-introduce a `Vec` backing.

## Scope

- Files you MAY modify:
  - `crates/rum-core/src/expr.rs` (the `SharedArgs` backing + its impls),
  - any site that fails to compile ONLY because the `Deref` target changed from
    `Vec<Expr>` to `[Expr]` — fix it by using `&[Expr]` / `as_ref()` /
    `.to_vec()` (NOT by reverting the backing). List every such site in your
    report; if there are more than ~6, STOP and report first.
- MUST NOT change: the `Expr::Call { head: SharedExpr, args: SharedArgs }`
  variant shape, `SharedExpr`, the `Expr` public API, the serde wire format, any
  `mma-hol` clone, `.git`.

## Design (implement exactly; idiomatic Rust is yours)

1. `pub struct SharedArgs(Arc<[Expr]>);` (keep the same derives).
2. `new(args: Vec<Expr>) -> Self`: `let shared = Self(Arc::from(args));
   record_shared_args_alloc(); shared` (`Arc::<[Expr]>::from(Vec<Expr>)`).
3. `into_vec(self) -> Vec<Expr>`: `self.0.to_vec()` — it always copies now;
   call `record_into_vec(true)` (it can no longer free-unwrap). Keep the
   signature.
4. `ptr_eq`: `Arc::ptr_eq(&lhs.0, &rhs.0)` (unchanged). `ptr_key(&self) ->
   usize`: the `Arc<[Expr]>` pointer is a fat pointer — return the data address,
   e.g. `Arc::as_ptr(&self.0) as *const Expr as usize` (or `.cast::<u8>() as
   usize`). It must stay a stable per-allocation key.
5. `Default`: `Self(Arc::from(Vec::new()))` (an empty `Arc<[Expr]>`).
6. `Deref`: `type Target = [Expr]; fn deref(&self) -> &[Expr] { &self.0 }`.
7. **Remove the `DerefMut` impl** (`Arc::make_mut` requires `T: Clone`, and
   `[Expr]` is unsized so it does not apply). If anything fails to compile
   because it relied on `DerefMut`, that is a hidden in-place mutator — STOP and
   report it rather than working around it.
8. `AsRef<[Expr]>`: `&self.0`. `From<Vec<Expr>>`: `Self::new(args)`.
   `PartialEq`: `Self::ptr_eq(self, other) || self.as_ref() == other.as_ref()`
   (unchanged). `IntoIterator for SharedArgs`: `self.into_vec().into_iter()`
   (unchanged); `&SharedArgs`: `self.iter()` (unchanged).

## Verification (MANDATORY — run and paste verbatim)

```
cargo fmt --check
cargo test --workspace
```

All green (the whole workspace — this swap touches a shared type). Then the
serde / snapshot round-trip specifically (find and run the existing DumpSave/Get
or serde round-trip test, e.g. `cargo test -p rum-runtime dump_save` /
`cargo test -p rum-core serde` — name the actual test you ran). Then the tier:

```
bash scripts/core-language-microbench.sh --engine both --output-dir /tmp/cdx_brief_012_micro
```

Paste the `ratios.tsv` rows for `if-true`, `downvalue-exact-hit`,
`downvalue-pattern-hit`, `sameq-integer`, `length-list`, `head-list`. Then the
Bool-stack load (acceptance clone, internal AbsoluteTiming):

```
target/release/rum-cli --eval '$Path=Prepend[$Path, "/Users/fft/Developments/mma-hol-acceptance"]; AbsoluteTiming[Get["ErrorUtil.wl"]; Get["Types.wl"]; Get["Terms.wl"]; Get["Kernel.wl"]; Get["Bootstrap.wl"]; Get["Equal.wl"]; Get["Bool.wl"];][[1]]'
```

Report the load time (baseline `0.70s`). The mma-hol clone is read-only; do not
modify it.

## rum / Rust pitfalls (read twice)

1. `Arc::make_mut` / `try_unwrap` do not work on `Arc<[Expr]>`; do not try to
   keep them. Remove `DerefMut`; `into_vec` copies.
2. The `Deref` target changes `Vec<Expr>` → `[Expr]`. Fix compile fallout by
   moving to slices, never by cloning into a `Vec` on a hot path.
3. Keep the serde derive and the wire format identical — gate on the snapshot
   round-trip.
4. Measure the RELEASE binary, profiling OFF, for the ratio and the load.

## Hard rules

- Do NOT `git commit`/branch/push; leave the working tree.
- Only the listed files; nothing outside repo + `/tmp`; no network. Commands:
  `cargo`, release `rum-cli`, the microbench script, `python3`. Remove any
  `.rum_history` you create.
- Never edit any `mma-hol` file.
- Stop-loss: same `cargo test` failure twice, OR a hidden in-place mutator
  surfaces (DerefMut removal breaks something), OR >~6 compile-fallout sites, OR
  a hot-tier label regresses → STOP and report.
- Minimal diff. Paste real `cargo test --workspace` summary, the serde test, the
  ratio rows, and the Bool-load time; never fake a result.

## Report format

1. Per-file changes, line ranges + one-line why; list every compile-fallout site.
2. The serde-compatibility argument + which round-trip test you ran.
3. Verification — verbatim `cargo fmt --check`, `cargo test --workspace`
   summary, the serde test, the ratio rows, and the Bool-load time.
4. The into_vec trade-off note (did any hot label regress?).
5. Stop-loss / open questions (empty if none).
