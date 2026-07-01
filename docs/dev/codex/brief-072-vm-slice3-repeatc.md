# brief-072 — VM slice 3: compiled REPEATC (Return early-exit + self-recursion)

## Goal

Slice 3 (the last combinator) of the compiled-body VM: compile `HOL`Drule`REPEATC`,
extending `vm_rhs_template.rs` with the two hardest general-VM techniques —
**non-local `Return` (early exit)** and a **self-recursive body**. REPEATC's body:

```
REPEATC[c_][t_] := Module[{res, rhs},
  res = tryConv[c[t]];
  If[res === $convFailed, Return[REFL[t]]];
  rhs = concl[res][[2]];
  If[aconv[rhs, t], Return[res]];
  TRANS[res, HOL`Drule`REPEATC[c][rhs]]
]
```

**Primary deliverable = the CAPABILITY** (Return/early-exit + recursion compilation
in a symbolic VM), per the user's route-C rationale (cross-project value). The
immediate mma-hol perf value is LIMITED and known: slices 1+2 measured ~parity on
real module loads (the 2x is probe-specific; see `docs/bytecode-vm-design.md`
"Measured value + opt-in-toggle principle"). REPEATC is used in the SIMP descent, so
the Pair probe may show some additional gain, but that is a nice-to-have, not the
justification. **Opt-in, zero-cost off, byte-identical on. Do NOT flip any default.**

## Context pointers

- Read `docs/bytecode-vm-design.md` — the WHOLE thing, especially: the "Measured
  value + OPT-IN TOGGLE principle" (VM stays opt-in, never default-ON), the "Module
  is the trust boundary" section (escape checks airtight; NEVER match kernel-state
  Module), and the slice sequence. Read brief-070/071 Outcomes (the machinery you
  extend) and PROGRESS 2026-07-01.
- rum's `Return`: `crates/rum-runtime/src/runtime/control_flow/nonlocal.rs:11`
  (`Return[x]` → `Err(EvalError::Return { value })`), caught at the Module/function
  application boundary (`session_eval.rs:16`, `builtins_input.rs:283`,
  `control_flow/cleanup.rs:128`). So `Return[R]` inside REPEATC's body propagates to
  REPEATC's OWN Module boundary → the Module returns R. In the compiled body this is
  an EARLY EXIT (stop, body result = R).
- Key files: `crates/rum-runtime/src/runtime/vm_rhs_template.rs` (extend: multi-local
  register file, statement sequence, `Return`/early-exit, reuse slice-2's `Op::Eval`
  + `SameQ` + branch + `suspend_with_frame_lazy`); `definition_application.rs` (the
  hook is already routed). The combinator body is in the acceptance clone `Drule.wl`
  (read-only reference).
- The compilable-shape flag stays `RUM_VM_RHS_TEMPLATE` (one switch for the whole
  compiled-body VM feature; extend its compilable set to REPEATC). Zero-cost off.

## Compilable shape (match EXACTLY; fall back otherwise)

Compile a down-value RHS iff it is precisely `Module[{L1, L2}, S1; S2; S3; S4; SF]`
where:
- exactly TWO Module locals `L1`,`L2` (both PURE TEMPORARIES: each occurs only as a
  `Set` LHS and as bare evaluated reads — NEVER in a held/symbolic position — so both
  are register-izable and provably non-escaping; if either escapes, FALL BACK);
- `S1 = Set[L1, E1]`; `S2 = If[SameQ[L1, C1], Return[R1]]` (2-arg If, no else);
  `S3 = Set[L2, E2]`; `S4 = If[COND2, Return[R2]]` (2-arg If); `SF` = the final
  expression (here `TRANS[L1, <recursive call>]`);
