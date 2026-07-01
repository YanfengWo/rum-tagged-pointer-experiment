# brief-074 — closure-generation VM: foundation + leaf-delegation + control skeleton

## Goal

Rebuild the general body VM as CLOSURE GENERATION (per
`docs/bytecode-vm-closure-plan.md`), replacing the op-stream approach that brief-073
proved fragile. `compile_body(expr)` walks a body ONCE and returns a tree of Rust
closures; running the tree replaces re-walking the AST. It compiles the CONTROL SKELETON
(`If`/`CompoundExpression`/`Return`/`Set`-to-local/non-escaping-`Module`) + the
substitution structure, and DELEGATES every leaf application's semantics to the real
`evaluate`/`evaluate_call`. It must SUBSUME slices 1-3 byte-identically (drule ON == 50/0,
all existing VM tests pass) AND compile arbitrary control skeletons beyond the 3
combinators. Opt-in (`RUM_VM_RHS_TEMPLATE`), zero-cost off, byte-identical on. This is a
CAPABILITY line — the ONLY gate is CORRECTNESS; do NOT measure or gate on mma-hol payoff.

## THE ONE RULE (this is why brief-073 failed and this must not)

**Compile ONLY the control skeleton + substitution structure. DELEGATE all application /
attribute / hold / evaluation-order / dispatch semantics to the real `evaluate` /
`evaluate_call`. NEVER decide per-head which args to `Eval` (that is what brief-073's
op-compiler did — a second semantics spec that diverged, drule ON 38/11).**

Concretely: a leaf application `f[args]` closure SUBSTITUTES bindings into the still-HELD
args, builds the call node, and hands it to `evaluate` — which applies the head's
attributes/hold/order/dispatch. The compiler does NOT look at `f`'s attributes or decide
which args to evaluate. Only the control forms (`If`/`CompoundExpression`/`Return`/`Set`/
`Module`-locals) are compiled into closure structure, using the REAL primitives
(`truthiness_symbol` for `If`, the `Err(EvalError::Return)` protocol for `Return`,
register reads for locals). When unsure whether a construct can be compiled
byte-identically → FALL BACK (`None`).

## Context pointers

- Read `docs/bytecode-vm-closure-plan.md` (the architecture + node kinds + the delegation
  rule), `docs/bytecode-vm-research-notes.md` (WHY: derive from the interpreter, don't
  re-specify semantics), `docs/bytecode-vm-design.md` (opt-in/byte-identical/trust-boundary
  invariants), and brief-073's Outcome (what failed and the root cause).
- Key file: `crates/rum-runtime/src/runtime/vm_rhs_template.rs` (current op-stream VM +
  slices 1-3 tests). REWRITE its compile/execute to closure generation; keep the env gate,
  the coverage counters, and the debug `debug_assert_eq!`-vs-tree-walk differential.
  Remove the op-stream `Op`/`CodeProgram::ModuleIfSameQ`/`ModuleRepeatC` ONCE the closure
  path subsumes them (all slice-1/2/3 tests + drule ON must pass first).
- Reuse (do NOT reinvent): `evaluate`/`evaluate_call` (the delegation target),
  `binding_substitution` (substitute bindings into held args), `eval_if`'s
  `truthiness_symbol`, `Err(EvalError::Return{value})` (`control_flow/nonlocal.rs`, caught
  at the Module boundary), `suspend_with_frame_lazy` (`suspension/core.rs`), the existing
  Module gensym machinery (for the LATER escaping-locals step — NOT this brief; fall back
  for escaping locals now).
- rum `Return`/`Module`/`If`/`SameQ` semantics as used by ORELSEC/TRYCONV/REPEATC in the
  acceptance clone `Drule.wl` (read-only reference; the drule tests are the real gate).

## Architecture (build this)

`compile_body(expr) -> Option<CompiledBody>` → a tree of
`Box<dyn Fn(&mut Session, &Bindings) -> Result<Expr, EvalError> + Send>`. Node kinds:
- constant subtree (no bindings/locals) → return a pre-shared `Expr`.
- binding var / Module local → return the bound value / register value.
- leaf application `f[args]` (ANY head) → substitute bindings into held args, build the
  call, `evaluate` it (delegation — no per-head decision).
