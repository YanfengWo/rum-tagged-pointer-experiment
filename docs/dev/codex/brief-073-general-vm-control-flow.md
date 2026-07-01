# brief-073 — general VM step 1: general control-flow instruction stream + `compile_body`

## Goal

Generalize the compiled-body VM (`docs/bytecode-vm-general-plan.md` step 1): replace
the per-shape `CodeProgram::ModuleIfSameQ` / `ModuleRepeatC` (slices 2/3, hand-matched
combinator shapes) with a GENERAL instruction stream (a `Vec<Op>` with labels/jumps +
a program-counter executor) and a recursive `compile_body(expr)` that lowers ARBITRARY
rule/function/Module bodies in the body grammar — any nesting of `If` /
`CompoundExpression` / `Return` / `Set`-to-local / application — where all Module
locals are provably NON-ESCAPING (register-ized). This is a CAPABILITY line: **do NOT
measure or gate on mma-hol payoff** (rum is a general kernel; mma-hol is one
application — see `docs/bytecode-vm-general-plan.md` "Why we build this regardless of
mma-hol payoff" and memory `rum-is-general-not-just-mma-hol`). The ONLY hard gate is
CORRECTNESS: byte-identical to the tree-walk, opt-in, zero-cost off. Escaping Module
locals (general hygiene) are OUT — fall back for them (that is step 2).

## Context pointers

- Read `docs/bytecode-vm-general-plan.md` (the whole plan: the general IR, the
  `compile_body` grammar, the incremental path, "build regardless of payoff") and
  `docs/bytecode-vm-design.md` (the opt-in/byte-identical/trust-boundary invariants).
  Read brief-070/071/072 Outcomes (the machinery you generalize).
- Key file: `crates/rum-runtime/src/runtime/vm_rhs_template.rs` (1705 lines) — current
  `Op` (`PushConst`/`PushBinding`/`PushLocal`/`Construct`/`Eval`), `CodeProgram`
  (`Construct{ops}` general-linear + `ModuleIfSameQ`/`ModuleRepeatC` per-shape), the
  executor, `compile_rhs_template`, the exact-shape matchers, the coverage counters,
  and the slice-1/2/3 unit tests (which MUST still pass — the general path must
  reproduce those shapes exactly).
- rum `Return` = `Err(EvalError::Return{value})` caught at the Module boundary
  (`control_flow/nonlocal.rs`); `suspend_with_frame_lazy` /
  `suspension/core.rs` for the suspension continuation; `eval_if` for `If` truthiness;
  `SameQ` semantics. REUSE these, do not reinvent.
- The compilable-shape flag stays `RUM_VM_RHS_TEMPLATE` (one opt-in switch), zero-cost
  off (one relaxed `AtomicU8` load → unchanged tree-walk).

## What to build

1. **General IR.** Extend `Op` with control flow: `JumpIfFalse(target)`, `Jump(target)`
   (indices into the op vector), `Return` (early-exit = body result), `SetLocal(slot)`
   (`Set[local, e]`: pop value → register), `Discard` (drop a `CompoundExpression`
   intermediate). Keep `PushConst`/`PushBinding`/`PushLocal`/`Construct`/`Eval` and the
   native `SameQ` compare. Collapse `CodeProgram` to a SINGLE general
   `{ ops: Vec<Op>, locals: … }` form; remove `ModuleIfSameQ`/`ModuleRepeatC` (they
   become ordinary outputs of `compile_body`).
2. **PC executor.** A loop over `ops` with a program counter, an operand stack, and a
   register file for locals. `Return` = stop, body result = TOS. Suspension: an
   `Op::Eval` that returns `Err(EvalError::Return{..})` from a SUB-call propagates
   unchanged; an `Op::Eval` that returns `Err(InputRequired)` must push a resume frame
   (via `suspend_with_frame_lazy`) capturing `(ops, pc+1, stack, registers)` so the PC
   loop RESUMES at the next op with the evaluated value pushed — the general analog of
   slices 2/3's per-Op::Eval continuations. (InputRequired never fires on the hot
   conversion path, but the workspace ON test suite exercises it — it MUST be correct.)
3. **`compile_body(expr) -> Option<CodeObject>`.** Recursive over the grammar:
   - atom / bound pattern var / Module local → `PushConst`/`PushBinding`/`PushLocal`;
   - `CompoundExpression[s1..sn]` → compile each; `Discard` all but the last;
   - `Set[local, e]` (local = a register-ized Module local) → compile `e`, `SetLocal`;
   - `If[c,t,e]` and `If[c,t]` (2/3-arg) → compile `c`, `JumpIfFalse L1`, compile `t`,
     `Jump L2`, `L1:` compile `e` (or the implicit `Null`/no-op for 2-arg), `L2:` —
     hold-correct;
   - `Return[x]` → compile `x`, `Return`;
   - `Module[{locals}, body]` / `With[{...}, body]` → escape-analyze each local; if ALL
     are provably non-escaping (occur only as `Set` LHS + bare evaluated reads, never
     held/symbolic/returned-as-symbol), register-ize and compile the body; if ANY
     escapes → `None` (fall back — step 2);
   - application `f[args]` (ordinary head, not a special form the grammar handles) →
     compile head + args, `Construct`, `Eval` (hold-correct: eval only where WL does);
   - `SameQ[a,b]` (`===`) → native compare where total, else `Construct`+`Eval`;
   - anything else → `None` (fall back).
   Never register-ize / never match `defineKernel`'s kernel-state Module.

The general path MUST reproduce the slice-1/2/3 shapes byte-identically (ORELSEC/
TRYCONV/REPEATC now compile via `compile_body`, not the deleted per-shape matchers).

