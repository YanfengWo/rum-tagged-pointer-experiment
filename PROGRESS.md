# Progress Log

This file records completed work, archived planning detail, and operational
lessons that are useful history but should not keep cluttering `TODO.md`.

`TODO.md` should stay focused on active or near-term work. When a completed
checklist stops helping prioritization, move or summarize it here.

**Archive note (2026-07-03):** the previous log (6386 lines, everything through
the long-tail/capability-sandbox/release-prep era) moved to
`docs/PROGRESS-archive-2026-07-03.md`. State at archive time: correctness
mainline complete (README 3126/0 reproduced exactly); host-embedding hardening
arc done (Tier-1 + Tier-2 core + rum-jupyter real interrupt); long-tail builtin
batches done and the support matrix accuracy-passed (brief-094); capability
sandbox landed (brief-095); public-release scrub method recorded and
`release-staging/` deleted pending re-creation (brief-096); release DEFERRED in
favor of code cleanliness. Active line: crate rebalancing, route 2a —
`docs/crate-rebalance-plan.md`.

## 2026-07-04 Stage-3 batch 2 DONE (brief-103) — integer_digits + number_theory moved; 3 families out

First BATCH move (two families in one brief) through the proven pipeline:
`rum-stdlib` now owns `integer_digits` (5 handlers) and `number_theory` (9
handlers, incl. the Prime/PrimePi small tables and Pollard-rho path); runtime
catalog 502 → 488, stdlib catalog 25 (11+5+9); runtime greps clean; CLI smoke
`--eval "NextPrime[100] + FromDigits[{1,2,3}] + EulerPhi[10]"` → 228; workspace
1299/0. Promotions: `Session::{invalid_call_with_arg_range,
enforce_generated_element_limit_for_count, thread_call_args, push_message}`
now `pub` (+ `evaluate` documented); `kernel_api` re-exports
`num_integer::{Integer,Roots}` + `num_traits::{One,Signed,Zero}` additions.

Two process notes. (1) The NEW test-migration rule proved out: `number_theory`
tests moved as a whole file (8 fns), while `integer_digits` tests were embedded
in `tests/arithmetic.rs` — the rule "run the runtime suite after the move; every
MEASURED failing function moves whole to the stdlib" migrated exactly 3
functions, no guesswork. (2) The first dispatch of this brief was killed
mid-move by my own foreground-chained dispatch command (Bash tool 2-min
timeout); the half-done tree was rolled back cleanly and re-dispatched. That
failure mode is now MECHANICALLY blocked by a PreToolUse hook
(`.claude/hooks/guard-dispatch.py`): any `dispatch-brief`/`cdx exec` Bash call
that is chained, bare-`&` detached, or not run_in_background is denied by the
harness before execution (DRY_RUN=1 exempt) — verified live.

## 2026-07-04 Stage 3 OPENED and the MOVE pipeline is PROVEN (brief-102) — combinatorics lives in rum-stdlib

The first family physically left the monolith. `rum-stdlib` now owns the
combinatorics implementation (`src/combinatorics.rs`, 11 handlers converted to
free fns while moving, bodies verbatim), its specs (`src/catalog.rs`, 11
entries byte-identical to the deleted runtime ones modulo handler path), and
its 5 behavioral tests; `install_core` = runtime catalog (now 502 entries)
then stdlib specs. rum-runtime greps CLEAN for combinatorics. Pilot family was
combinatorics (not logic) because it has zero incidental test usage — logic
owns the comparison operators the kernel's own tests use everywhere.

Step-0 resolution probe (the one real unknown) PASSED: a builtin registered
under a name ABSENT from the static catalog resolves and dispatches through
`eval_text` — no vocabulary fix needed; the probe stays as a regression test
(`registered_builtin_resolution.rs`). Kernel promotions were restrained and
documented: `Session::{invalid_call_with_exact_arg_count,
combine_additive_args, combine_multiplicative_args}` now `pub`;
`kernel_api` re-exports `BuiltinSpec` + `num_traits::ToPrimitive` (spares the
stdlib a num-traits dep). rum-stdlib gained the `rum-core` dep.

Gate: fmt + workspace 1299/0 (runtime −5 moved tests +1 probe, stdlib +5);
CLI smoke `--eval "Fibonacci[10] + CatalanNumber[4]"` → 69 (the brief's
literal `eval` subcommand syntax was wrong — rum-cli takes `--eval`; noted for
future briefs). No microbench (registration source only). Remaining Stage-3
work: the other 12 families through the same pipeline; load-bearing ones
(logic/predicates/arithmetic/…) additionally need the test-migration /
dev-dep-cycle decision recorded in the memory notes.

## 2026-07-03 Stage-2 step 3 DONE (brief-101) — logic handlers are free functions

