# brief-070 — VM slice 1: compiled RHS substitution template

## Goal

Build the FIRST slice of the compiled-body VM (`docs/bytecode-vm-design.md`): a
compiled **RHS substitution template**. Compile a down-value rule RHS ONCE into a
tiny code object (ops `PushConst`/`PushBinding`/`Construct` only — NO `Eval`, NO
control flow), cache it, and on each application EXECUTE the template to construct
the instantiated result instead of the current per-application discovery walk
(`binding_substitution.rs::substitute_rule_bindings_for_rhs`, which walks the whole
RHS tree every time to find what changed). This slice's PRIMARY deliverable is the
proven machinery — the IR + code-object cache + feature flag + tree-walk fallback +
a byte-identical gate — which every later slice reuses; the Pair win is expected to
be MODEST (substitution is only part of the body bucket). It must be behind an
env-gated feature flag that is ZERO-cost when off, and byte-identical to the
tree-walk when on. Keep-or-revert is decided at the gate on the Pair number; either
way the IR/infra is the point.

## Context pointers

- Read `AGENTS.md`, `docs/bytecode-vm-plan.md` (the line: three buckets, do cheap
  levers... — but the user chose route C deliberately, so this slice is step 3),
  and `docs/bytecode-vm-design.md` (THE spec — the IR, interop/fallback model, and
  the invariants; this brief implements its Slice 1). Read PROGRESS
  `## 2026-07-01 brief-069 ...` (the Pair decomposition that motivates it).
- Key files:
  - `crates/rum-runtime/src/runtime/binding_substitution.rs:44` `substitute_bindings`
    / `:76` `substitute_rule_bindings` / `:85` `substitute_rule_bindings_for_rhs` —
    the substitution semantics your template MUST reproduce byte-identically. Note
    the SPECIAL cases (`substitute_rule_scoped_call_changed`,
    `substitute_rule_function_call_changed`,
    `substitute_rule_with_binding_list_changed`): slice 1 does NOT compile these —
    if the RHS contains any construct that needs scoped/function/binding-list
    alpha-handling, mark the body NON-COMPILABLE and FALL BACK to the existing
    path. Slice 1 compiles ONLY the plain-substitution case.
  - `crates/rum-runtime/src/runtime/definition_application.rs` — the down-value
    application path; the `down_values.rhs_substitution`-tagged calls to
    `substitute_rule_bindings_for_rhs` (lines ~193/275/477/544) are the hook points
    where, when the flag is on AND the body is compilable, you run the template
    instead. Keep the subsequent `evaluate` of the result UNCHANGED.
  - `crates/rum-core/src/profiling.rs` `RUM_PROFILE_REPR` — the cold-load
    `AtomicU8` env-gate pattern to MIRROR for the new `RUM_VM_RHS_TEMPLATE` flag.
  - `crates/rum-core/src/expr.rs` — `Expr::Call { head, args }`, `SharedExpr`,
    `SharedArgs`, `ptr_key()` (for keying the compiled-template cache on the RHS
    identity). Do NOT change `Expr`/serde.
  - `scripts/core-language-microbench.sh` (rule-application + construction labels),
    `scripts/pair-simp-hotspot-probes.sh` (`runtime-profile-basic-only-pair4`).
- Acceptance/source clone (READ-ONLY): `/Users/fft/Developments/mma-hol-acceptance`
  (HEAD `200f13a`).

## Scope

- Files you MAY modify:
  - NEW `crates/rum-runtime/src/runtime/vm_rhs_template.rs` — the IR (`enum Op {
    PushConst(SharedExpr), PushBinding(slot), Construct{argc}, ... }`), the
    `compile_rhs_template(rhs) -> Option<CodeObject>` (returns `None` = not
    compilable → fallback), the `execute(code, bindings) -> Expr`, the code-object
    cache (keyed by RHS `ptr_key`, epoch-validated/cleared on epoch bump), and the
    `RUM_VM_RHS_TEMPLATE` cold-load `AtomicU8` gate.
  - `crates/rum-runtime/src/runtime/mod.rs` — wire the module.
  - `crates/rum-runtime/src/runtime/definition_application.rs` — at the RHS
    substitution hook(s), when the flag is on: try the template; on `None`/miss,
    the UNCHANGED existing `substitute_rule_bindings_for_rhs`. Minimal diff.
  - You MAY add read-only helpers but PREFER reusing `binding_substitution`'s
    semantics; do NOT change its behavior.