## rum / Rust pitfalls (read twice)

1. **Byte-identical is the ONLY gate that matters here — and it is HARD for arbitrary
   bodies.** Evaluation ORDER, attributes/hold, `Sequence` splicing, `Nothing`,
   `Return` (own early-exit vs sub-call propagation), and non-escaping-local semantics
   must match the tree-walk EXACTLY. When unsure, FALL BACK (return `None`) — a missed
   compile is safe; a wrong compile is a correctness bug.
2. **Suspension in the PC executor.** The resume frame must capture the full executor
   state (ops, pc, stack, registers) and re-enter the loop; reuse
   `suspend_with_frame_lazy`, do NOT bail-after-eval (side effects already ran).
3. **Escape analysis: non-escaping ONLY.** Escaping locals → fall back (step 2). Never
   match kernel-state Module.
4. **Zero-cost off; no new `unsafe`; opt-in — do NOT flip any default.**
5. **Stop-loss.** If byte-identical for a grammar construct can't be met, NARROW the
   grammar (fall back for it) until green, or STOP and report. Same failure twice → STOP.
6. **No payoff measurement.** This is a capability line; do not run a Pair/timing probe
   to justify it. (If you time anything for your own curiosity: raw-to-file first, rum
   timing is a rational num/den, probe budget applies — but it is not required.)

## Verification (MANDATORY — correctness only; paste verbatim)

```
cargo fmt --check
cargo test --workspace                          # OFF baseline green
RUM_VM_RHS_TEMPLATE=1 cargo test --workspace    # ON must be EQUALLY green (byte-identical proof)
```

Both MUST be identically green, and ALL existing slice-1/2/3 tests MUST still pass
(the general path subsumes them). Then the real-combinator differential:

```
# drule_tests exercises ORELSEC/TRYCONV/REPEATC via the NEW general path — OFF vs ON identical:
<run tests/drule_tests via mma-hol-bounded-smoke --only test/drule_tests, OFF and ON, diff>
```

Paste: both `cargo test --workspace` summaries; confirmation the slice-1/2/3 tests
pass under the general compiler; drule_tests OFF-vs-ON identical `List[50,0]`; the
coverage counter (how many distinct bodies now compile via the general path vs fall
back — a CAPABILITY metric).

## Hard rules

- No `git commit`/branch/push. Leave changes in the tree for Claude to gate.
- Modify `vm_rhs_template.rs` (+ `mod.rs`/`definition_application.rs` only if strictly
  needed). Never edit `mma-hol`; no `Expr`/serde change; no new `unsafe`.
- OFF path zero-cost; opt-in — do NOT flip any default.
- Do NOT measure/gate on mma-hol payoff (capability line). Correctness (byte-identical)
  is the only gate.
- Stop-loss: byte-identical unmet → narrow grammar / fall back, or STOP; same failure
  twice → STOP.
- Paste REAL verbatim output; never fake a green.

## Report format

1. What changed — the general IR ops, the PC executor (incl. suspension resume), the
   `compile_body` grammar + what falls back (escaping locals, out-of-grammar); confirm
   `ModuleIfSameQ`/`ModuleRepeatC` removed and subsumed; no `Expr`/serde change; never
   matches kernel-state Module.
2. Verification — both `cargo test --workspace` summaries, slice-1/2/3 tests still green,
   drule_tests OFF-vs-ON identical, the coverage counter — all verbatim.
3. Finding — the general control-flow compiler is byte-identical (capability delivered);
   compile coverage (how many more body shapes compile now).
4. Stop-loss / open questions — what fell back, any order/hold/Return/suspension edge
   you were unsure about.

---

## Outcome (Claude gate, 2026-07-01) — FAILED the byte-identical gate; REVERTED. Root cause: the general compiler needs attribute-aware compilation.

Codex built the general refactor (one `CodeProgram{ops, locals, evaluates}` + control ops
`JumpIfFalse`/`Jump`/`Return`/`SetLocal`/`Discard` + native `SameQ` + a PC executor with
suspension resume + a recursive `compile_body`, removing `ModuleIfSameQ`/`ModuleRepeatC`)
and honestly STOPPED at the stop-loss when the gate failed:
- `cargo test --workspace` 723/0 OFF and ON — but rum unit tests don't exercise the
  problematic bodies, so this was a false-green.
- **`drule_tests` OFF 50/0, ON `List[38, 11]`** — the general `compile_body` OVER-ACCEPTED
  bodies it could not compile byte-identically → 11 real failures under the flag.
  Coverage `general_body_distinct=22` (22 bodies compiled, some wrong).

**Root cause:** byte-identical general body compilation requires ATTRIBUTE/HOLD-AWARE
compilation — the compiler must know each application head's attributes and only `Eval`
the args WL would evaluate, in WL's order. The naive grammar Eval'd/ordered wrong for
some 2-local Drule helper bodies outside the combinator-safe subset. A non-byte-identical
VM is not shippable even opt-in (the ON path returns wrong results), so Claude REVERTED to
e02e39c (slices 1-3 intact, tree clean). The general IR/executor machinery looked sound;
the defect is in `compile_body`'s acceptance being too permissive without attribute
modeling.

**Next (a fresh decision):** either re-brief with a CONSERVATIVE, ATTRIBUTE-AWARE
`compile_body` (only compile a body if every application head's attributes are known and
handled hold-correctly; fall back otherwise; gate = `drule_tests` ON == 50/0), OR defer the
general VM (slices 1-3 remain the shipped opt-in VM). Note: a conservative attribute-aware
version may compile little MORE than slices 2/3 already do — the attribute modeling is the
hard general part.
