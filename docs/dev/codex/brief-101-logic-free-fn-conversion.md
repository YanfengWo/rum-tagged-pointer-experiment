# Brief 101 — convert the `logic` family's handlers from `Session` methods to free functions

## Goal

Stage-2 step 3, pilot family (`docs/crate-rebalance-plan.md`). Builtin handlers
cannot be inherent `Session` methods once their family lives in another crate.
Convert every handler DEFINED IN the `logic` implementation module
(`runtime/logic.rs` + `runtime/logic/{truth,comparison,shared}.rs`) from
`impl Session { pub(in crate::runtime) fn eval_x(&mut self, …) }` to a free
function `pub(in crate::runtime) fn eval_x(session: &mut Session, …)` in the
same file, and point the catalog/adapter references at the free functions.
Zero behavior change; no hot-path delta (a static call stays a static call and
dispatch still goes through the same fn pointers), so no microbench gate —
state that in your report.

## Context pointers

- Read `AGENTS.md` and `docs/crate-rebalance-plan.md` first. Landed so far:
  097 (fn-pointer dispatch), 099 (kernel_api + logic de-glob), 100
  (registration-only dispatch).
- KEY scope fact (verified): the catalog file
  `builtin_catalog/logic.rs` references 69 `Session::eval_*` handlers, but most
  are DEFINED in other implementation modules (`predicates/`,
  `structure_ops/`, …) — catalog file grouping ≠ implementation module
  grouping. This brief converts ONLY the handlers defined in the `logic`
  implementation module: `eval_and`, `eval_or`, `eval_nand`, `eval_nor`,
  `eval_xor`, `eval_equivalent`, `eval_implies`, `eval_not` (truth.rs);
  `eval_equal`, `eval_unequal`, `eval_less`, `eval_greater`, `eval_less_equal`,
  `eval_greater_equal`, `eval_comparison_operator_wrapper`, and the private
  helper `eval_comparison_subset` (comparison.rs). Verify the inventory with
  `grep -rn "fn eval_" crates/rum-runtime/src/runtime/logic*` before starting;
  convert exactly what lives there, nothing else.
- Catalog references live in
  `crates/rum-runtime/src/runtime/builtin_catalog/logic.rs`
  (`Session::eval_and` → the free fn path). `eval_nand`/`eval_nor` are
  args-only signatures — check `runtime/builtin_adapters.rs` for their adapter
  wrappers and update those references instead.
- The `logic` module is inside the runtime glob, but use EXPLICIT paths at the
  reference sites (`crate::runtime::logic::eval_and`) — the point of this line
  is severing glob dependence.
- Facade lesson from brief-099 (standing): items in private submodules need
  per-item `pub use self::<submod>::…;` lines in the family facade
  (`runtime/logic.rs`) to be nameable from outside the family — never a glob
  re-export.

## The design

1. In each logic file, move the methods out of `impl Session` blocks into free
   functions AT THE SAME LOCATION, changing ONLY the receiver:
   `&mut self` → `session: &mut Session` (first parameter), `self.` → `session.`
   inside the bodies. Keep each function's existing parameter list, return
   type, visibility (`pub(in crate::runtime)`; the private
   `eval_comparison_subset` stays private), and body otherwise VERBATIM.
   Internal cross-calls (`self.eval_comparison_subset(…)` →
   `eval_comparison_subset(session, …)`) follow mechanically.
2. Calls to KERNEL `Session` methods inside the bodies
   (`session.evaluate(…)`, `session.invalid_call_with_exact_arg_count(…)`,
   `session.suspend_with_frame(…)`, …) stay method calls — they are the
   kernel's method API, not the family's.
3. Add the per-item `pub use` lines to `runtime/logic.rs` (facade) so the
   converted functions are nameable as `crate::runtime::logic::eval_x`.
4. Update every reference: `builtin_catalog/logic.rs` entries
   (`Session::eval_and` → `crate::runtime::logic::eval_and` — only for the
   converted set), plus any `builtin_adapters.rs` wrappers for nand/nor and any
   other call sites the compiler reveals (list each in the report).
5. Grep gates: `grep -rn "impl Session" crates/rum-runtime/src/runtime/logic.rs crates/rum-runtime/src/runtime/logic/`
   → zero hits; `grep -rn "Session::eval_and\|Session::eval_or\b\|Session::eval_not\b" crates/`
   → zero hits.

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/logic.rs`,
    `crates/rum-runtime/src/runtime/logic/{truth,comparison,shared}.rs`
  - `crates/rum-runtime/src/runtime/builtin_catalog/logic.rs` (reference
    updates for the converted set ONLY)
  - `crates/rum-runtime/src/runtime/builtin_adapters.rs` (nand/nor and any
    adapter that wraps a converted handler)
  - Other files ONLY for compiler-forced reference fixes to the converted
    functions — visibility/path edits, no behavior; list every one.
- MUST NOT touch: any other family's implementation or its catalog entries,
  `kernel_api.rs` (unless a converted fn must be re-exported there — it must
  NOT; these are handlers, not kernel API), `eval_dispatch/`, `resolution.rs`,
  other crates, any `mma-hol` clone, `docs/**` except your report, `.git`,
  `Cargo.toml`/`Cargo.lock`.

## Deliverable

- The conversion in the working tree (no commit); converted-function inventory
  in the report (name → file → reference sites updated).
- Verbatim gates below.

## rum / Rust pitfalls (read twice)

1. **Bodies verbatim.** This is a receiver change, not a rewrite. Any
   evaluation-order or message-text difference is a defect. Suspension shapes
   (`suspend_with_frame` in comparison/truth flows) must survive untouched.
2. **fn-pointer coercion still works**: a free `fn eval_and(&mut Session, &Expr,
   &[Expr]) -> Result<Expr, EvalError>` coerces to `BuiltinFn` exactly like the
   method path did. Args-only signatures keep their adapters.
3. **Don't convert what isn't logic's**: the other ~53 `Session::eval_*`
   references in `builtin_catalog/logic.rs` belong to other families and stay
   method references until their family's brief.
4. Zero behavior change: `cargo test --workspace` totals must match the current
   1298/0.
5. Stop-loss: same failure twice → STOP and report.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test --workspace
grep -rn "impl Session" crates/rum-runtime/src/runtime/logic.rs crates/rum-runtime/src/runtime/logic/   # zero hits
grep -rn "Session::eval_and\|Session::eval_xor" crates/                                                   # zero hits
```

Paste all outputs verbatim (per-crate totals or the aggregate). No microbench:
static-call shape is unchanged (justify in one sentence in the report).

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config; changes stay in the
  working tree for Claude's gate.
- Do NOT modify files outside Scope; STOP and report if you believe you must.
- Full machine access, but nothing outside this repo, no network; only file
  edits in Scope, `cargo`, `grep`, `python3`.
- Never edit any `mma-hol` file. **No new `unsafe`.**
- Clean-check probes: `git status --short -- crates/` (Claude may commit docs
  concurrently).
- Minimal diff; no drive-by reformatting beyond what rustfmt forces.
- Paste real verbatim outputs; never fake a green.

## Report format (your final message)

1. What changed — converted-function inventory; facade `pub use` lines; every
   reference site updated (catalog/adapters/other).
2. Verification — each command with exact verbatim output.
3. Verdict — conversion complete, behavior-identical, one-sentence no-hot-path
   justification.
4. Stop-loss / open questions (empty if none).
