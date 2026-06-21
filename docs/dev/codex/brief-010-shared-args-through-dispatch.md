# Brief 010 — Layer A1: thread `SharedArgs` through call dispatch (implementation)

## Goal

Layer A1 of the representation rewrite (`docs/representation-rewrite-plan.md`):
stop eagerly materializing owned call args. Today `evaluate_inner_unstable`
does `self.evaluate_call(head.into_owned(), args.into_vec())` BEFORE dispatch
knows whether the call is a held special form, a builtin, or a definition — so
**every** call deep-clones its args vector up front. The Phase-0 counters prove
it: `if-true` (a pure held special form that never rebuilds its node) shows
`into_vec_clone = 1.00M` per 1M calls. A held / special-form call should read
its args by borrow and convert nothing.

Thread the args as `SharedArgs` (an `Arc<Vec<Expr>>`, cheap to clone = one
refcount bump) from `evaluate_inner_unstable` through `evaluate_call` /
`resume_call_after_head` / `CallDispatchState`, reading them as `&[Expr]` in the
stages. Only the `PrepareArguments` stage (which genuinely needs owned args to
evaluate them) converts to a `Vec`, and even that conversion is now a free
`Arc::try_unwrap` whenever the args Arc is unique.

**Scope note (read carefully):** this brief is ARGS ONLY. Leave the head as
`head.into_owned()` for now — the head conversion is a cheap no-heap symbol
clone in the hot cases, not the structural cost, and threading the head as
`SharedExpr` is a separate later step. Do not touch it.

## Expected, measurable result (this is the gate)

- `if-true` `into_vec_clone` drops from ~`1.00M` to ~`0` per 1M calls (the held
  special-form path stops converting args), and the `if-true` microbench ratio
  improves.
- `downvalue-exact-hit` / `downvalue-pattern-hit` `into_vec_clone` stay ~`1.00M`
  — they genuinely need owned, evaluated args (that is Layer A2's COW job, not
  this brief). They must NOT regress.
- No behavior change anywhere; all tests green.

If `if-true`'s `into_vec_clone` does NOT fall to near zero, the threading missed
the held path — STOP and report rather than papering over it.

## Context pointers

- Read `docs/representation-rewrite-plan.md` (Layer A) and the Phase-0 baseline
  entry in `PROGRESS.md` first.
- The eager conversion is `crates/rum-runtime/src/runtime/eval_core.rs:61`:
  `self.evaluate_call(head.into_owned(), args.into_vec())` inside
  `evaluate_inner_unstable`'s `Expr::Call { head, args }` arm. Here `args` is
  already a `SharedArgs` and `head` a `SharedExpr`.
- Dispatch flow: `evaluate_call(head: Expr, args: Vec<Expr>)`
  (`eval_core.rs:624`) → `resume_call_after_head(evaluated_head, args)`
  (`eval_core.rs:663`) → `CallDispatchState::new(args)` (`eval_core.rs:672`) →
  `resume_call_with_dispatch_plan`.
- `CallDispatchState` (`crates/rum-runtime/src/runtime/session_state.rs:80`):
  `{ raw_args: Vec<Expr>, evaluated_args: Option<Vec<Expr>> }`. Its ONLY
  `raw_args` consumers are: `into_final_args` (session_state.rs:99-101),
  `eval_dispatch/mod.rs:75` (SpecialForm reads `state.raw_args.as_slice()`),
  and `eval_dispatch/mod.rs:78` (PrepareArguments `std::mem::take(&mut
  state.raw_args)` → `prepare_arguments_for_call(.., raw_args: Vec<Expr>)`).
  The `state.raw_args` / `state.head` hits in `functional_ops/.../fold/*` and
  `debug_tools/test_report/report.rs` are a DIFFERENT struct (it has a `.head`
  field; `CallDispatchState` does not) — do NOT touch them.
