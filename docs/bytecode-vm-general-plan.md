# General Bytecode VM Plan (generalizing the exact-shape body compiler)

Status: proposed, 2026-07-01. Follows the completed slice line
(`docs/bytecode-vm-design.md`): slices 1-3 shipped an OPT-IN compiled-body VM that
compiles a FEW hand-matched body shapes (slice 1 = a general construct-only op-stream
for pure RHS substitution via `CodeProgram::Construct`; slices 2/3 = PER-SHAPE control
programs `ModuleIfSameQ`/`ModuleRepeatC` for ORELSEC/TRYCONV/REPEATC). This plan
generalizes the per-shape control programs into a GENERAL instruction stream so that
ARBITRARY rule/function/Module bodies compile — no exact-shape matcher, no
fallback-for-anything-non-exact.

## Why we build this regardless of mma-hol payoff (user directive 2026-07-01)

**Do NOT gate this line on mma-hol-measured payoff, and do NOT run a payoff-sizing
diagnostic for it.** rum is a GENERAL Wolfram-language kernel; mma-hol is ONE
application, not the universal benchmark. Data measured on mma-hol (the ~parity real
loads, the probe-specific 2x) says how the VM helps *this* project — it must NOT be
used as a directional decision to build/skip a general CAPABILITY. The general VM is a
technical-capability investment: once it exists it may be high-value in some OTHER
workload we don't have yet, and it is OPT-IN (zero-cost off) so it costs nothing when
unused. Therefore: **just build it.** The only non-negotiable gate is CORRECTNESS
(byte-identical + opt-in); payoff is not a gate here.

## Goal

A general body compiler: `compile_body(expr) -> Option<CodeObject>` that lowers an
arbitrary body (the WL subset that appears in rule RHS / function / Module bodies) to
a general IR instruction stream with control flow, run by a general executor. Replace
the `ModuleIfSameQ`/`ModuleRepeatC` per-shape variants with the general path; keep the
opt-in flag, zero-cost off, byte-identical on.

## The general IR (generalize the current `Op`/`CodeProgram`)

Current `Op`: `PushConst`, `PushBinding`, `PushLocal`, `Construct{argc}`, `Eval`.
`CodeProgram`: `Construct{ops}` (general, linear) + `ModuleIfSameQ`/`ModuleRepeatC`
(per-shape). Generalize to ONE `CodeProgram` = a general instruction stream + a
program counter, adding control flow:

- `JumpIfFalse(label)`, `Jump(label)` — general `If`/branching (any nesting), replacing
  the hardcoded per-shape branches.
- `Return` — early-exit (generalizing the two hardcoded REPEATC returns); propagates as
  the body result (mirrors `Return` caught at the Module boundary).
- `SetLocal(slot)` — `Set[local, e]` to a register-ized local (pop value → register).
- `Discard`/`Pop` — `CompoundExpression` intermediate results.
- `SameQ` (and other cheap total comparisons) native, or via `Eval`.
- `AllocGensym(slot)` + own-value binding ops — for ESCAPING Module locals (the general
  hygiene case; slices 1-3 avoided it by only register-izing non-escaping locals).
- Keep `Eval` for the irreducible symbolic dispatch (delegates to the tree-walker;
  suspension via `suspend_with_frame_lazy`).

The executor becomes a PC loop over the stream (operand stack + register file),
Return-aware and suspension-aware.

## The general compiler (recursive over the body grammar)

`compile_body(expr)` handles the body grammar and FALLS BACK (returns `None`) for
anything outside it (opt-in ⇒ fallback is always safe):

- atom / bound pattern var / Module local → `PushConst`/`PushBinding`/`PushLocal`.
- `CompoundExpression[s1,...,sn]` → compile each, `Discard` all but the last.
- `Set[local, e]` → compile `e`, `SetLocal`.
- `If[c,t,e]` / `If[c,t]` → compile `c`, `JumpIfFalse`, compile `t`, `Jump end`, compile
  `e` — hold-correct per `If`'s semantics.
