# brief-056: SAFE iterative `Drop` for deep `Expr` (rework of brief-055's reverted unsafe version)

## Goal

A very deep `Expr` (e.g. a host embedder building `Expr::call(...)` in a Rust loop —
NOT reachable via WL paths, which are depth-capped by the parser guard + eval
`recursion_limit` + resource limits, but reachable via the rum-core Rust API) must
`Drop` WITHOUT a native stack overflow. Implement this in **SAFE Rust (no `unsafe`)**
and with **NO default-path cost** (gated by a microbench A/B per the standing rules in
`TEMPLATE.md` / `AGENTS.md`). brief-055's first attempt is REVERTED — it used
`unsafe ManuallyDrop`/`ptr::drop_in_place` (the first `unsafe` in rum-core) and an
O(N)-per-list-drop depth probe (~10% list-op regression). This redo must avoid BOTH.

The parser depth guard (the WL-path crash) is ALREADY committed; this brief is ONLY
the `Drop` half. `rum-core` and `rum-runtime` are ZERO-`unsafe` today — keep it that
way.

## Design (safe; pick the cleaner of these, measure)

The manual `Drop` must take the `Arc` out of `&mut self` and drain the tree
iteratively (a work-stack, not recursion), descending ONLY into `Arc`-UNIQUE children
(`Arc::try_unwrap` Ok) so shared subtrees are just ref-decremented, never deep-walked.

Two safe ways to extract the `Arc` in `Drop` (NO `unsafe`, NO size change — both keep
`size_of::<SharedExpr>() == 8`):
- **(A) `mem::replace` with a static sentinel** — keep `SharedExpr(Arc<Expr>)`
  UNCHANGED, and in `drop`: `let arc = std::mem::replace(&mut self.0, sentinel())`,
  where `sentinel()` clones a process-`static OnceLock<Arc<Expr>>` holding a shallow
  placeholder (e.g. `Arc::new(Expr::symbol("System`Null"))`). Then drain `arc`. After
  `drop` returns, the field holds the sentinel and auto-drops shallowly. Pro: `Deref`/
  `as_ref` are UNCHANGED (no per-access cost). Con: one atomic refcount bump per drop.
- **(B) `Option<Arc<Expr>>`** — `SharedExpr(Option<Arc<Expr>>)` (niche-packs to 8 B);
  `drop` does `self.0.take()`. Pro: no sentinel, drop is a plain `.take()`. Con:
  `Deref`/`as_ref`/constructors must `.expect("always Some outside Drop")` — a
  predictable branch on the hot access path.

Derefs are far more frequent than drops, so (A) is likely the better perf trade —
but DECIDE BY THE MICROBENCH, not by argument. No depth probe / no per-drop subtree
scan in EITHER design: the iterative drain itself handles any depth, and the common
shallow case is just `try_unwrap` + a tiny work-stack loop. Do the same for
`SharedArgs(Arc<Vec<Expr>>)`.

Drain sketch (safe): pop an owned `Expr` from the work stack; if `Call { head, args }`,
`try_unwrap` head's `Arc` (Ok → push the inner `Expr`; Err → it's shared, dropping the
`Arc` is cheap) and `try_unwrap` args' `Arc` (Ok → extend the work stack with the
`Vec<Expr>`; Err → drop). `Atom` → just drop. The owned `Expr`'s own fields must not be
recursively dropped by the language after you've moved their `Arc`s out — with a plain
owned `Expr` and `mem::replace`/`Option::take` on the `Shared*` fields, moving the
`Arc` out leaves the `Shared*` holding the sentinel/`None`, which drops shallowly. No
`ManuallyDrop`, no `ptr::drop_in_place`.

## Scope

- MAY modify: `crates/rum-core/src/expr.rs` (the `Shared*` `Drop` + the drain helpers +
  a `rum-core` deep-drop test). Do NOT change `Atom`/`Symbol`/`Expr` enum shapes or the
  parser (already done).
- MUST NOT: introduce ANY `unsafe`; add a per-access or per-drop O(N)/subtree cost;
  touch `mma-hol`, `AGENTS.md`/`PROGRESS.md`/`TODO.md`/`docs/**` except your report;
  `.git`, `Cargo.lock`.

## Verification (MANDATORY — run, paste verbatim)

```
cargo fmt --check
grep -rn unsafe crates/rum-core/src crates/rum-runtime/src   # MUST print nothing
cargo test -p rum-core            # incl. a NEW test: build a 100_000-deep Expr via Expr::call, drop it, no overflow
cargo test --workspace
```
Confirm `size_of::<SharedExpr>()` and `::<SharedArgs>()` are unchanged (8 each) via a
throwaway test (then delete it).

**Microbench A/B (REQUIRED — this is the gate the first attempt lacked).** Build BEFORE
(`git stash push -- crates/`) and AFTER release binaries to `/tmp/rum_base` /
`/tmp/rum_new`, then:
```
bash scripts/core-language-microbench.sh --engine rum --rum-cli /tmp/rum_base --output-dir /tmp/cdx_056_base
bash scripts/core-language-microbench.sh --engine rum --rum-cli /tmp/rum_new  --output-dir /tmp/cdx_056_new
```
Paste the per-label seconds (col 4 of `measurements.tsv`) base-vs-new with ratios,
ESPECIALLY the list/structure labels (`length-list`, `part-list`, `head-list`,
`sameq-list`, `do-small`, `table-small`, `module-local`). NO hot label may regress
beyond noise (~±3%); a real list-op regression like the first attempt's ~10% is a
defect to fix before reporting. If you cannot get it neutral, STOP and report the
approach + numbers.

## Hard rules

- Do NOT `git commit`/branch/push. Leave changes in the working tree for Claude's gate.
- **NO `unsafe`** anywhere in this change (the whole point of the rework). If you think
  you need it, STOP and report — you do not.
- Hot-path change → the microbench A/B above is mandatory, not optional.
- Stop-loss: if you cannot reach safe + neutral-perf after two approaches, STOP and
  report both with numbers (do not ship unsafe or a regression).
- Paste REAL verbatim output (incl. the `grep unsafe` empty result, the size check, and
  the microbench ratios); never fabricate.

## Report format

1. What changed — which design (A or B) and why; the drain structure; the test added.
2. Verification — every command + verbatim output: `grep unsafe` (empty), test result
   lines, the deep-drop test, `size_of` lines, and the microbench base-vs-new ratios
   for the list/structure labels.
3. The verdict — deep `Expr` drops without overflow? zero `unsafe`? size unchanged?
   microbench neutral (paste the worst-case hot-label ratio)?
4. Stop-loss / open questions (empty if none).
