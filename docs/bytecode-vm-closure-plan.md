# Closure-Generation VM Plan (the corrected general-VM architecture)

Status: proposed, 2026-07-01. SUPERSEDES the op-stream approach in
`docs/bytecode-vm-general-plan.md` (which brief-073 proved fragile — it re-derived WL
application semantics and diverged). Adopts closure generation per
`docs/bytecode-vm-research-notes.md` (Feeley / PyPy principle: derive the fast path FROM
the interpreter, never re-specify semantics). Slices 1-3 (the op-stream VM at e02e39c)
remain the shipped opt-in VM until this closure VM subsumes them byte-identically.

## The one design rule (this is why it will be byte-identical where brief-073 was not)

**Compile ONLY the control skeleton + substitution structure into closures; DELEGATE all
application / attribute / hold / evaluation-order / dispatch semantics to the real
`evaluate` / `evaluate_call`. Never re-derive application semantics in the compiler.**

brief-073 failed because its op-compiler decided *per head* which args to `Eval` (hold/
order) — a second semantics spec that diverged. The closure VM does NOT decide that: a
leaf application closure substitutes bindings into the (still-held) sub-expression and
hands the constructed call to `evaluate`, which applies the head's attributes/hold/order.
Semantics are inherited by construction. The compiler precomputes only STRUCTURE.

## Architecture

`compile_body(expr) -> Option<CompiledBody>` walks the body ONCE and returns a tree of
closures, each `Box<dyn Fn(&mut Session, &Bindings) -> Result<Expr, EvalError> + Send>`
(Send: `Session` is Send). Node kinds:

- **constant subtree** (no bindings/locals) → return a pre-shared `Expr` (structure only).
- **binding var / Module local** → return the bound value / register value.
- **leaf application `f[args]`** (ANY head) → substitute bindings into the held args, build
  the call, hand to `evaluate` (or `evaluate_call`) — ALL hold/attribute/order/dispatch
  semantics come from there. This is the delegation that keeps it byte-identical for
  arbitrary heads WITHOUT modelling attributes.
- **`If[c,t,e]` / `If[c,t]`** → run the cond closure, apply the REAL truthiness test
  (`truthiness_symbol` / the same check `eval_if` uses), run the taken-branch closure.
  Compiles the branch skeleton; delegates cond/branch evaluation.
- **`CompoundExpression[s1..sn]`** → run each closure in order, discard all but the last.
- **`Return[x]`** → run the x closure, early-exit (the `Err(EvalError::Return{..})`
  protocol, caught at the Module boundary exactly as the tree-walk does).
- **`Set[local, e]`** → run the e closure, bind the result to the local's register.
- **`Module[{locals}, body]` / `With`** → escape-analyze; register-ize provably
  non-escaping locals and compile the body; FALL BACK if any local escapes (later step).
- **anything else** → `None` (fall back; opt-in ⇒ fallback is always safe).

**Suspension** reuses the existing contract: closures return `Result<Expr, EvalError>`;
`Return` and `InputRequired` propagate through the `Err` channel, and an application whose
`evaluate` suspends uses `suspend_with_frame_lazy` to resume the enclosing closure — the
same pattern slices 2/3 already use. No bespoke resume machinery.

## Why the win is real but bounded (and that's fine — it's a CAPABILITY line)

The closure chain removes the per-eval AST re-match (the `match expr` dispatch on node
type) and the substitution discovery-walk, and compiles the control skeleton (If/Set/
Return/Module-frame) into direct closure calls instead of constructing+re-dispatching
those nodes. It does NOT remove the leaf-application dispatch (that stays delegated to
`evaluate_call`). So the direct win is bounded (≈ the slices-2/3 combinator win,
generalized) — consistent with the measured ~parity on real loads. The larger payoff, if
ever pursued, is a meta-JIT that SPECIALIZES the closure chain (Lineiform-style). Per
`rum-is-general-not-just-mma-hol`: this is built for CAPABILITY + correctness, not gated
on mma-hol payoff.

## Correctness gate (the ONLY non-negotiable gate)

Byte-identical to the tree-walk, opt-in, zero-cost off. Each step:
- `cargo test --workspace` OFF and ON identical (debug `debug_assert_eq!` on every
  compiled body vs the tree-walk — keep this);
- `drule_tests` ON == 50/0 (must SUBSUME slices 2/3's ORELSEC/TRYCONV/REPEATC via the
  GENERAL closure control compiler — this is exactly where brief-073 failed);
- a broad corpus differential ON vs OFF (CORRECTNESS, not payoff); a Strict-mode check
  once escaping-local hygiene is in (the trust boundary, `docs/bytecode-vm-design.md`);
- coverage counter = how many body shapes now compile (a capability metric).

## Incremental path (opt-in, byte-identical-gated each step)

1. **brief-074 — closure foundation + leaf-delegation + control skeleton.** Build the
   closure IR + `compile_body` for constant/binding/local/leaf-application(delegated)/
   `If`/`CompoundExpression`/`Return`/`Set`-to-local/non-escaping-`Module`. Must SUBSUME
   slices 1-3 byte-identically (drule ON == 50/0, all existing VM tests pass) AND compile
   arbitrary control skeletons beyond the 3 combinators. Once it subsumes them, remove the
   op-stream `ModuleIfSameQ`/`ModuleRepeatC`. This is brief-073's goal with the CORRECT
   (delegating) architecture. Opt-in (`RUM_VM_RHS_TEMPLATE`), zero-cost off.
2. **Escaping Module locals (general hygiene)** — `AllocGensym` + own-value binding via
   the existing Module machinery + GC (049/050), airtight kernel-state-Module exclusion.
   The riskiest step (LCF trust boundary); done LAST, gated against Strict.
3. **(Optional, much later) meta-JIT specialization** of the closure chain (Lineiform-
   style) — the only path to a LARGER win; a separate large decision, not now.

## Risk register

- **Byte-identical is the gate; delegation is the mitigation.** Because application
  semantics are delegated to `evaluate`, the compiler can only be wrong about the CONTROL
  skeleton (If truthiness, CompoundExpression order, Return propagation, register-ized
  locals) — a much smaller, checkable surface than brief-073's per-head attribute
  modelling. When unsure about a construct, FALL BACK.
- **Closure allocation/indirection cost.** Measure STRUCTURE (is the closure chain cheaper
  than the op-stream+PC executor); do not gate on mma-hol payoff.
- **Escaping-local hygiene = the trust boundary.** Step 2, last, Strict-gated.
- **Complexity budget.** If a construct can't be made byte-identical cheaply, fall back and
  bank the rest.

## First brief

`docs/dev/codex/brief-074-closure-vm-foundation.md` — step 1 above.
