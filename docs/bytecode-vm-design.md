# Bytecode VM Design (step-3 of the bytecode-VM line)

Status: proposed, 2026-07-01. This is the architecture sub-plan for step 3 of
`docs/bytecode-vm-plan.md` — the compiled-body VM. The user chose this route
DELIBERATELY over the cheaper targeted levers (Module-closure resume,
fixed-point/stability): it is the long/hard/high-risk path, but the technique
(compiling a symbolic rewriter's bodies to an executable IR) is a NEW capability
whose value is cross-project and survives even a rollback of this specific
deployment. This doc is therefore a first-class deliverable in its own right, not
just a means to a Pair number.

## Goal

Compile the hot WL "bodies" that rum currently RE-WALKS and RE-DISPATCHES on every
application — down-value rule RHS, the WL control combinators (`ORELSEC`/`REPEATC`/
`TRYCONV`, curried down/sub-value functions), and Module/Function bodies — into a
linear instruction stream (a code object) that is compiled ONCE, cached on the
definition, and EXECUTED per application. This targets the brief-069 measured Pair
prize: ~36% "WL body/combinator" + the 22% `module.resume_initializers` row +
the 1.467x combinator amplification (~58% of the Pair descent is VM-addressable).

**Non-goal (the 069 guardrail): NOT a generic expression-tree VM.** Compiling
arbitrary expressions to "evaluate this call" bytecode just re-encodes the
irreducible symbolic dispatch and buys nothing. The unit of compilation is a BODY
(a held tree instantiated + evaluated repeatedly with fresh bindings), never an
arbitrary runtime term.

## What rum does today (the cost we are compiling away)

Per down-value application (`definition_application.rs::try_apply_down_values` →
`binding_substitution.rs::substitute_rule_bindings_for_rhs`): after a match, rum
WALKS the RHS tree with the bindings map to build the instantiated result (a COW
walk — `substitute_bindings_changed` shares unchanged subtrees, but still WALKS the
whole tree every time to DISCOVER what changed), then hands the result back to
`evaluate`, which re-descends and re-dispatches it. For a control combinator body
(`ORELSEC[a,b]` etc.), that body is itself a down-value whose RHS is re-walked +
re-evaluated on every conversion attempt — the 1.467x amplification. For a Module
body, `module.resume_initializers` rebuilds/re-evaluates the closure per subterm.
The compiled body replaces the per-application WALK + re-dispatch drive with a
precompiled op stream.

## The IR (symbolic, not arithmetic)

A code object is `Vec<Op>` + metadata (arity, local/binding slot count, the source
symbol for provenance). The operand model is a small operand stack + a binding
register file (pattern vars / Module locals bound before execution). Ops:

- `PushConst(SharedExpr)` — push a PRE-SHARED constant subtree of the body (the
  parts with no bindings; shared once at compile time, never rebuilt).
- `PushBinding(slot)` — push the value bound to a pattern-var / local slot.
- `Construct { head_from, argc }` — pop `argc` args (+ head) and build a `Call`
  node. This is the compiled analog of the substitution rebuild.
- `Eval` — invoke the existing evaluator (`evaluate`) on the top-of-stack. This is
  the IRREDUCIBLE symbolic dispatch — the VM does NOT reimplement it, it CALLS it.
- `EvalHeld` / hold-aware variants — respect the body head's attributes
  (`HoldAll`/`HoldFirst`/…) so evaluation order is byte-identical to the tree-walk.
- Control: `BranchIfFailure(target)` / `Jump(target)` (compile `If`/`ORELSEC`
  short-circuit), `Fixpoint(block)` (compile `REPEATC`'s converge loop).
- `AllocLocals(n)` — allocate fresh gensyms for a Module body (hygiene), binding
  them into the register file (slice 3 only).
- `Return`.

The IR is deliberately small and closed; extending it is how later slices add
capability. It is NOT Turing-complete beyond what the compiled bodies need.

## Interop / fallback model (non-negotiable)

The VM is a COMPILED FAST-PATH layered on the tree-walker, never a replacement:

- The tree-walker (`evaluate`/`evaluate_call`) stays the DEFAULT and the source of
  truth. The VM is behind a feature flag / env gate; OFF ⇒ zero cost (the standing
  rule), the existing path unchanged.
- A body is compiled LAZILY on first application and cached on the definition
  (invalidated on the definition's epoch bump — reuse the `evaluation_epoch`
  machinery). Compilation failure (an un-compilable body shape) FALLS BACK to the
  tree-walk silently — the VM only ever handles shapes it fully supports; anything
  else uses today's path. This makes every slice safe and partial.
- `Op::Eval` calls back into `evaluate`, so the VM handles the STRUCTURE
  (construct/bind/branch) and delegates the irreducible symbolic work. Suspension/
  resume: an `Eval` that suspends must suspend the VM frame too (preserve the
  continuation contract) — see invariants.

## Module is the project's LOAD-BEARING trust boundary — gate Module-touching slices against the REAL Strict-mode suite

mma-hol's entire premise (README) is `Module`/`Unique` gensyms AS an encapsulation
/ LCF trust boundary: in **Strict mode** the kernel-state symbols (`thmTag`, the
constant/axiom/definition tables) are **Module-local gensyms with no stable name**,
and soundness reduces to "no code outside the kernel can fabricate a `thmTag[...]`."
So Module hygiene here is not an optimization detail — it is the point of the
project. Consequences for any VM slice that compiles a Module body:

- **The escape check must be airtight.** Register-izing a Module local (skipping its
  gensym) is sound ONLY for a local that provably never escapes and is NOT a
  kernel-state / theorem-constructor symbol. A compilable-shape match must be narrow
  enough that it can NEVER match `defineKernel`'s kernel-state Module. When in doubt,
  fall back.
- **Correctness gate MUST include the real mma-hol theorem suite with the flag ON,
  in Strict mode** — not just the Stable Pair probe + rum unit tests, which never
  cross the trust boundary. Minimum: `tests/run_fast.wls` (Stable, all post-bootstrap
  files) OFF vs ON identical pass count, PLUS a Strict-mode check (a `run_all.wls`
  subset, real gensyms) OFF vs ON. A Module slice that passes rum unit tests + the
  Stable Pair probe but is not run against the Strict suite is NOT adequately gated.

## Correctness invariants (the hard part — gate every slice on these)

A compiled body MUST be byte-identical to the tree-walk, or it is a correctness
bug, not a perf tradeoff:

- Same evaluation ORDER and same attribute handling (`HoldAll`/`HoldFirst`/
  `SequenceHold`/`Flat`/`Orderless`) — the compiler reads the head's attributes and
  emits hold-correct `Eval`/`EvalHeld` ops.
- `Sequence` splicing, `Evaluate`-in-hold, and `Nothing`-elision identical.
- Pattern-var binding semantics identical (incl. `Optional`/default, `x__`
  sequence bindings → a slot holds a sequence).
- Module HYGIENE identical (fresh gensyms per instantiation — slice 3).
- Suspension/resume continuation shape preserved (an `Eval` mid-stream that
  suspends resumes the VM at the right op).
- serde/snapshot unaffected (the code object is an in-memory cache, never
  serialized; a restored session recompiles lazily).

Gate: `cargo test --workspace` + a dedicated byte-identical differential (run the
same corpus with the VM OFF and ON, assert identical results) + the unmodified
mma-hol smoke. A single divergence blocks the slice.

## Measured value + the OPT-IN TOGGLE principle (2026-07-01, user directive)

Measured reality after slices 1+2 (ON vs OFF, same binary, Method M = restore
checkpoint + timed single-file `Get`, raw logs in `/tmp/heavyprobe/`):

| workload | ON/OFF | note |
|---|---|---|
| Pair descent probe (`simpFixpointConv` on a hand-hard nested term) | **~0.49 (2x)** | ~100% combinator conversion — slice-2's exact target |
| Option/List/Rat/Real-Field module loads | 0.97–1.07 | ≈ parity (single-run noise band) |
| Set/Finite/Int heavy module loads | 0.95–1.11 | ≈ parity; Int +11% (noise, or unrecouped compile cost) |

**Conclusion: the 2x is PROBE-SPECIFIC, not a general speedup.** The gain is
proportional to the combinator-conversion fraction of the workload; the probe is an
artificial ~100%-combinator stress test, while every REAL module load measured is
≈ parity (combinators are a small fraction). A worst case where a slice is NET
NEGATIVE is plausible (a body compiled but rarely applied doesn't recoup its
one-time compile cost). So the VM line is "opt-in acceleration for
conversion-heavy tasks," NOT a broad win. (`rumPairProbePair4Same` is a specific
expression — NOT the `Pair.wl` stdlib module; the name collision misled earlier.)

**OPT-IN TOGGLE principle (user directive):** each VM/JIT feature is an INDEPENDENT
opt-in switch (env flag), ZERO-cost when off, that a caller turns ON for a workload
that benefits and leaves OFF otherwise. **We do NOT flip any VM feature to
default-ON** — the tree-walker stays the default; the VM is a per-task accelerator.
This makes "before flipping the default ON" moot for the common path (it stays OFF);
the deferred validation (full-tower OFF==ON, Strict) only gates advertising a
feature as safe-to-enable, not a default change. Slice 3 (and any later slice)
follows the same pattern: its own flag, zero-cost off, kept opt-in.

## Grounding correction (2026-07-01, after brief-070): the hot combinators are Module-wrapped

The `HOL`Drule`` conversion combinators that drive the 1.467x amplification are all
curried down-values whose RHS body is a `Module`:

```
ORELSEC[c1_,c2_][t_] := Module[{res}, res = tryConv[c1[t]]; If[res===$convFailed, c2[t], res]]
TRYCONV[c_][t_]      := Module[{res}, res = tryConv[c[t]];   If[res===$convFailed, REFL[t], res]]
REPEATC[c_][t_]      := Module[{res,rhs}, res=tryConv[c[t]]; If[res===$convFailed, Return[REFL[t]]]; rhs=concl[res][[2]]; If[aconv[rhs,t], Return[res]]; TRANS[res, REPEATC[c][rhs]]]
```

So the control flow is NESTED INSIDE a `Module`, and slice 1 already falls back on
all of them (`Module` → `rhs_contains_uncompiled_scope`). This means the 22%
`module.resume_initializers` Pair row IS these combinator bodies, and the
amplification prize requires compiling MODULE bodies — the Module frame is the
FOUNDATION, control layers on top. The original "slice 2 = control, slice 3 =
Module" ordering is revised accordingly. Also confirmed: suspension is triggered
ONLY by `EvalError::InputRequired` (interactive input), which never arises in the
hot conversion descent — so `Op::Eval` is effectively synchronous on the hot path,
and suspension correctness is handled by REUSING rum's existing continuation
primitives (`suspend_with_frame_lazy` / `drive_evaluated_value`) around each
`Op::Eval`, NOT a bespoke VM resume loop.

## Incremental slices (feature-flagged; keep-or-revert per the Pair delta) — REVISED per the grounding above

Each slice compiles a strictly larger body class, is independently committable,
byte-identical-gated, and measured on the Pair descent + microbench under the
`TEMPLATE.md` PROBE BUDGET (best-of-2, cheap probe for the quick A/B). A slice that
does not move Pair is banked as capability (the IR/infra) or reverted — decided on
its Pair number, not faith.

1. **Slice 1 (brief-070) — compiled RHS substitution template.** Ops:
   `PushConst`/`PushBinding`/`Construct` only (NO `Eval`/control). Compile a
   down-value rule RHS into a template that, given the bindings, constructs the
   instantiated result WITHOUT the per-application discovery walk. Purpose:
   validate the compile→cache→execute→byte-identical machinery on the SIMPLEST
   body; the expected Pair win is modest (substitution is part, not all, of the
   body bucket) — this slice's real deliverable is the proven IR + code-object
   cache + fallback, which every later slice reuses. Fall back to the tree-walk for
   any RHS shape the template does not yet support.
2. **Slice 2 (brief-071) — compiled Module-combinator bodies ORELSEC/TRYCONV
   (Module + Set + `Op::Eval` + `If`/`SameQ` branch).** Compile the exact
   `Module[{res}, res = <eval>; If[res === <const>, <alt>, res]]` shape of the two
   hottest combinators: allocate the Module local(s) via the EXISTING module
   gensym-allocation path (so hygiene + the 049/050 gensym GC still apply), `Op::Eval`
   the Set RHS into the local, compare via native `SameQ`, and construct+`Op::Eval`
   ONLY the taken branch (the win: the untaken branch + the If-node dispatch are
   skipped). Suspension is handled by REUSING `suspend_with_frame_lazy` around each
   `Op::Eval` (the same primitive `eval_if` uses) — no bespoke resume loop. This is
   the first slice with a real expected Pair win (attacks the 1.467x amplification +
   part of the 22% Module row). REPEATC (Return + recursion) explicitly falls back.
3. **Slice 3 — REPEATC (non-local `Return` + the self-recursive fixpoint) and any
   remaining Module-body shapes.** Highest control complexity (early `Return` out of
   the Module, the `TRANS[res, REPEATC[c][rhs]]` recursion), done LAST on the proven
   slice-2 machinery.

## Risk register

- **Does not translate to Pair** (the A3 lesson: microbench wins wash out on Pair).
  Every slice is gated on the bounded Pair descent, not just the microbench.
- **Big-bang.** Feature-flagged + sliced; the tree-walker is always the default and
  the fallback. No slice leaves the tree in a half-compiled state across a commit.
- **Semantics drift.** The byte-identical differential is the hard gate; WL
  evaluation order / attributes / hygiene are the trap. Slice 3 (Module hygiene) is
  the riskiest and is last.
- **Compile cost.** Lazy + cached + epoch-invalidated; a body compiled once and
  applied N times amortizes. Measure the first-application compile cost is not a
  regression on cold paths.
- **Complexity budget.** If a slice's byte-identical gate cannot be met at
  reasonable cost, STOP at the prior slice and bank the IR as the cross-project
  capability (the user's stated fallback value).

## First brief

`docs/dev/codex/brief-070-vm-slice1-rhs-template.md` — slice 1: the compiled RHS
substitution template + the code-object cache + the feature flag + the
byte-identical gate + a probe-budgeted Pair/microbench delta.
