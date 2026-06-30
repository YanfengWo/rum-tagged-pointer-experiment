# brief-057: safe iterative `Drop` — DESIGN A ONLY (Arc field unchanged + sentinel), measure the perf

## Goal

Re-attempt the safe iterative `Drop` from brief-056, but with **DESIGN A specifically**
— brief-056's design B (`Option<Arc>`) regressed list/dispatch microbench labels
8-44% because `Option<Arc>` adds a branch to the hot `Deref` (per-ACCESS cost). Design
A keeps the `Arc` field UNCHANGED so `Deref`/`as_ref` are untouched (zero per-access
cost); the only added cost is per-DROP. MEASURE whether A is neutral. Same hard
constraints: **NO `unsafe`** (rum-core/rum-runtime are zero-`unsafe`), and the
**microbench A/B is the gate** — if A still regresses a hot label beyond ~±3%, do NOT
ship it; report the numbers and STOP (we will then conclude the iterative Drop is not
worth it and document the limitation instead).

This is a measured spike: do NOT `git commit`. The deliverable is the safe-A code +
the verdict numbers. The parser depth guard is already committed; this is ONLY the
`Drop`.

## Design A (pin exactly)

Keep `SharedExpr(Arc<Expr>)` and `SharedArgs(Arc<Vec<Expr>>)` field types UNCHANGED
(so `Deref`/`AsRef`/`Display`/`PartialEq` are untouched). Add manual `Drop` impls that
extract the `Arc` via `std::mem::replace` with a shallow process-`static` sentinel,
then drain iteratively:

- `fn sentinel_expr() -> Arc<Expr>`: `static OnceLock<Arc<Expr>>` holding
  `Arc::new(Expr::Atom(Atom::Symbol(Symbol::system("Null"))))`; return `.clone()`.
  Same idea for `sentinel_args()` (`Arc::new(Vec::new())`).
- `impl Drop for SharedExpr { fn drop(&mut self) { let arc =
  std::mem::replace(&mut self.0, sentinel_expr()); drain_expr_tree(arc); } }` — after
  `drop` returns, the field holds the sentinel and auto-drops shallowly. Same for
  `SharedArgs`.
- `drain_expr_tree(root: Arc<Expr>)` / `drain_args_tree`: a work-stack of OWNED `Expr`.
  `try_unwrap` the root; if Ok push the owned `Expr`, else (shared) just drop the
  clone (cheap, no descent). Loop: pop an owned `Expr`; if `Call { head, args }`,
  extract each child's `Arc` via `std::mem::replace(&mut head.0, sentinel_expr())` /
  `…args.0, sentinel_args()` (so the child wrappers, now holding sentinels, drop
  shallowly when they go out of scope — NOT recursing), then `try_unwrap` each
  extracted `Arc` and push the inner `Expr` / `extend` the inner `Vec<Expr>` onto the
  stack; `Atom` drops in place. This is fully safe (`mem::replace`, `try_unwrap`,
  ownership) — NO `unsafe`.
- **Fix the move-out-of-Drop constraint:** adding `Drop` to `SharedExpr` makes
  `into_owned`/`into_vec`'s `Arc::try_unwrap(self.0)` illegal (cannot move a field out
  of a `Drop` type). Rewrite them as `let arc = std::mem::replace(&mut self.0,
  sentinel_*());` (take `mut self`), then `Arc::try_unwrap(arc)`; `self` then drops its
  sentinel shallowly. Keep the `record_*` counter calls intact. Adjust `Default for
  SharedArgs` if needed (it constructs directly — fine).
- Optional cheap shallow fast-path IF (and only if) the plain version regresses: in
  `drop`, if `Arc::strong_count(&self.0) > 1` (shared — the common reuse case), do
  `drop(std::mem::replace(&mut self.0, sentinel_*()))` directly (ref-decrement, no
  drain) — a shared Arc cannot be the sole owner of a deep unique chain that needs
  iterative teardown. Try the plain version first; add this only if the microbench
  needs it.

## Scope

- MAY modify: `crates/rum-core/src/expr.rs` only (the two `Drop`s, the drain helpers,
  the `into_owned`/`into_vec` adjustment, a deep-drop test). NO enum/`Atom`/`Symbol`
  shape change; NO parser change (committed).
- MUST NOT: any `unsafe`; any per-access cost (the field stays `Arc<…>`, `Deref`
  unchanged); touch `mma-hol`, `AGENTS.md`/`PROGRESS.md`/`TODO.md`/`docs/**` except
  your report, `.git`, `Cargo.lock`.

## Verification (MANDATORY — run, paste verbatim)

```
cargo fmt --check
grep -rn unsafe crates/rum-core/src crates/rum-runtime/src    # MUST be empty
cargo test -p rum-core      # incl. a NEW test: 100_000-deep Expr via Expr::call, drop, no overflow
cargo test --workspace
```
Throwaway `size_of::<SharedExpr>()/<SharedArgs>()` test (expect 8/8), then delete.

**Microbench A/B (the gate).** Build BEFORE (`git stash push -- crates/`) + AFTER to
`/tmp/rum_base` / `/tmp/rum_new`; run `scripts/core-language-microbench.sh --engine rum
--rum-cli …` for each; paste the per-label SECONDS (col 4 of `measurements.tsv`)
base-vs-new with ratios, ESPECIALLY `length-list`, `part-list`, `head-list`,
`sameq-list`, `do-small`, `table-small`, `module-local`, `downvalue-exact-miss-repeat`.
**Verdict rule:** if the worst hot-label ratio is ≤ ~1.03, A is neutral → report
"A NEUTRAL, shippable". If any hot label regresses > ~1.03, report "A REGRESSES" with
the worst ratio and STOP (do not try to force it green by weakening anything).

## Hard rules

- Do NOT `git commit`/branch/push. Leave in the working tree.
- NO `unsafe`. If you think you need it, STOP and report — you do not (design A is
  fully safe).
- The microbench A/B is mandatory; an unmeasured "no cost" claim is not acceptable.
- Stop-loss: if A regresses after trying the optional `strong_count` fast-path, STOP
  and report the numbers — do NOT ship a regression.
- Paste REAL verbatim output (the empty `grep unsafe`, the deep-drop test, the size
  check, the microbench ratios); never fabricate.

## Report format

1. What changed — the `Drop`/drain code; how `into_owned`/`into_vec` were adjusted;
   whether the `strong_count` fast-path was needed.
2. Verification — every command + verbatim output.
3. The verdict — "A NEUTRAL (worst ratio X)" and shippable, OR "A REGRESSES (worst
   ratio X)" → recommend documenting the limitation instead.
4. Stop-loss / open questions.