- the ONLY `Return`s are the two top-level `If[cond, Return[...]]` (targeting this
  Module's boundary). If the body has any other `Return`, nesting, extra statements,
  or a different arity, FALL BACK.
Be conservative: a missed compile is safe; a wrong compile is a correctness bug.
This shape is REPEATC-specific by construction — it must NEVER match `defineKernel`'s
kernel-state Module (it does not: that Module has a different body).

## Execution semantics (byte-identical to Module + Set + If + Return)

Register file holds `L1`,`L2`. Execute the statement sequence:
1. `L1 = Op::Eval(E1)`.
2. if `SameQ(L1, C1)`: EARLY-EXIT, body result = `Op::Eval(R1)` (only now build/eval R1).
3. `L2 = Op::Eval(E2)`.
4. if truthy(`Op::Eval(COND2)`): EARLY-EXIT, body result = `Op::Eval(R2)`.
5. body result = `Op::Eval(SF)` (SF contains the recursive `REPEATC[c][rhs]`, which is
   just Op::Eval'd → re-dispatches into the compiled REPEATC; no special recursion
   handling — each REPEATC application is its own `evaluate` call, bounded by the
   normal `recursion_limit`).
- EARLY-EXIT = the compiled body returns that value (mirrors `Return` being caught at
  REPEATC's Module boundary).
- An `Op::Eval` that returns `Err(EvalError::Return { .. })` (a Return escaping a
  SUB-call, not REPEATC's own) MUST PROPAGATE unchanged (return the Err) — do NOT
  swallow it as an early-exit; only the compiled body's OWN `Return` ops early-exit.
- Suspension: wrap each `Op::Eval` with `suspend_with_frame_lazy` so an
  `InputRequired` suspension resumes at the right statement (same as slice 2).

## rum / Rust pitfalls (read twice)

1. **Return byte-identity is the trap.** Early-exit = REPEATC's Module returns the
   value; a sub-call's escaping `Err(Return)` must propagate, not early-exit. The
   byte-identical gate (workspace ON + drule_tests) is the proof; fall back if unsure.
2. **Recursion depth / stack safety.** The recursive `REPEATC[c][rhs]` is Op::Eval'd;
   each call is a fresh `evaluate` bounded by `recursion_limit`. Do NOT add unbounded
   NATIVE VM recursion — the VM executes one body's ops iteratively and delegates the
   recursive call to `evaluate` (which manages depth). Confirm a deep REPEATC chain
   does not overflow differently than the tree-walk.
3. **Register-ize only provably non-escaping locals** (the occurrence check). Never
   match kernel-state Module.
4. **Zero-cost off; no new `unsafe`; minimal diff.**
5. **Stop-loss**: byte-identical unmet → narrow the shape / fall back more, or STOP.
   Same failure twice → STOP.
6. **Measurement (if you time anything): persist raw output to a FILE first, then
   parse; rum timing is an exact RATIONAL num/den (parse with Fraction). Probe budget:
   best-of-2, ≤6 probe runs, full env up front, derive timeouts from the historical
   per-file table.**

## Verification (MANDATORY — run exactly; paste verbatim)

```
cargo fmt --check
cargo test --workspace                          # OFF baseline green
RUM_VM_RHS_TEMPLATE=1 cargo test --workspace    # ON must be EQUALLY green (byte-identical proof)
```

Add NEW unit tests: a REPEATC-shape down-value exercising BOTH early-exit paths
(res===convFailed → Return REFL; aconv → Return res) AND the recursive fall-through,
asserting VM result == tree-walk (the debug differential also fires). Then the
combinator differential + optional probe (probe budget applies):

```
# drule_tests.wl directly exercises REPEATC — OFF vs ON, assert identical:
<run tests/drule_tests via mma-hol-bounded-smoke --only test/drule_tests, OFF and ON, diff>
# (optional, nice-to-have) Pair descent OFF vs ON best-of-2 — REPEATC is in the descent:
<runtime-profile-basic-only-pair4 OFF x2 / ON x2, correct simpFixpointConv seconds metric>
```

Paste: both `cargo test --workspace` summaries; the new tests; drule_tests OFF-vs-ON
identical; (if run) the Pair best-of-2 + is_thm lines; and the coverage counter
(REPEATC compiled — a new `module_if_...`/repeatc distinct count).

## Hard rules

- No `git commit`/branch/push. Leave changes in the tree for Claude to gate.
- No files outside `vm_rhs_template.rs` / `definition_application.rs` (+ `mod.rs` if
  needed). Never edit `mma-hol`; no `Expr`/serde change; no new `unsafe`.
- OFF path zero-cost (unchanged gate). **Do NOT flip any default — the VM stays
  opt-in** (`docs/bytecode-vm-design.md` toggle principle).
- Probe budget + timeout discipline + raw-to-file (see pitfall 6).
- Stop-loss: byte-identical unmet → narrow/fall back or STOP; same failure twice → STOP.
- Paste REAL verbatim output; never fake a green.

## Report format

1. What changed — per file/line; the new ops (Return/early-exit), the exact
   compilable REPEATC shape + what falls back; confirm no `Expr`/serde change, no
   gensym for the locals, never matches kernel-state Module.
2. Verification — both `cargo test --workspace` summaries, the new tests, drule_tests
   OFF-vs-ON identical, (optional) Pair, the coverage counter — all verbatim.
3. Finding — whether the Return+recursion machinery is proven byte-identical (the
   capability delivered); any Pair delta if measured (framed as nice-to-have).
4. Stop-loss / open questions — what fell back, any Return/suspension/recursion edge
   you were unsure about.