- The constructors you need already exist in `crates/rum-core/src/expr.rs`:
  `Expr::call_with_shared_args(head: Expr, args: SharedArgs) -> Expr` (line
  ~648) and `Expr::unwrap_shared_args(args: SharedArgs) -> Vec<Expr>` (line
  ~659, a branch-counted `Arc::try_unwrap` — free when unique, clones only when
  shared). `SharedArgs` derefs to `[Expr]` and impls `AsRef<[Expr]>`.

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/eval_core.rs` (the `evaluate_inner_unstable`
    call arm, `evaluate_call` / `resume_call_after_head` signatures + bodies,
    the final rebuild at line ~695),
  - `crates/rum-runtime/src/runtime/session_state.rs` (`CallDispatchState`
    `raw_args` field + `new` + `into_final_args`),
  - `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs` (the two `raw_args`
    consumers at lines ~75 and ~78),
  - `crates/rum-core/src/expr.rs` ONLY if you add a trivial additive
    `impl Default for SharedArgs` (empty `Arc<Vec>`), needed for `mem::take` /
    `mem::replace` at the PrepareArguments site. Nothing else in rum-core.
- MUST NOT change: dispatch ordering, stage behavior, the `PrepareArguments`
  evaluation/normalization logic, `prepare_arguments_for_call`'s `Vec<Expr>`
  parameter (keep it `Vec` — A2 changes it later), any builtin/matcher, serde,
  `Expr`/`SharedArgs` public behavior, any `mma-hol` clone, `.git`.

## Design (implement exactly; idiomatic Rust is yours)

1. `eval_core.rs:61`: pass the `SharedArgs` straight through —
   `self.evaluate_call(head.into_owned(), args)` (drop `.into_vec()`; keep
   `head.into_owned()`).
2. `evaluate_call(&mut self, head: Expr, args: SharedArgs)`: the
   `InputRequired` suspend closure (lines ~635-638) clones `args` — now a cheap
   `Arc` bump (`let args_for_resume = args.clone();`). Pass `SharedArgs` to
   `resume_call_after_head` in both the suspend closure and the normal path
   (line ~655).
3. `resume_call_after_head(&mut self, evaluated_head: Expr, args: SharedArgs)`:
   unchanged except the type; `CallDispatchState::new(args)` now takes
   `SharedArgs`.
4. `CallDispatchState`: `raw_args: SharedArgs`; `new(raw_args: SharedArgs)`;
   `into_final_args(self) -> Vec<Expr>` returns
   `self.evaluated_args.unwrap_or_else(|| Expr::unwrap_shared_args(self.raw_args))`.
   (Optionally also rebuild the final node at `eval_core.rs:695` via
   `Expr::call_with_shared_args` to avoid the fallback `Vec`→`SharedArgs`
   re-wrap, but that is optional polish — keep it correct and minimal.)
5. `eval_dispatch/mod.rs:75` (SpecialForm): read by borrow —
   `state.raw_args.as_ref()` (or `&state.raw_args[..]`). NO clone, NO take.
6. `eval_dispatch/mod.rs:78` (PrepareArguments): take the shared args out and
   convert to owned for `prepare_arguments_for_call`:
   `let raw_args = Expr::unwrap_shared_args(std::mem::take(&mut state.raw_args));`
   (requires `Default for SharedArgs`) — or `std::mem::replace(&mut
   state.raw_args, SharedArgs::default())`. On the normal non-suspended path the
   args Arc is unique here, so `unwrap_shared_args` is a free unwrap.

**Why this is correct (state it in your report):** the args value is identical;
only the carrier changes from `Vec` to `Arc<Vec>`. Held / special-form calls
return from the SpecialForm stage by borrowing the args (never converting).
`PrepareArguments` still produces the same owned, evaluated `Vec`. The suspend
closure still resumes with the same args (now via a refcount-bumped Arc, which
is `Send`/`Sync`). `into_final_args` yields the same `Vec`.

## Verification (MANDATORY — run and paste verbatim)

```
cargo fmt --check
cargo test -p rum-core
cargo test -p rum-runtime
```

All green. Then PROVE the counter moved (the gate):

```
RUM_PROFILE_EVALUATION_CACHE=1 RUM_PROFILE_REPR=1 target/release/rum-cli --eval \
  'Do[Do[If[True,1,2],{i,1,100000}],{j,1,10}]' 2>&1 | grep 'rum-profile repr'
RUM_PROFILE_EVALUATION_CACHE=1 RUM_PROFILE_REPR=1 target/release/rum-cli --eval \
  'fExact[1]=2; Do[Do[fExact[1],{i,1,100000}],{j,1,10}]' 2>&1 | grep 'rum-profile repr'
RUM_PROFILE_EVALUATION_CACHE=1 RUM_PROFILE_REPR=1 target/release/rum-cli --eval \
  'fPattern[x_Integer]:=x+1; Do[Do[fPattern[1],{i,1,100000}],{j,1,10}]' 2>&1 | grep 'rum-profile repr'
```

Report all three counter lines. The gate: `if-true` `into_vec_clone` ~`1M` →
~`0`; the two downvalue shapes' `into_vec_clone` stay ~`1M`/~`2M` (unchanged).
Then the ratio:

```
bash scripts/core-language-microbench.sh --engine both --output-dir /tmp/cdx_brief_010_micro
```

Paste the `ratios.tsv` rows for `if-true`, `downvalue-exact-hit`,
`downvalue-pattern-hit`, `sameq-integer`, `length-list`, `head-list`. `if-true`
should improve; the others must not regress beyond noise.

## rum / Rust pitfalls (read twice)

1. Do NOT touch the head conversion (`into_owned`) — args only this brief.
2. `mem::take` needs `Default`; add a trivial `impl Default for SharedArgs` in
   rum-core (empty `Arc::new(Vec::new())`) or use `mem::replace`. Do not add any
   other rum-core surface.
3. SpecialForm must read args by BORROW (`as_ref`), never take/clone — that is
   the whole point (it is the held path that should convert nothing).
4. Keep `prepare_arguments_for_call`'s `Vec<Expr>` parameter; the COW version is
   Layer A2, not this brief. Measure the RELEASE binary, profiling OFF for the
   ratio.

## Hard rules

- Do NOT `git commit`/branch/push; leave the working tree.
- Only the listed files; nothing outside repo + `/tmp`; no network. Commands:
  `cargo`, release `rum-cli`, the microbench script, `python3`.
- Never edit any `mma-hol` file. Remove any `.rum_history` you create.
- Stop-loss: same `cargo test` failure twice, OR `if-true` `into_vec_clone`
  does not fall to near zero, OR any downvalue shape regresses → STOP and
  report.
- Minimal diff. Paste real test summaries, the three counter lines, and the
  ratio rows; never fake a result.

## Report format

1. Per-file changes, line ranges + one-line why.
2. The correctness argument (carrier-only change; held path borrows).
3. Verification — verbatim `cargo fmt --check`, `cargo test -p rum-core` and
   `-p rum-runtime` summaries, the three `rum-profile repr` lines, and the ratio
   rows.
4. Stop-loss / open questions (empty if none).