- MUST NOT touch: `crates/rum-core/src/expr.rs` `Expr`/`Symbol` layout or serde;
  any `mma-hol` clone; `AGENTS.md`/`PROGRESS.md`/`TODO.md`/`docs/**` except your own
  report; `.git`; `Cargo.lock`.
- **No default-path cost.** The hook must be a SINGLE relaxed `AtomicU8` load when
  off, falling straight through to the existing `substitute_rule_bindings_for_rhs`.
  No compile, no cache touch, no branch beyond the one guarded load when off.

## Deliverable

- A working `RUM_VM_RHS_TEMPLATE`-gated compiled RHS template: compiles the
  plain-substitution RHS case, caches it (RHS `ptr_key`, epoch-safe), executes it
  on application, falls back for any non-compilable shape. Byte-identical to the
  tree-walk. OFF ⇒ zero cost.
- The byte-identical evidence + the probe-budgeted Pair/microbench delta (below).

## Correctness (the hard gate — byte-identical or it's a bug)

The template result MUST equal `substitute_rule_bindings_for_rhs` for every
compilable RHS. Slice-1 scope keeps this tractable by COMPILING ONLY the plain case
and falling back otherwise. Preserve: pattern-var binding (incl. `x__` sequence
slots → a slot holding a `Sequence` spliced on `Construct`), `Nothing`-elision and
`Sequence`-splicing exactly as the substitution path does them, and the UNCHANGED
subsequent `evaluate` of the constructed result (do not fold evaluation into the
template — slice 1 is construction only).

## rum / Rust pitfalls (read twice)

1. **Zero-cost when off** (the standing rule): one relaxed `AtomicU8` load, then the
   existing path. Prove with the microbench OFF A/B.
2. **No new `unsafe`** in rum-core/rum-runtime. A `Vec<Op>` + `HashMap<usize,
   (CodeObject, u64 epoch)>` + `AtomicU8` is all this needs.
3. **Suspension safety**: slice 1 does NOT `Eval` inside the template, so it cannot
   suspend mid-template — but the CONSTRUCTED result still flows into the unchanged
   `evaluate`; do not alter that call's suspend/resume shape.
4. **Cache invalidation**: a stale template after a redefinition is a correctness
   bug. Redefinition creates a new RHS `Arc` (new `ptr_key`) so it auto-misses, but
   also clear/epoch-validate the cache on `evaluation_epoch` bump to be safe.
5. **One variable per measurement**; **probe budget** (see below).
6. **Stop-loss**: if the byte-identical gate cannot be met for the plain case,
   NARROW the compilable set further (fall back more) until it is green, or STOP and
   report — never ship a divergence.

## Verification (MANDATORY — run exactly; paste verbatim)

Correctness (byte-identical = run the WHOLE suite both ways):

```
cargo fmt --check
cargo test --workspace                          # flag OFF (baseline green)
RUM_VM_RHS_TEMPLATE=1 cargo test --workspace    # flag ON must be EQUALLY green (byte-identical proof)
```

Both MUST be identically green. Then the Pair result-unchanged check + the delta,
under the **PROBE BUDGET** (`TEMPLATE.md`): best-of-2, cap total runs, full env up
front.

```
# OFF-path zero-cost (microbench, a few labels, current vs a clean main binary):
bash scripts/core-language-microbench.sh --engine rum --output-dir /tmp/cdx_brief_070_micro
# Pair descent, OFF vs ON, best-of-2 (4 runs total), result must stay is_thm=True:
RUM_MMA_HOL_ROOT=/Users/fft/Developments/mma-hol-acceptance \
  bash scripts/pair-simp-hotspot-probes.sh --only runtime-profile-basic-only-pair4 --engine rum --timeout 300 --output-dir /tmp/cdx_brief_070_pair_off
RUM_VM_RHS_TEMPLATE=1 RUM_MMA_HOL_ROOT=/Users/fft/Developments/mma-hol-acceptance \
  bash scripts/pair-simp-hotspot-probes.sh --only runtime-profile-basic-only-pair4 --engine rum --timeout 300 --output-dir /tmp/cdx_brief_070_pair_on
```