- `If[c,t,e]`/`If[c,t]` → cond closure → REAL truthiness → taken-branch closure.
- `CompoundExpression[s1..sn]` → run each; discard all but the last.
- `Return[x]` → run x, early-exit via the `Err(EvalError::Return)` protocol.
- `Set[local, e]` → run e, bind result to the local's register.
- `Module[{locals}, body]`/`With` → register-ize provably NON-escaping locals + compile
  body; FALL BACK if any local escapes (escaping-locals is the next brief). Never match
  `defineKernel`'s kernel-state Module.
- anything else → `None` (fall back).
Suspension: `Return`/`InputRequired` propagate through the `Err` channel; an application
whose `evaluate` suspends resumes the enclosing closure via `suspend_with_frame_lazy`
(the slices-2/3 pattern). Closures are `Send`.

## Scope

- MAY modify: `crates/rum-runtime/src/runtime/vm_rhs_template.rs` (the rewrite),
  `mod.rs`/`definition_application.rs` only if strictly needed.
- MUST NOT: `Expr`/`Symbol`/serde; any `mma-hol` file; `AGENTS.md`/`PROGRESS.md`/`TODO.md`/
  `docs/**` except your report; `.git`; no new `unsafe`.
- Zero-cost off (the existing `RUM_VM_RHS_TEMPLATE` gate: one relaxed `AtomicU8` load →
  unchanged tree-walk). Opt-in — do NOT flip any default.

## rum / Rust pitfalls (read twice)

1. **DELEGATE, don't re-derive** (the ONE RULE above). This is the whole point.
2. **Byte-identical is the only gate**; delegation shrinks the risk to the control
   skeleton (If truthiness / CompoundExpression order / Return propagation / register-ized
   locals) — a small, checkable surface. Fall back when unsure.
3. **Closures must be `Send`** (Session is Send). Capture children by move; box them.
4. **Suspension**: reuse `suspend_with_frame_lazy`; don't bail-after-eval; propagate
   `Return`/`InputRequired` through `Err`.
5. **Escaping Module locals → fall back** (next brief). Never match kernel-state Module.
6. **No payoff measurement** (capability line). If you time anything for curiosity:
   raw-to-file first; rum timing is a rational num/den; probe budget applies — not required.
7. **Stop-loss**: if a construct can't be made byte-identical, fall back for it; if
   `drule_tests` ON != 50/0, narrow until it is or STOP and report; same failure twice → STOP.

## Verification (MANDATORY — correctness only; paste verbatim)

```
cargo fmt --check
cargo test --workspace                          # OFF baseline green
RUM_VM_RHS_TEMPLATE=1 cargo test --workspace    # ON must be EQUALLY green (byte-identical proof)
```

Both identically green, and ALL existing slice-1/2/3 tests MUST still pass (the closure
path subsumes them — ORELSEC/TRYCONV/REPEATC now compile via the GENERAL closure control
compiler, not the removed per-shape matchers). Then the real-combinator differential:

```
# drule_tests exercises ORELSEC/TRYCONV/REPEATC via the NEW closure path — OFF vs ON identical:
<run tests/drule_tests via mma-hol-bounded-smoke --only test/drule_tests, OFF and ON, diff>
```

Paste: both `cargo test --workspace` summaries; confirmation slice-1/2/3 tests pass;
`drule_tests` OFF-vs-ON identical `List[50,0]` (THE gate brief-073 failed); the coverage
counter (how many body shapes compile via the closure path vs fall back — a capability
metric).

## Hard rules

- No `git commit`/branch/push. Leave changes in the tree for Claude to gate.
- No files outside Scope; never edit `mma-hol`; no `Expr`/serde change; no new `unsafe`.
- OFF path zero-cost; opt-in — do NOT flip any default.
- CORRECTNESS (byte-identical) is the only gate; do NOT gate on mma-hol payoff.
- Stop-loss: construct not byte-identical → fall back; drule ON != 50/0 → narrow or STOP;
  same failure twice → STOP.
- Paste REAL verbatim output; never fake a green.

## Report format

1. What changed — the closure IR + `compile_body` grammar + the DELEGATION of leaf
   applications to `evaluate`; what falls back; confirm the op-stream ModuleIfSameQ/
   ModuleRepeatC removed and subsumed; no `Expr`/serde change; never matches kernel-state
   Module.
2. Verification — both `cargo test --workspace` summaries, slice-1/2/3 tests green,
   drule_tests OFF-vs-ON identical `List[50,0]`, coverage counter — all verbatim.
3. Finding — closure generation is byte-identical (the capability delivered where the
   op-compiler failed); compile coverage (how many more shapes than slices 1-3).
4. Stop-loss / open questions — what fell back, any control-skeleton edge you were unsure
   about.