- `Return[x]` → compile `x`, `Return`.
- `Module[{locals}, body]` / `With` → escape-analyze each local (register vs gensym),
  emit `AllocGensym` for escaping ones, compile the body.
- application `f[args]` (not a special form) → compile head+args, `Construct`, then
  `Eval` (hold-correct: only eval where WL would).
- `SameQ`/comparisons → native op where total, else `Construct`+`Eval`.

## Correctness = byte-identical for ARBITRARY bodies (the ONLY hard gate)

The blast radius is far larger than 3 shapes. The general compiler+executor must
replicate EXACTLY: evaluation ORDER, attributes/hold (`HoldAll`/`HoldFirst`/…),
`Sequence` splicing, `Nothing`-elision, `Return` propagation (own vs sub-call), and
Module HYGIENE (incl. escaping locals). Gate:
- the existing debug `debug_assert_eq!` (every compiled application vs the tree-walk) —
  now covering FAR more bodies;
- `cargo test --workspace` OFF and ON identical;
- a BROAD corpus differential ON vs OFF (the whole test suite; and — because escaping-
  local hygiene touches the trust boundary — a STRICT-mode check per
  `docs/bytecode-vm-design.md`), since a general compiler compiles bodies across the
  entire tower, not just Drule's combinators.

(Note: the corpus differential here is a CORRECTNESS check — "the compiled path gives
identical results" — NOT a payoff measurement.)

## Incremental path (for ENGINEERING RISK, not payoff — byte-identical-gated each step)

Build incrementally because byte-identical for arbitrary bodies is HARD, not to gate
on payoff. Each step is opt-in, byte-identical-gated, and reports compile COVERAGE
(how many more bodies compile — a capability metric, not a go/no-go).

1. **General control flow + non-escaping locals**: replace `ModuleIfSameQ`/
   `ModuleRepeatC` with the general instruction stream (`Jump*`/`Return`/`SetLocal`/
   `Discard` + PC executor), compiling arbitrary `If`/`CompoundExpression`/`Return`/
   `Set`-to-local bodies where ALL Module locals are provably non-escaping
   (register-ized). Generalizes slices 2/3 to arbitrary control shapes WITHOUT the
   escaping-local hygiene risk. Byte-identical gate + coverage.
2. **Escaping Module locals (general hygiene)** — `AllocGensym` + own-value binding,
   reusing the existing Module gensym machinery + GC (049/050), with the airtight
   kernel-state-Module exclusion. The riskiest step (the LCF trust boundary), done LAST,
   gated hard against the Strict suite.

Each step keeps the tree-walker the default; the VM stays opt-in.

## Risk register

- **Byte-identical blast radius.** A general compiler compiles bodies across the whole
  tower; a single semantic mismatch (order/hold/Sequence/Return) is a correctness bug.
  The debug differential + broad corpus ON-vs-OFF is the gate; extend the grammar
  incrementally, never big-bang.
- **Escaping-local hygiene = the trust boundary.** Getting Module gensym hygiene wrong
  for escaping locals could break the LCF encapsulation (`docs/bytecode-vm-design.md`).
  Do it LAST, gate against Strict, exclude kernel-state Module airtight.
- **Do NOT over-index on mma-hol for directional decisions.** mma-hol is one
  application; ~parity there is not a reason to stop a general capability. Payoff will
  surface in other workloads; the feature is opt-in so it is free when unused.
- **Complexity budget.** If byte-identical for a grammar extension is genuinely
  unachievable at reasonable cost, STOP at the prior step and bank the IR/capability
  (a correctness limit, not a payoff decision).

## First brief

`docs/dev/codex/brief-073-general-vm-control-flow.md` — step 1: the general instruction
stream (`Jump*`/`Return`/`SetLocal`/`Discard` + PC executor) + the recursive
`compile_body` for arbitrary `If`/`CompoundExpression`/`Return`/`Set` bodies with
non-escaping (register-ized) locals, replacing the `ModuleIfSameQ`/`ModuleRepeatC`
per-shape programs. Opt-in, byte-identical, zero-cost off.