The pilot family's handlers no longer live in `impl Session`: 16 handlers
(truth.rs 8 + comparison.rs 8, incl. the private `eval_comparison_subset` and
the truth continuation helpers) are free functions
`fn eval_x(session: &mut Session, …)` with bodies VERBATIM (receiver-only
change); the facade re-exports them per-item; `builtin_catalog/logic.rs` and
the nand/nor adapters reference `crate::runtime::logic::eval_*` explicitly.
Scope note that shaped this brief: catalog file grouping ≠ implementation
module grouping (catalog/logic.rs references 69 handlers; the logic MODULE
owns 16 — the rest are predicates/structure_ops and stay untouched until their
family's brief). Gate: fmt + workspace 1298/0; zero `impl Session` under
logic*; no microbench (static call → static call, dispatch fn-pointer shape
unchanged). The logic family is now move-ready: brief-102 = the physical move
pilot (files → the stdlib crate, spec ownership, kernel Session-method
pub-ification, test migration).

## 2026-07-03 Stage-2 step 2 DONE (brief-100) — registration is the ONLY builtin dispatch path

Discovery that motivated it: dispatch resolution silently fell back to a static
`OnceLock` map over `core_builtin_specs()`, so builtins never needed
registration and `rum-stdlib::install_core` was decorative — families cannot
leave the crate while the kernel statically consults its own catalog. Now:
`resolution.rs` reads ONLY the session's registered map (fallback deleted);
`Session::default()`/`new()` auto-install `core_builtin_specs()` (the
transitional bridge — shrinks as families move out); `with_config()` builds a
truly RAW session (hosts/stdlib must install — they already do);
`register_builtin` applies the `with_fallback_dispatch` merge against the
previously REGISTERED spec, preserving user-override semantics; `Names`
enumerates registered specs. The static `default_builtin_dispatch_spec*`
helpers legitimately REMAIN as vocabulary (known-system-symbol predicates in
`expr_runtime_helpers.rs` — a dispatch-vs-vocabulary distinction the original
brief's grep gate missed; Codex stop-lossed on it and the gate was amended).
New test: fresh session evaluates a builtin + DumpSave/Get restore keeps the
registered set. Gate: fmt + workspace 1298/0; resolution grep = 0; microbench
A/B flat-to-better (−8%…+1.6%, no consistent regression; mechanism touches
only the cache-miss resolution path).

## 2026-07-03 Stage-2 step 1 DONE (brief-099) — kernel_api v1 exists, logic family de-globbed

`runtime/kernel_api.rs` is now the single public door to the kernel's helper
surface: 40 audited fan-out ≥2 items re-exported in 14 grouped `pub use` lines
(`pub mod kernel_api` from `runtime/mod.rs`; defining `mod`s stay private, so
the promoted `pub` items are unreachable except via kernel_api). Pilot: the
`logic` family (`logic.rs` + `logic/{comparison,shared,truth}.rs`) carries NO
`use super::*` — only explicit `kernel_api`/`rum_core` imports. The pilot
CONFIRMED the audit model: demanded exactly the predicted 9 items (5 kernel_api
+ 4 rum_core), zero unplanned promotions.

Learned (now standing for briefs 100+): promoted items sit in private
submodules behind family facades; Rust cannot re-export a restricted re-export
(E0364/E0365), so facades need per-item `pub use self::<submod>::item;` lines
(never widen a glob re-export) — facade files belong in MAY-modify from the
start. Codex stop-lossed correctly on this and was resumed with a scoped
authorization. Gate: fmt + `cargo test --workspace` 1297/0; visibility/imports
only (rustfmt reflow aside, no body/signature changes; no microbench — no
codegen delta); the `SymbolExprMap`-vs-private-hasher warning closed at gate
time by making `RuntimeCacheHasher` `pub` inside its private mod (one word +
comment). Diff: 21 files, ~+56/−51.

## 2026-07-03 Stage-2 audit COMPLETE (brief-098) — the real kernel API is tiny

Method: per-family compile-error audit (comment out `use super::*` in each of the
14 builtin family modules, one at a time; collect unresolved names from
`cargo check`; restore) + a grep axis for kernel `Session` methods called from
family code. Full artifact: `docs/dev/kernel-api-audit-2026-07-03.md` (from
`/tmp/cdx_brief_098_report.md`); raw logs `/tmp/cdx_brief_098_raw_<family>.log`.
Diagnostic only — tree clean, final `cargo check` green. (Operational note: the
first run stopped at 6/14 families because Claude's concurrent PROGRESS-archive
commit tripped the whole-tree clean-check; resumed with the check scoped to
`crates/`. Lesson recorded: no repo mutation while a brief is in flight.)

Findings (free-item axis): of the runtime-internal namespace, only **258 items
are used by any family at all**; **24 have fan-out ≥3 families** (the prelude
core: `Session`/`EvalError` types, `system_expr`, `is_system_symbol_name`, the
`expr_as_*` coercion helpers, `compare_expr_canonical`, a few model/session
types like `ThreadingDispatch`/`ThreadArgSpec`/`CasesLevelSpec`), 16 have
fan-out 2, **218 are single-family** (move WITH their family, don't promote),
and ~600 internal symbols are used by no family (stay kernel-private). Known
family→family couplings that constrain move order: `arithmetic` numeric
helpers (`expr_as_exact_integer` 7 families, `expr_as_exact_rational` 5),
`associations/data` helpers (`association_entries_from_expr`,
`render_association` — 6 families each), structure path helpers.

Findings (Session-method axis, the future pub method surface, top of ~15-25):
`evaluate` (148 calls), the `invalid_call_with_*` argument-error helpers
(131+113+112+67), `suspend_with_frame` (100) + `drive_internal_value` (41) +
`suspend_with_result_frame` (17), `push_message` (37), the `enforce_*` resource
limits, `match_expr_owned`, `merged_option_exprs_for_system_symbol`.

Cheapest first move: **`logic`** — 9 free items, no same-family glob fallout, no
other-family coupling. Count reconciliation note: the earlier "528
`pub(in crate::runtime)` symbols" was a grep line count; the audit's item-name
scan counts 651 unique names — same order of magnitude, the design conclusion
(promote ~40, not 500+) is insensitive to the method.

## 2026-07-03 Crate-rebalance line OPENED (route 2a) — Stage 1 DONE: builtin dispatch is open

Line context: `rum-runtime` had grown into a 116,293-line / 391-file monolith
(evaluator + every builtin family) while `rum-stdlib` was a 21-line installer
facade; builtins were unmovable because dispatch was a CLOSED ~470-variant
`BuiltinHandler` enum + a three-file chained `match` + an exhaustive per-variant
capability `match`, and the families compile against a 528-symbol
`pub(in crate::runtime)` shared namespace via `use super::*` (377 files).
Canonical plan: `docs/crate-rebalance-plan.md` (d49e101) — Stage 1 open the
dispatch, Stage 2 define the kernel API, Stage 3 move the families; file-slimming
deliberately AFTER the split.

**Stage 1 (brief-097, commit 45d2e8a): delete the closed enum.** `BuiltinSpec`
now carries the handler as a plain fn pointer
(`pub type BuiltinFn = fn(&mut Session, &Expr, &[Expr]) -> Result<Expr, EvalError>`)
and the capability as data (`capability: Capability`, default `Pure`; 8 FileRead
+ 3 FileWrite annotated in the catalog). Deleted: the `BuiltinHandler` enum, the
`eval_dispatch/{surface,symbolic,control_functional}.rs` match chain (~530
lines), and the `impl BuiltinHandler::capability` match (~500 lines); net −1467
lines. The 513 catalog entries reference `Session::eval_*` directly (fn-item →
fn-pointer coercion in const arrays); ~25 non-standard arms (args-only
signatures + inline bodies) live VERBATIM as adapters in `builtin_adapters.rs`
— `Evaluate` keeps its `suspend_with_frame` suspension shape. `CallDispatchPlan`
carries the resolved `BuiltinSpec` (Copy, resolve-once-at-plan-build preserved);
capability inheritance follows handler inheritance in `with_fallback_dispatch`
(a re-registered `Get` cannot shed its `FileRead` requirement). The
fn-pointer-comparison lint is allowed locally at the type (all spec copies
originate from one catalog entry, so pointer equality is stable).

Gate evidence: `cargo fmt --check` + `cargo test --workspace` green (1297/0);
`grep -rn "BuiltinHandler" crates/` = 0 hits; microbench A/B run TWICE
independently (Codex once, Claude reproduced via a HEAD worktree). KEY
measurement lesson: the two A/Bs disagreed label-by-label IN BOTH DIRECTIONS
(labels reported −21% in one run showed +9–11% in the other, and vice versa) —
sub-ms dispatch-tier deltas are noise-dominated; judge only cross-run
consistent-direction movement, of which there was none. Verdict: match-jump →
indirect call is measurement-neutral; the extraction precondition (builtin =
data registered into the session) now holds.

**Stage 2 audit dispatched (brief-098, e251618):** diagnostic, no source change
— per-family compile-error-driven audit (comment out `use super::*`, collect
unresolved names, restore) over 14 family modules + a grep axis for kernel
`Session` methods called from family code. Output: the promotion-candidate list
for the explicit kernel API (≥3-family items → prelude; single-family items →
move with the family; unused internals → stay private) + the family→family
couplings that constrain move order.
