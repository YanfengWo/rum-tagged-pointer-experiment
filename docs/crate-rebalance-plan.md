# Crate rebalance plan — route 2a (full builtin extraction)

Status: **COMPLETE (2026-07-04)** — executed as briefs 097-113 (plus the follow-on
file-slimming line, briefs 114-121, and the Diagnostic-capability/test-consolidation
codas 122/123). End state: all 14 builtin families' spec ownership lives in
`rum-stdlib` (catalog 345 specs, per-family files); `rum-runtime` is the pure kernel
(65k lines vs the original 116k monolith; residual catalogs = evaluator semantics:
definition/rule machinery + the Part family, `Function`/`HoldForm`, scoping,
namespace, diagnostics); two solution shapes emerged (full implementation move vs
spec-ownership move with kernel-resident wrappers — used for control_flow/
rewrite_builtins/arithmetic); boundary laws recorded: the Part machinery, shared data
layers (associations/text rendering), and STATIC ATTRIBUTE KNOWLEDGE stay
kernel-side (the last learned from the brief-113 14x fast-path regression, fixed by
the kernel attribute vocabulary + bidirectional drift test). Each batch was gated
with `cargo fmt --check` + `cargo test --workspace` (never a narrow `-p` gate) and —
for hot-path touches — a `scripts/core-language-microbench.sh` A/B. Original plan
below for the record.

## The problem (measured 2026-07-03, dev repo, shipping crates)

- `rum-runtime` = **116,293 lines / 391 files** — a monolith holding the evaluator
  AND every builtin family (strings, structure ops, functional ops, associations,
  polynomials, logic, predicates, arithmetic, number theory, combinatorics, calculus,
  integer digits, control flow, …).
- `rum-stdlib` = **21 lines** — a misnamed installer facade (`install_core` just
  registers `core_builtin_specs()`); the real "stdlib" lives inside `rum-runtime`.
- Everything else is reasonably sized: rum-jupyter 13,412 / rum-syntax 7,813 /
  rum-host 2,267 / rum-core 1,234 / rum-repl 907 / rum-cli 346 / rum-terminal 295.

Why builtins cannot simply be moved (verified against the code):

1. **Dispatch is a CLOSED enum.** `BuiltinHandler` (`runtime/metadata.rs`, ~470
   variants) + a three-file chained `match`
   (`eval_dispatch/{surface,symbolic,control_functional}.rs`, ~530 lines) + an
   exhaustive per-variant capability `match` (`runtime/capabilities.rs`, ~500 lines).
   Adding a builtin means editing kernel dispatch tables; hosting one outside the
   crate is impossible.
2. **A shared internal namespace.** 528 `pub(in crate::runtime)` symbols; 377 files
   `use super::*`; every builtin module globs the whole runtime namespace.
3. **Builtins are `Session` methods** (`self.eval_plus(head, args)`), so they cannot
   be inherent methods from another crate.

## The stages (bottom-up dependency order)

### Stage 1 — open the dispatch (brief-097)

Delete the `BuiltinHandler` enum. `BuiltinSpec` carries the handler as a plain
fn pointer (`pub type BuiltinFn = fn(&mut Session, &Expr, &[Expr]) ->
Result<Expr, EvalError>`) and the capability as data (`capability: Capability`
field, default `Pure`). The three match files and the capability match are deleted;
the 513 catalog entries reference `Session::eval_*` directly (fn-item → fn-pointer
coercion, valid in const arrays). Fully internal to `rum-runtime`; zero behavior
change; microbench-gated (the innermost dispatch changes from a match jump to an
indirect call).

After this stage a builtin is *data registered into the session*
(`register_builtin(BuiltinSpec)`), not a kernel enum case — the extraction
precondition.

### Stage 2 — define the kernel API (brief-098+)

Audit which of the 528 `pub(in crate::runtime)` symbols the builtin families
actually use (script the audit; do not blanket-`pub` all 528). Promote that subset
into an explicit, documented kernel API surface (likely a `kernel_api` module
re-exporting the promoted items), then convert the builtin family modules from
`use super::*` to explicit imports of that surface. The kernel API is the contract
the moved crate will compile against; getting it small is the point.

Also in this stage: convert builtin `Session` methods to free functions
`fn eval_x(session: &mut Session, head: &Expr, args: &[Expr])` (family by family —
Stage 1's fn-pointer signature already matches the free-function shape).

### Stage 3 — move the families (one brief per family)

Move each family into the real stdlib crate and register its specs from there;
`rum-stdlib` stops being a facade (keep the name — it becomes honest). Candidate
order: start with the least kernel-coupled family (strings is the current guess;
decide from the Stage-2 audit numbers). `rum-runtime` ends as the pure kernel:
eval_core, eval_dispatch, definition_application, binding_substitution,
pattern_matching, scoping, model, evaluation_cache, session_*.

## Deferred / open

- **File-slimming is AFTER the split** (7 files >35k chars incl. `profiling.rs`
  138KB; convention `docs/code-organization-guidelines.md`, unit = characters).
  Slimming a file about to move is wasted effort.
- **Open sub-decisions** (settle during Stage 2/3 design): fate of `profiling.rs`,
  `vm_rhs_template.rs`, `debug_tools/` — stay-in-kernel vs feature-gate vs move-out.
  They instrument/extend the evaluator, so they are kernel-coupled either way.

## Per-brief gate (Claude runs it; unchanged from CLAUDE.md)

1. Scope check (`git diff` vs the brief's MAY-modify list).
2. `cargo fmt --check` + `cargo test --workspace`.
3. Microbench A/B re-run for any dispatch/hot-path touch; neighbors must not regress.
4. Grep gates named by the brief (e.g. Stage 1: `grep -rn "BuiltinHandler" crates/`
   must be empty).