Paste: both `cargo test` summaries; the microbench OFF rows; the Pair OFF vs ON
best-of-2 times + ratio + the `is_thm=True changed=False` lines for both; and a
counter (how many distinct RHS bodies compiled vs fell back, so we know coverage).

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the tree
  for Claude to gate.
- Do NOT modify files outside Scope. Never edit any `mma-hol` file. No `Expr`/serde
  change.
- **No new `unsafe`.** **OFF path zero-cost — microbench A/B mandatory.**
- **Probe budget**: best-of-2, ≤6 total probe runs, state the count; full env up
  front; if a number looks off, REPORT — do not silently re-measure everything.
- Stop-loss: byte-identical gate unmet → narrow the compilable set or STOP; same
  failure twice → STOP and report.
- Minimal diff; paste REAL verbatim output; never fake a green.

## Report format (your final message)

1. What changed — per file, line ranges, one-line why; the IR ops implemented and
   the compilable-shape boundary (what falls back).
2. Verification — both `cargo test --workspace` summaries (OFF and ON), the
   microbench OFF rows, the Pair OFF-vs-ON best-of-2 + ratio + is_thm lines, the
   compile-vs-fallback coverage counter — all verbatim.
3. The finding — the Pair delta (kept or not), the compile coverage, and whether
   the machinery (IR/cache/fallback/byte-identical) is proven for later slices.
4. Stop-loss / open questions — what you narrowed to fallback, anything unsure.

---

## Outcome (Claude gate, 2026-07-01) — DONE + KEEP. Slice-1 machinery proven; byte-identical; ~6% Pair win; flag OFF-default.

Committed (the first product-code commit of the VM line). Gated by reproducing the
two load-bearing claims independently.

- **Scope clean:** only `vm_rhs_template.rs` (new) + `definition_application.rs`
  (3 hook swaps) + `mod.rs`; no `Expr`/serde/mma-hol.
- **Byte-identical (the hard gate):** `cargo test --workspace` is 715/0 both OFF and
  `RUM_VM_RHS_TEMPLATE=1` ON — reproduced by Claude. In debug builds the template
  `debug_assert_eq!`s its result against `substitute_rule_bindings_for_rhs` on EVERY
  execution, so a green ON suite = every RHS substitution in the whole suite verified
  identical. New module unit tests cover compile/execute/splice + Module/With/Function
  fallback.
- **OFF zero-cost:** the hook is one relaxed `AtomicU8` load then the unchanged
  `substitute_rule_bindings_for_rhs`.
- **Pair delta (Claude repro, best-of-2, correct `simpFixpointConv` metric):** OFF
  59.88/64.10s, ON 56.23/58.01s → **ON/OFF best = 0.939 (~6.1% faster)**, both runs
  ON < OFF (Codex got 0.932); `is_thm=True changed=False` all runs. Coverage
  `compiled_distinct=140 fallback_distinct=72` (~66% of distinct RHS bodies compiled).
- **Cache correctness:** keyed by RHS `ptr_key` BUT guarded by a `source == *expr`
  structural check (defeats Arc-address reuse) + cleared on `evaluation_epoch` bump +
  a `code_compatible_with_bindings` slot/const check. Sound.

**Verdict: KEEP, committed flag-OFF-default.** The IR + code-object cache + tree-walk
fallback + byte-identical machinery is proven and is the foundation slices 2/3 build
on; a modest ~6% Pair win is bankable and activatable via `RUM_VM_RHS_TEMPLATE=1`.

**Before the default is ever flipped to ON (NOT in this slice):** (1) remove the
per-distinct-body `eprintln!` coverage counter (it would spam stderr in normal use);
(2) replace the global `Mutex<TemplateCache>` with a per-`Session` cache (Session is
`Send`/!`Sync`, so a global lock is unnecessary contention on the hot path); (3) run
a broader corpus differential (the 886-case exact-arith corpus and/or run_fast, OFF
vs ON) beyond the workspace suite. Until then the tree-walker stays the default per
`docs/bytecode-vm-design.md`. Next: slice 2 (add `Eval` + control combinators — the
1.467x amplification, the first slice with a real expected Pair win).
